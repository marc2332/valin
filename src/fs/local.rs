use async_trait::async_trait;
use tokio::fs::OpenOptions;

use super::FSTransportInterface;

pub struct FSLocal;

#[async_trait]
impl FSTransportInterface for FSLocal {
    async fn read_to_string(&self, path: &std::path::Path) -> tokio::io::Result<String> {
        tokio::fs::read_to_string(path).await
    }
    async fn open(
        &self,
        path: &std::path::Path,
        open_options: &mut OpenOptions,
    ) -> tokio::io::Result<tokio::fs::File> {
        open_options.open(path).await
    }

    async fn read_dir(&self, path: &std::path::Path) -> tokio::io::Result<tokio::fs::ReadDir> {
        tokio::fs::read_dir(path).await
    }

    async fn canonicalize(&self, path: &std::path::Path) -> tokio::io::Result<std::path::PathBuf> {
        tokio::fs::canonicalize(path).await
    }
}
