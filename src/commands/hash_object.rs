use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::{
    fs,
    io::{self, prelude::*, Bytes},
    path::{Path, PathBuf},
};

use crate::{objects, HashedWriter};

pub fn hash_object(filename: &Path, write: bool) -> anyhow::Result<String> {
    let hasher = Sha1::new();

    let mut input_file = fs::File::open(filename)?;

    let uncompressed_tmp_file = tempfile::NamedTempFile::new()?;
    let uncompressed_temp_file_path = uncompressed_tmp_file.path();

    let mut hash_writer = HashedWriter {
        hasher,
        writer: &uncompressed_tmp_file,
    };

    let file_len = input_file.metadata().expect("get file metadata").len();
    write!(hash_writer, "blob {}\0", file_len)?;
    std::io::copy(&mut input_file, &mut hash_writer)?;

    let hash_bytes = hash_writer.hasher.finalize();
    let object_hash = hex::encode(hash_bytes);

    let hash_bytes_vec: Vec<u8> = hash_bytes.iter().cloned().collect();

    if write {
        write_object(&object_hash, &uncompressed_temp_file_path.to_path_buf())?;
    }

    fs::remove_file(uncompressed_temp_file_path)?;

    Ok(object_hash)
}

fn write_object(new_file_hash: &str, source_file_path: &PathBuf) -> anyhow::Result<()> {
    let (dir_path, file_path) = objects::paths_from_sha(new_file_hash);

    // Source
    let mut uncompressed_temp_file_reopened =
        fs::File::open(source_file_path).expect("could not re-open temp file");

    // Destination
    let compressed_tmp_file = tempfile::NamedTempFile::new()?;

    fs::create_dir_all(dir_path).expect("Failed to create objects dir.");

    let mut compressor = ZlibEncoder::new(&compressed_tmp_file, Default::default());
    std::io::copy(&mut uncompressed_temp_file_reopened, &mut compressor)?;
    compressor.finish().expect("Zlib compression failed.");

    // Atomically replace file in object store with tmp file once it's fully written.
    compressed_tmp_file.persist(file_path)?;

    drop(uncompressed_temp_file_reopened);

    Ok(())
}
