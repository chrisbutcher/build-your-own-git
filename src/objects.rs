use crate::{Blob, Object, Tree, TreeEntry, TreeEntryMode};
use anyhow::Context;
use bytes::Buf;
use flate2::{bufread::ZlibDecoder, write::ZlibEncoder};
use std::{
    ffi::CStr,
    fs::{self, File},
    io::{prelude::*, BufReader, Cursor},
    path::{Path, PathBuf},
};

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

    let mut buf = Vec::new();
    decoded_reader.read_to_end(&mut buf)?;

    let result = match kind {
        "blob" => {
            // Finish reading object contents as UTF-8
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
                let mut tree_entry_buf = Vec::new();

                c.read_until(0x0, &mut tree_entry_buf)?;

                let tree_entry_cstr = CStr::from_bytes_with_nul(&tree_entry_buf)
                    .context("Failed to read header as cstr")?;
                let tree_entry_str = tree_entry_cstr
                    .to_str()
                    .context("Failed to convert cstr to str")?;

                let Some((raw_mode, name)) = tree_entry_str.split_once(' ') else {
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
                let mut tree_entry_sha_buf = [0; 20];
                c.read_exact(&mut tree_entry_sha_buf)?;
                let tree_entry_hash = hex::encode(tree_entry_sha_buf);

                let new_entry = TreeEntry {
                    mode,
                    name: name.to_string(),
                    object_sha: tree_entry_hash,
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

pub fn write_byte_reader_to_file<R: ?Sized>(
    reader: &mut R,
    dir_path: &Path,
    file_path: &Path,
) -> anyhow::Result<File>
where
    R: Read,
{
    // Destination
    let compressed_tmp_file = tempfile::NamedTempFile::new()?;

    fs::create_dir_all(dir_path).expect("Failed to create objects dir.");

    let mut compressor = ZlibEncoder::new(&compressed_tmp_file, Default::default());
    std::io::copy(reader, &mut compressor)?;
    compressor.finish().expect("Zlib compression failed.");

    // Atomically replace file in object store with tmp file once it's fully written.
    let written_file = compressed_tmp_file.persist(file_path)?;

    Ok(written_file)
}
