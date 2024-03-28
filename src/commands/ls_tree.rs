use crate::{objects, Object, TreeEntryMode};
use std::io::{stdout, Write};

pub fn ls_tree(tree_sha: &str, name_only: bool) -> anyhow::Result<()> {
    let (_dir_path, file_path) = objects::paths_from_sha(tree_sha);

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
    };

    Ok(())
}
