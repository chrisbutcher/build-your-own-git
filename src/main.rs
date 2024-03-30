use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use commands::ls_tree;
use std::path::PathBuf;

use sha1::{Digest, Sha1};
use std::{io, io::prelude::*};

use crate::commands::*;
mod commands;
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

    /// TODO
    WriteTree,

    /// TODO
    CommitTree {
        #[clap(short, long)]
        /// TODO
        parent_sha: Option<String>,

        #[clap(short, long)]
        /// TODO
        message: String,

        /// TODO
        tree_sha: String,
    },
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

struct HashedWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> io::Write for HashedWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

fn main() -> Result<()> {
    // let path = env::current_dir().unwrap();
    // println!("The current directory is {}", path.display());

    let cli = Cli::parse();

    // TODO: Decide if/how I/O should be extracted from core logic in commands.

    // You can check the value provided by positional arguments, or option arguments
    if let Some(commands) = cli.command {
        match commands {
            Commands::Init => {
                init::init()?;
                println!("Initialized git directory");
            }

            Commands::CatFile {
                pretty_print,
                blob_sha,
            } => {
                if !pretty_print {
                    bail!("Pretty print flag required (for this exercise).")
                }

                let contents = cat_file::cat_file(&blob_sha)?;

                let stdout = std::io::stdout();
                stdout.lock().write_all(contents.as_bytes())?;
            }

            Commands::HashObject { write, filename } => {
                let object_hash = hash_object::hash_object(&filename, write)?;

                println!("{}", object_hash);
            }

            Commands::LsTree {
                name_only,
                tree_sha,
            } => {
                ls_tree::ls_tree(&tree_sha, name_only)?;
            }

            Commands::WriteTree => {
                let tree_entry = write_tree::write_tree()?;
                println!("{}", tree_entry.object_sha);
            }

            Commands::CommitTree {
                tree_sha,
                parent_sha,
                message,
            } => {
                let commit_hash = commit_tree::commit_tree(tree_sha, parent_sha, message)?;

                println!("{}", commit_hash);
            }
        }
    }

    Ok(())
}
