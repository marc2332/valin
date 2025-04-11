use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use tokio::fs::{File, OpenOptions};

pub type FSTransport = Arc<Box<dyn FSTransportInterface>>;

#[async_trait]
pub trait FSReadTransportInterface {
    async fn read_to_string(&self, path: &Path) -> tokio::io::Result<String>;
}

#[async_trait]
pub trait FSTransportInterface: FSReadTransportInterface {
    fn as_read(&self) -> Box<dyn FSReadTransportInterface + 'static>;

    async fn open(&self, path: &Path, open_options: &mut OpenOptions) -> tokio::io::Result<File>;

    async fn read_dir(&self, path: &Path) -> tokio::io::Result<tokio::fs::ReadDir>;

    async fn canonicalize(&self, path: &Path) -> tokio::io::Result<PathBuf>;
}
