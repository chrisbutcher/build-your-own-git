use sha1::{Digest, Sha1};
use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

use crate::{objects, HashedWriter};

pub fn hash_object(filename: &Path, write: bool) -> anyhow::Result<String> {
    let mut input_file = fs::File::open(filename)?;
    let file_len = input_file.metadata().expect("get file metadata").len();
    let (_file, object_hash) =
        objects::build_hashed_file(&mut input_file, "blob", file_len as usize, write)?;

    Ok(object_hash)
}
