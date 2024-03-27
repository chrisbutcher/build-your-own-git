use crate::Object;
use std::io::prelude::*;

use crate::objects;

pub fn cat_file(blob_sha: &str) -> anyhow::Result<()> {
    let (_, file_path) = objects::paths_from_sha(blob_sha);

    let loaded_blob = objects::read_object_from_file(&file_path)?;

    match loaded_blob {
        Object::Blob(blob) => {
            let stdout = std::io::stdout();
            stdout.lock().write_all(blob.contents.as_bytes())?;
        }

        Object::Tree(tree) => {
            todo!("TODO cat-file support for trees");
        }
    };

    Ok(())
}
