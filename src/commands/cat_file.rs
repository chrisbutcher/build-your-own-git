use crate::Object;

use crate::objects;

pub fn cat_file(blob_sha: &str) -> anyhow::Result<String> {
    let (_, file_path) = objects::paths_from_sha(blob_sha);

    let loaded_blob = objects::read_object_from_file(&file_path)?;

    match loaded_blob {
        Object::Blob(blob) => Ok(blob.contents),

        Object::Tree(_tree) => {
            todo!("cat-file support for trees");
        }
    }
}
