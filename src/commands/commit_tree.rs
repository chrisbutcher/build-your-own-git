use std::{fmt::Write, io::Cursor};

use crate::objects;

pub fn commit_tree(
    tree_sha: String,
    parent_sha: Option<String>,
    message: String,
) -> anyhow::Result<String> {
    // ```
    // tree {tree_sha}
    // parent {parent_sha} (for each parent commit)
    // author {author_name} <{author_email}> {author_date_seconds} {author_date_timezone}
    // committer {committer_name} <{committer_email}> {committer_date_seconds} {committer_date_timezone}

    // {commit message}
    // ```

    let mut commit_contents = String::new();

    writeln!(&mut commit_contents, "tree {tree_sha}")?;

    if let Some(parent_sha) = parent_sha {
        eprintln!("parent_sha: {parent_sha}");
        writeln!(&mut commit_contents, "parent {parent_sha}")?;
    }

    writeln!(
        &mut commit_contents,
        "author Bob Smith <b.smith@gmail.com> 1243040974 -0700"
    )?;
    writeln!(
        &mut commit_contents,
        "committer Bob Smith <b.smith@gmail.com> 1243040974 -0700"
    )?;

    writeln!(&mut commit_contents)?;

    writeln!(&mut commit_contents, "{message}")?;

    let content_size = commit_contents.len();
    let mut c = Cursor::new(commit_contents);
    let (_file, commit_hash) = objects::build_hashed_file(&mut c, "commit", content_size, true)?;

    Ok(commit_hash)
}
