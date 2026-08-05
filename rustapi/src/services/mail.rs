use crate::config::AppConfig;
use anyhow::Context;
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use std::path::PathBuf;

pub async fn send_email(cfg: &AppConfig, to: &str, title: &str, body: &str) -> anyhow::Result<()> {
    if cfg.smtp_pass.is_empty() {
        // Dev fallback: write to runtime/mail
        let dir = PathBuf::from("runtime/mail");
        tokio::fs::create_dir_all(&dir).await.ok();
        let file = dir.join(format!(
            "{}.eml",
            chrono::Local::now().format("%Y%m%d%H%M%S")
        ));
        let content = format!("To: {to}\nSubject: {title}\n\n{body}");
        tokio::fs::write(file, content).await?;
        return Ok(());
    }

    let from: Mailbox = format!("与梦城 <{}>", cfg.smtp_from)
        .parse()
        .context("smtp from")?;
    let to_mb: Mailbox = to.parse().context("smtp to")?;
    let email = Message::builder()
        .from(from)
        .to(to_mb)
        .subject(title)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(body.to_string())?;

    let creds = Credentials::new(cfg.smtp_user.clone(), cfg.smtp_pass.clone());
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)?
            .port(cfg.smtp_port)
            .credentials(creds)
            .build();
    mailer.send(email).await?;
    Ok(())
}
