use std::fs;

pub fn init_repo() {
    // TODO: Handle files already existing:
    // file:///Users/chris/.rustup/toolchains/stable-aarch64-apple-darwin/share/doc/rust/html/std/io/enum.ErrorKind.html

    // TODO: Swith to anyhow result, `?` syntax
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();

    println!("Initialized git directory")
}
