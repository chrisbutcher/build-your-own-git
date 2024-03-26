use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
use crate::commands::cat_file::*;
use crate::commands::hash_object::*;
use crate::commands::init::*;

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
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

pub struct Blob {
    #[allow(dead_code)]
    size: usize,
    contents: String,
}

pub enum Object {
    Blob(Blob),
}

fn main() -> Result<()> {
    // let path = env::current_dir().unwrap();
    // println!("The current directory is {}", path.display());

    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(commands) = cli.command {
        match commands {
            Commands::Init => {
                init_repo();
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
                hash_object(&filename, write);
            }
        }
    }

    Ok(())
}
