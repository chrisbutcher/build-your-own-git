use crate::objects;
use std::{
    env,
    io::{Cursor, Write},
    path::Path,
};
use walkdir::WalkDir;

use crate::{commands, TreeEntry};

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

    let content_size = contents_bytes.len();
    let mut c = Cursor::new(contents_bytes);
    let (_file, tree_hash) = objects::build_hashed_file(&mut c, "tree", content_size, true)?;

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
