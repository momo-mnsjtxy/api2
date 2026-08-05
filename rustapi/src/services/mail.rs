use crate::config::AppConfig;
use std::path::PathBuf;

pub async fn send_email(_cfg: &AppConfig, to: &str, title: &str, body: &str) -> anyhow::Result<()> {
    let dir = PathBuf::from("runtime/mail");
    tokio::fs::create_dir_all(&dir).await.ok();
    let file = dir.join(format!(
        "{}.eml",
        chrono::Local::now().format("%Y%m%d%H%M%S")
    ));
    let content = format!("To: {to}\nSubject: {title}\n\n{body}");
    tokio::fs::write(file, content).await?;
    Ok(())
}
