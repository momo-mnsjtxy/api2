use crate::config::AppConfig;
use anyhow::{bail, Context};
use base64::{engine::general_purpose::STANDARD, Engine};
use rustls::{ClientConfig, ServerName};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

/// Send HTML email via SMTP when credentials are configured; otherwise write `.eml` to disk.
pub async fn send_email(cfg: &AppConfig, to: &str, title: &str, body: &str) -> anyhow::Result<()> {
    if cfg.smtp_pass.is_empty() || cfg.smtp_user.is_empty() {
        return write_eml(cfg, to, title, body, None).await;
    }
    match smtp_send(cfg, to, title, body).await {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!(error = %e, "smtp send failed; falling back to .eml");
            write_eml(cfg, to, title, body, Some(&e.to_string())).await
        }
    }
}

async fn write_eml(
    cfg: &AppConfig,
    to: &str,
    title: &str,
    body: &str,
    note: Option<&str>,
) -> anyhow::Result<()> {
    let dir = PathBuf::from("runtime/mail");
    tokio::fs::create_dir_all(&dir).await.ok();
    let file = dir.join(format!(
        "{}.eml",
        chrono::Local::now().format("%Y%m%d%H%M%S%.3f")
    ));
    let note_hdr = note
        .map(|n| format!("X-Fallback-Note: {n}\r\n"))
        .unwrap_or_default();
    let content = format!(
        "{note_hdr}From: {}\r\nTo: {to}\r\nSubject: {title}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{body}",
        cfg.smtp_from
    );
    tokio::fs::write(file, content).await?;
    Ok(())
}

async fn smtp_send(cfg: &AppConfig, to: &str, title: &str, body: &str) -> anyhow::Result<()> {
    let addr = format!("{}:{}", cfg.smtp_host, cfg.smtp_port);
    let tcp = TcpStream::connect(&addr)
        .await
        .with_context(|| format!("connect {addr}"))?;
    tcp.set_nodelay(true).ok();

    let connector = tls_connector();
    let server_name = ServerName::try_from(cfg.smtp_host.as_str())
        .map_err(|_| anyhow::anyhow!("invalid SMTP host for TLS"))?
        .to_owned();

    let mut tls_stream: TlsStream<TcpStream> = if cfg.smtp_port == 465 {
        connector
            .connect(server_name, tcp)
            .await
            .context("implicit TLS connect")?
    } else {
        // STARTTLS (587/25)
        let (reader, mut writer) = tcp.into_split();
        let mut reader = BufReader::new(reader);
        expect_code(&mut reader, b"220").await?;
        cmd(
            &mut writer,
            &mut reader,
            &format!("EHLO {}\r\n", ehlo_name(cfg)),
            b"250",
        )
        .await?;
        cmd(&mut writer, &mut reader, "STARTTLS\r\n", b"220").await?;
        // Reunite split halves for TLS upgrade.
        let tcp = reader
            .into_inner()
            .reunite(writer)
            .map_err(|_| anyhow::anyhow!("reunite tcp for starttls"))?;
        connector
            .connect(server_name, tcp)
            .await
            .context("STARTTLS handshake")?
    };

    handshake_and_send(&mut tls_stream, cfg, to, title, body).await
}

async fn handshake_and_send(
    stream: &mut TlsStream<TcpStream>,
    cfg: &AppConfig,
    to: &str,
    title: &str,
    body: &str,
) -> anyhow::Result<()> {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);

    // Implicit TLS (465): server greets first. STARTTLS: re-EHLO immediately.
    if cfg.smtp_port == 465 {
        expect_code(&mut reader, b"220").await?;
    }

    cmd(
        &mut writer,
        &mut reader,
        &format!("EHLO {}\r\n", ehlo_name(cfg)),
        b"250",
    )
    .await?;

    let user_b64 = STANDARD.encode(cfg.smtp_user.as_bytes());
    let pass_b64 = STANDARD.encode(cfg.smtp_pass.as_bytes());
    cmd(&mut writer, &mut reader, "AUTH LOGIN\r\n", b"334").await?;
    cmd(&mut writer, &mut reader, &format!("{user_b64}\r\n"), b"334").await?;
    cmd(&mut writer, &mut reader, &format!("{pass_b64}\r\n"), b"235").await?;
    cmd(
        &mut writer,
        &mut reader,
        &format!("MAIL FROM:<{}>\r\n", cfg.smtp_from),
        b"250",
    )
    .await?;
    cmd(
        &mut writer,
        &mut reader,
        &format!("RCPT TO:<{to}>\r\n"),
        b"250",
    )
    .await?;
    cmd(&mut writer, &mut reader, "DATA\r\n", b"354").await?;

    let subject = encode_subject(title);
    let mut data = format!(
        "From: 与梦城 <{}>\r\nTo: {to}\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{body}\r\n.\r\n",
        cfg.smtp_from
    );
    data = data.replace("\n.", "\n..");
    writer.write_all(data.as_bytes()).await?;
    expect_code(&mut reader, b"250").await?;
    let _ = cmd(&mut writer, &mut reader, "QUIT\r\n", b"221").await;
    Ok(())
}

fn ehlo_name(cfg: &AppConfig) -> &str {
    if cfg.smtp_host.is_empty() {
        "localhost"
    } else {
        &cfg.smtp_host
    }
}

fn tls_connector() -> TlsConnector {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    TlsConnector::from(Arc::new(config))
}

fn encode_subject(title: &str) -> String {
    if title.is_ascii() {
        return title.to_string();
    }
    format!("=?UTF-8?B?{}?=", STANDARD.encode(title.as_bytes()))
}

async fn cmd<W, R>(writer: &mut W, reader: &mut R, cmd: &str, expect: &[u8]) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
    R: AsyncBufReadExt + Unpin,
{
    writer.write_all(cmd.as_bytes()).await?;
    expect_code(reader, expect).await
}

async fn expect_code<R: AsyncBufReadExt + Unpin>(reader: &mut R, expect: &[u8]) -> anyhow::Result<()> {
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            bail!("smtp eof");
        }
        if line.len() >= 4 && line.as_bytes()[3] == b' ' {
            if !line.as_bytes().starts_with(expect) {
                bail!("smtp unexpected: {}", line.trim_end());
            }
            return Ok(());
        }
    }
}
