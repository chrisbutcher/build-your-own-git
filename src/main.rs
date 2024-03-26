use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::{ffi::CStr, fs};

// clap docs: https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_0/index.html
#[derive(Subcommand)]
enum Commands {
    /// Initialize git repo in the current folder.
    Init,

    /// Print internal git object file contents
    CatFile {
        /// pretty prints blob contents
        #[arg(id = "pretty_print", short, long, value_name = "blob_sha")]
        blob_sha: String,
    },

    /// Print internal git object file contents
    HashObject {
        /// pretty prints blob contents
        #[arg(id = "write", short, long, value_name = "filename")]
        filename: String,
    },
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

struct Blob {
    size: usize,
    contents: String,
}

enum Object {
    Blob(Blob),
}

fn read_object(file_path: &Path) -> anyhow::Result<Object> {
    let f = File::open(file_path)?;

    let encoded_reader = BufReader::new(f);
    let zlib_decoder = ZlibDecoder::new(encoded_reader);
    let mut decoded_reader = BufReader::new(zlib_decoder);

    let mut header_buf = Vec::new();
    let _header_bytes_read = decoded_reader
        .read_until(0x0, &mut header_buf)
        .context("Reading header of object file")?;

    let header_c_str =
        CStr::from_bytes_with_nul(&header_buf).context("Failed to read header as cstr")?;
    let header_str = header_c_str
        .to_str()
        .context("Failed to convert cstr to str")?;
    let Some((kind, size)) = header_str.split_once(" ") else {
        anyhow::bail!("Header was malformed");
    };

    // Read header into c Str and strip out the blob and space
    let size = size
        .parse::<usize>()
        .context("Failed to parse {size} as usize")?;

    let mut buf = Vec::new();
    decoded_reader.read_to_end(&mut buf)?;
    let str = String::from_utf8(buf)?;

    let result = match kind {
        "blob" => Object::Blob(Blob {
            size: size,
            contents: str,
        }),
        _ => anyhow::bail!("object kind ({}) not supported", kind),
    };

    Ok(result)
}

fn main() -> Result<()> {
    // let path = env::current_dir().unwrap();
    // println!("The current directory is {}", path.display());

    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(commands) = cli.command {
        match commands {
            Commands::Init => {
                fs::create_dir(".git").unwrap();
                fs::create_dir(".git/objects").unwrap();
                fs::create_dir(".git/refs").unwrap();
                fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
                println!("Initialized git directory")
            }
            Commands::CatFile { blob_sha } => {
                let blob_prefix = &blob_sha[0..2];
                let blob_suffix = &blob_sha[2..];

                let raw_path = format!(".git/objects/{}/{}", blob_prefix, blob_suffix);
                let file_path = Path::new(&raw_path);

                let loaded_blob = read_object(&file_path)?;

                match loaded_blob {
                    Object::Blob(blob) => {
                        let stdout = std::io::stdout();
                        stdout.lock().write_all(blob.contents.as_bytes())?;
                    }
                }
            }

            Commands::HashObject { filename } => {
                let bytes = fs::read(filename).expect("Could not read file.");

                let mut hasher = Sha1::new();
                let header = format!("blob {}\0", bytes.len());
                hasher.update(&header);
                hasher.update(&bytes);
                let sha1 = hasher.finalize();
                let hex_hash = hex::encode(sha1);

                let (prefix, filename) = hex_hash.split_at(2);
                let mut path = String::from(format!(".git/objects/{}", prefix));

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

                println!("{}", hex_hash);
            }
        }
    }

    Ok(())
}
