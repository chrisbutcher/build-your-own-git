use std::{fs, io::stdout, io::Write};

pub fn init_repo() -> anyhow::Result<()> {
    // TODO: Handle files already existing:
    // file:///Users/chris/.rustup/toolchains/stable-aarch64-apple-darwin/share/doc/rust/html/std/io/enum.ErrorKind.html

    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;

    println!("Initialized git directory");

    Ok(())
}
