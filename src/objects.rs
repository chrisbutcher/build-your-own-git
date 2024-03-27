use crate::{Blob, Object, Tree};
use crate::{TreeEntry, TreeEntryMode};
use anyhow::Context;
use bytes::Buf;
use flate2::bufread::ZlibDecoder;
use std::ffi::CStr;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;

pub fn paths_from_sha(object_sha: &str) -> (PathBuf, PathBuf) {
    let (prefix, filename) = object_sha.split_at(2);

    let raw_dir_path = format!(".git/objects/{}", prefix);
    let dir_path = Path::new(&raw_dir_path);

    let raw_file_path = format!(".git/objects/{}/{}", prefix, filename);
    let file_path = Path::new(&raw_file_path);

    (dir_path.to_path_buf(), file_path.to_path_buf())
}

pub fn read_object_from_file(file_path: &Path) -> anyhow::Result<Object> {
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
    let Some((kind, size)) = header_str.split_once(' ') else {
        anyhow::bail!("Header was malformed");
    };

    let size = size
        .parse::<usize>()
        .context("Failed to parse {size} as usize")?;

    // Finish reading object contents as UTF-8
    let mut buf = Vec::new();
    decoded_reader.read_to_end(&mut buf)?;

    let result = match kind {
        "blob" => {
            let str = String::from_utf8(buf)?; // BUG when reading tree objects

            Object::Blob(Blob {
                size,
                contents: str,
            })
        }
        "tree" => {
            // Read remaining tree contents as C-strings
            let mut c = Cursor::new(buf);
            let mut entries = Vec::new();

            loop {
                let mut tree_line_buf = Vec::new();

                c.read_until(0x0, &mut tree_line_buf)?;

                let tree_line_cstr = CStr::from_bytes_with_nul(&tree_line_buf)
                    .context("Failed to read header as cstr")?;
                let tree_line_str = tree_line_cstr
                    .to_str()
                    .context("Failed to convert cstr to str")?;

                let Some((raw_mode, filename)) = tree_line_str.split_once(' ') else {
                    anyhow::bail!("tree entry was malformed");
                };

                let mode = match raw_mode {
                    "40000" => TreeEntryMode::Directory,
                    "100644" => TreeEntryMode::RegularFile,
                    _ => {
                        todo!("Unhandled tree entry mode: {}", raw_mode);
                    }
                };

                // Read 20-byte sha hash from buffer wrapped in cursor `c``.
                let mut tree_line_sha_buf = [0; 20];
                c.read_exact(&mut tree_line_sha_buf)?;
                let tree_line_hash = hex::encode(tree_line_sha_buf);

                let new_entry = TreeEntry {
                    mode: mode,
                    name: filename.to_string(),
                    object_sha: tree_line_hash,
                };

                entries.push(new_entry);

                if !c.has_remaining() {
                    break;
                }
            }

            Object::Tree(Tree { entries })
        }
        _ => anyhow::bail!("object kind ({}) not supported", kind),
    };

    Ok(result)
}
