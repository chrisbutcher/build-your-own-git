use crate::{Blob, Object};
use anyhow::Context;
use flate2::bufread::ZlibDecoder;
use std::ffi::CStr;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
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
    let str = String::from_utf8(buf)?;

    let result = match kind {
        "blob" => Object::Blob(Blob {
            size,
            contents: str,
        }),
        _ => anyhow::bail!("object kind ({}) not supported", kind),
    };

    Ok(result)
}