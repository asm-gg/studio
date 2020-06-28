use std::pin::Pin;

use async_std::fs::File;
use async_std::path::Path;
use async_std::prelude::*;

pub struct FileHandle {
    pub data: Pin<Box<[u8]>>,
    pub name: String,
}

impl FileHandle {
    pub async fn new(at: &Path) -> std::io::Result<Self> {
        let mut file = File::open(at).await?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;

        // if this wasn't a file with a name we would have already bailed out
        let filename = at.file_name().unwrap();

        Ok(Self {
            data: Box::into_pin(buf.into_boxed_slice()),
            name: String::from(filename.to_string_lossy()),
        })
    }
}
