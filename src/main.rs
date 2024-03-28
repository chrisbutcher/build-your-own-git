use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::io::stdout;
use std::io::Write;
use std::path::PathBuf;

mod commands;
use crate::commands::cat_file::*;
use crate::commands::hash_object::*;
use crate::commands::init::*;
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
                init_repo()?;
            }
            Commands::CatFile {
                pretty_print,
                blob_sha,
            } => {
                if !pretty_print {
                    bail!("Pretty print flag required (for this exercise).")
                }

                cat_file(&blob_sha)?;
            }

            Commands::HashObject { write, filename } => {
                hash_object(&filename, write)?;
            }

            Commands::LsTree {
                name_only,
                tree_sha,
            } => {
                let (_dir_path, file_path) = objects::paths_from_sha(&tree_sha);

                let obj = objects::read_object_from_file(&file_path)?;

                match obj {
                    Object::Tree(tree) => {
                        let mut out = stdout().lock();

                        for entry in &tree.entries {
                            if name_only {
                                writeln!(out, "{}", entry.name).unwrap();
                            } else {
                                let (mode, kind) = match &entry.mode {
                                    TreeEntryMode::RegularFile => ("100644", "blob"),
                                    TreeEntryMode::Directory => ("040000", "tree"),
                                    _ => todo!("printing not supported yet for mode"),
                                };

                                writeln!(
                                    out,
                                    "{} {} {}\t{}",
                                    mode, kind, entry.object_sha, entry.name
                                )
                                .unwrap();
                            }
                        }
                    }
                    _ => todo!("ls-tree unhandled object type"),
                }
            }
        }
    }

    Ok(())
}
