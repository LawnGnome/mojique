use std::path::PathBuf;

use mojique::{Config, DefaultConfig};

fn main() -> anyhow::Result<()> {
    let path: PathBuf = std::env::args_os()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("no path given as a command line argument"))?
        .into();

    let mut handle = DefaultConfig::default().build_handle()?;
    let desc = handle.file(path)?;

    println!("{desc}");

    Ok(())
}
