use std::pin::Pin;

use async_std::prelude::*;
use async_std::path::Path;
use async_std::fs::File;

pub struct FileHandle {
    pub data: Pin<Box<[u8]>>,
}

impl FileHandle {
    pub async fn new(at: &Path) -> std::io::Result<Self> {
        let mut file = File::open(at).await?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;

        Ok(Self { data: Box::into_pin(buf.into_boxed_slice()) })
    }
}
