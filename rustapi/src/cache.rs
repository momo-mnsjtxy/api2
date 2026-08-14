use anyhow::Context;
use moka::future::Cache;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;

#[derive(Clone)]
pub struct AppCache {
    memory: Cache<String, String>,
    dir: PathBuf,
}

impl AppCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let _ = std::fs::create_dir_all(&dir);
        Self {
            memory: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(43_200))
                .build(),
            dir,
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        if let Some(v) = self.memory.get(key).await {
            return Some(v);
        }
        let path = self.path_for(key);
        match fs::read_to_string(&path).await {
            Ok(raw) => {
                let (expire, data) = Self::parse_file(&raw)?;
                if expire > 0 && expire < chrono::Utc::now().timestamp() {
                    let _ = fs::remove_file(path).await;
                    return None;
                }
                self.memory.insert(key.to_string(), data.clone()).await;
                Some(data)
            }
            Err(_) => None,
        }
    }

    pub async fn set(
        &self,
        key: &str,
        value: impl Into<String>,
        ttl_secs: i64,
    ) -> anyhow::Result<()> {
        let value = value.into();
        self.memory.insert(key.to_string(), value.clone()).await;
        let expire = if ttl_secs > 0 {
            chrono::Utc::now().timestamp() + ttl_secs
        } else {
            0
        };
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.ok();
        }
        let payload = format!("{expire}\n{value}");
        fs::write(path, payload)
            .await
            .with_context(|| format!("write cache {key}"))?;
        Ok(())
    }

    fn path_for(&self, key: &str) -> PathBuf {
        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(key.as_bytes());
        let digest = hex::encode(hasher.finalize());
        self.dir.join(format!("{digest}.cache"))
    }

    fn parse_file(raw: &str) -> Option<(i64, String)> {
        let (expire_s, rest) = raw.split_once('\n')?;
        let expire = expire_s.parse().ok()?;
        Some((expire, rest.to_string()))
    }
}
