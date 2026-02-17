use smol::fs::OpenOptions;

use super::{FSReadTransportInterface, FSTransportInterface};

pub struct FSLocal;

#[async_trait::async_trait]
impl FSReadTransportInterface for FSLocal {
    async fn read_to_string(&self, path: &std::path::Path) -> smol::io::Result<String> {
        smol::fs::read_to_string(path).await
    }
}

#[async_trait::async_trait]
impl FSTransportInterface for FSLocal {
    fn as_read(&self) -> Box<dyn FSReadTransportInterface + 'static> {
        Box::new(FSLocal)
    }

    async fn open(
        &self,
        path: &std::path::Path,
        open_options: &mut OpenOptions,
    ) -> smol::io::Result<smol::fs::File> {
        open_options.open(path).await
    }

    async fn read_dir(&self, path: &std::path::Path) -> smol::io::Result<smol::fs::ReadDir> {
        smol::fs::read_dir(path).await
    }

    async fn canonicalize(&self, path: &std::path::Path) -> smol::io::Result<std::path::PathBuf> {
        smol::fs::canonicalize(path).await
    }
}
