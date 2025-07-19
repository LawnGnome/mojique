use mojique::{Config, DefaultConfig};

fn main() -> anyhow::Result<()> {
    let mut handle = DefaultConfig::default().build_handle()?;
    let desc = handle.read(std::io::stdin())?;

    println!("{desc}");

    Ok(())
}
