use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use tokio::fs::{File, OpenOptions};

pub type FSTransport = Arc<Box<dyn FSTransportInterface>>;

#[async_trait]
pub trait FSTransportInterface {
    async fn read_to_string(&self, path: &Path) -> tokio::io::Result<String>;

    async fn open(&self, path: &Path, open_options: &mut OpenOptions) -> tokio::io::Result<File>;

    async fn read_dir(&self, path: &Path) -> tokio::io::Result<tokio::fs::ReadDir>;
}
