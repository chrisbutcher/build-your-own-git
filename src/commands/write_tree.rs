use crate::objects;
use bytes::Buf;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::{
    env, fs,
    io::{self, Cursor, Write},
    path::Path,
};
use walkdir::WalkDir;

use crate::{commands, HashedWriter, TreeEntry};

pub fn write_tree() -> anyhow::Result<TreeEntry> {
    let path = env::current_dir()?;

    build_tree_entry(&path)
}

fn build_tree_entry(path: &Path) -> anyhow::Result<TreeEntry> {
    let walker = WalkDir::new(path)
        .follow_links(false)
        .max_depth(1)
        .into_iter();

    let mut tree_entries: Vec<TreeEntry> = Vec::new();

    for fs_entry in walker.filter_entry(|e| !is_hidden(e)) {
        let fs_entry = fs_entry.unwrap();
        if fs_entry.path() == path {
            // Skip the parent path itself.
            continue;
        }

        let relative_path = fs_entry.path().file_name().unwrap();

        if fs_entry.file_type().is_dir() {
            let tree_entry = build_tree_entry(fs_entry.path())?;
            tree_entries.push(tree_entry);
        } else {
            let object_hash = commands::hash_object::hash_object(fs_entry.path(), true)?;

            tree_entries.push(TreeEntry {
                mode: crate::TreeEntryMode::RegularFile,
                name: relative_path.to_str().unwrap().to_string(),
                object_sha: object_hash,
            });
        }
    }

    // Create tree object and then return its values here:
    // 0. Build up tree entry lines, get byte size for header.
    // 1. Write header
    // 2. Write each entry line
    // 3. Produce SHA-1 hash of the bytes written in total, to return as this tree's tree sha.

    tree_entries.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

    let mut contents_bytes = Vec::new();

    for tree_entry in tree_entries.iter() {
        let mode = match tree_entry.mode {
            crate::TreeEntryMode::RegularFile => "100644",
            crate::TreeEntryMode::Directory => "40000",
            _ => todo!("writing files of this mode not supported"),
        };

        write!(&mut contents_bytes, "{} {}\0", mode, tree_entry.name)?;
        contents_bytes.extend(hex::decode(&tree_entry.object_sha)?);
    }

    let tree_bytes = Vec::new();
    let mut hashed_writer = HashedWriter {
        hasher: Sha1::new(),
        writer: tree_bytes,
    };

    write!(&mut hashed_writer, "tree {}\0", contents_bytes.len())?;

    let mut c = Cursor::new(contents_bytes);
    io::copy(&mut c, &mut hashed_writer)?;

    let hash_bytes = hashed_writer.hasher.finalize();
    let tree_hash = hex::encode(hash_bytes);

    let (dir_path, file_path) = objects::paths_from_sha(&tree_hash);
    fs::create_dir_all(dir_path).expect("Failed to create objects dir.");

    // Source
    let mut reader_tree_bytes = hashed_writer.writer.reader();

    // Destination
    let compressed_tmp_file = tempfile::NamedTempFile::new()?;

    let mut compressor = ZlibEncoder::new(&compressed_tmp_file, Default::default());
    std::io::copy(&mut reader_tree_bytes, &mut compressor)?;
    compressor.finish().expect("Zlib compression failed.");

    // Atomically replace file in object store with tmp file once it's fully written.
    compressed_tmp_file.persist(file_path)?;

    Ok(TreeEntry {
        mode: crate::TreeEntryMode::Directory,
        name: path.file_name().unwrap().to_str().unwrap().to_string(),
        object_sha: tree_hash,
    })
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
