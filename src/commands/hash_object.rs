use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn hash_object(filename: &PathBuf, write: bool) {
    let bytes = fs::read(filename).expect("Could not read file.");

    let mut hasher = Sha1::new();
    let header = format!("blob {}\0", bytes.len());
    hasher.update(&header);
    hasher.update(&bytes);
    let sha1 = hasher.finalize();
    let hex_hash = hex::encode(sha1);

    let (prefix, filename) = hex_hash.split_at(2);
    let mut path = String::from(format!(".git/objects/{}", prefix));

    if write {
        fs::create_dir_all(&path).expect("Failed to create objects dir.");

        let mut compressor = ZlibEncoder::new(Vec::new(), Default::default());
        compressor
            .write_all(header.as_bytes())
            .expect("Failed to compress blob header.");
        compressor
            .write_all(&bytes)
            .expect("Failed to compress blob content.");

        let compressed = compressor.finish().expect("Zlib compression failed.");

        path.push_str("/");
        path.push_str(filename);

        // TODO: Switch to creating tmp file to write contents into and then renaming it instead (for perf).
        fs::write(path, compressed.as_slice()).expect("Failed to write object to disk.");
    }

    println!("{}", hex_hash);
}
