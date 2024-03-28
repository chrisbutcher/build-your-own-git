use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use commands::ls_tree;
use std::path::PathBuf;
use walkdir::WalkDir;
mod commands;
use std::env;

use crate::commands::*;
pub mod objects;

// clap docs: https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_0/index.html
#[derive(Subcommand)]
enum Commands {
    /// Initialize git repo in the current folder
    Init,

    /// Print internal git object file contents
    CatFile {
        /// pretty prints blob contents
        #[arg(
            id = "pretty_print",
            short,
            long,
            value_name = "blob_sha",
            default_value_t = true
        )]
        pretty_print: bool,

        blob_sha: String,
    },

    /// Print internal git object file contents
    HashObject {
        #[clap(short, long, default_value_t = false)]
        /// Write computed object to disk
        write: bool,

        /// target file
        filename: PathBuf,
    },

    /// Print tree object contents
    LsTree {
        #[clap(short, long, default_value_t = false)]
        /// Print out only file names
        name_only: bool,

        /// Target tree file by its SHA-1 hash
        tree_sha: String,
    },

    WriteTree,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug)]
pub struct Blob {
    #[allow(dead_code)]
    size: usize,
    contents: String,
}

#[derive(Debug)]
pub enum TreeEntryMode {
    RegularFile,
    ExecutableFile,
    SymbolicLink,
    Directory,
}

#[derive(Debug)]
pub struct TreeEntry {
    mode: TreeEntryMode,
    name: String,
    object_sha: String,
}

#[derive(Debug)]
pub struct Tree {
    entries: Vec<TreeEntry>,
}

#[derive(Debug)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
}

fn main() -> Result<()> {
    // let path = env::current_dir().unwrap();
    // println!("The current directory is {}", path.display());

    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(commands) = cli.command {
        match commands {
            Commands::Init => {
                init::init()?;
            }
            Commands::CatFile {
                pretty_print,
                blob_sha,
            } => {
                if !pretty_print {
                    bail!("Pretty print flag required (for this exercise).")
                }

                cat_file::cat_file(&blob_sha)?;
            }

            Commands::HashObject { write, filename } => {
                hash_object::hash_object(&filename, write)?;
            }

            Commands::LsTree {
                name_only,
                tree_sha,
            } => {
                ls_tree::ls_tree(&tree_sha, name_only)?;
            }

            Commands::WriteTree => {
                let path = env::current_dir()?;
                println!("The current directory is {}", path.display());

                let tree_entries = recurse_path(&path);
            }
        }
    }

    Ok(())
}

fn recurse_path(path: &PathBuf) {
    let path_str = path.to_str().unwrap();

    let walker = WalkDir::new(&path)
        .follow_links(false)
        .max_depth(1)
        .into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();

        let p = entry.path().display().to_string();

        if entry.path() != path {
            println!("{}", p);
            println!("is_dir: {}", entry.file_type().is_dir());

            if entry.file_type().is_dir() {
                let pb = entry.path().to_path_buf();

                recurse_path(&pb);
            }

            let path_with_trailing_slash = format!("{}/", path_str);
            let relative_path = p.replace(&path_with_trailing_slash, "");

            println!("relative_path: {}", relative_path);
        }
    }
}
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
