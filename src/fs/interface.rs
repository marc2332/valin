use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use smol::fs::{File, OpenOptions};

pub type FSTransport = Arc<Box<dyn FSTransportInterface>>;

#[async_trait::async_trait]
pub trait FSReadTransportInterface {
    async fn read_to_string(&self, path: &Path) -> smol::io::Result<String>;
}

#[async_trait::async_trait]
pub trait FSTransportInterface: FSReadTransportInterface {
    fn as_read(&self) -> Box<dyn FSReadTransportInterface + 'static>;

    async fn open(&self, path: &Path, open_options: &mut OpenOptions) -> smol::io::Result<File>;

    async fn read_dir(&self, path: &Path) -> smol::io::Result<smol::fs::ReadDir>;

    async fn canonicalize(&self, path: &Path) -> smol::io::Result<PathBuf>;
}
