use std::{env, path::PathBuf};
use walkdir::WalkDir;

use crate::{commands, TreeEntry};

pub fn write_tree() -> anyhow::Result<TreeEntry> {
    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());

    let tree_entry = build_tree_entry(&path);

    tree_entry
}

fn build_tree_entry(path: &PathBuf) -> anyhow::Result<TreeEntry> {
    let path_str = path.to_str().unwrap();

    let walker = WalkDir::new(&path)
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

        let path_string = fs_entry.path().display().to_string();
        let path_with_trailing_slash = format!("{}/", path_str);
        let relative_path = path_string.replace(&path_with_trailing_slash, "");

        let mut relative_path_buf = PathBuf::new();
        relative_path_buf.push(&relative_path);

        let path_buf = fs_entry.path().to_path_buf();

        // println!("{}", path_string);

        if fs_entry.file_type().is_dir() {
            println!("Recursing into directory: {:?}", path_buf);

            let tree_entry = build_tree_entry(&path_buf)?;
            tree_entries.push(tree_entry);
        } else {
            let object_hash = commands::hash_object::hash_object(&path_buf, true)?;

            tree_entries.push(TreeEntry {
                mode: crate::TreeEntryMode::RegularFile,
                name: relative_path.clone(),
                object_sha: object_hash,
            });
        }
    }

    // TODO Create tree object and then return its values here:
    // 0. Build up tree entry lines, get byte size for header.
    // 1. Write header
    // 2. Write each entry line
    // 3. Produce SHA-1 hash of the bytes written in total, to return as this tree's tree sha.

    println!("reporting all tree_entries::START:::");

    for tree_entry in tree_entries.iter() {
        println!(
            "mode: {:?}, name: {:?}, object_sha: {:?}",
            tree_entry.mode, tree_entry.name, tree_entry.object_sha
        );
    }

    println!("reporting all tree_entries::END:::");

    Ok(TreeEntry {
        mode: crate::TreeEntryMode::RegularFile,
        name: "foobar".to_string(),
        object_sha: "abc".to_string(),
    })
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
