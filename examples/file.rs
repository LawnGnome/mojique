use std::path::PathBuf;

use clap::Parser;
use mojique::{Config, DefaultConfig, Flag};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser)]
struct Opt {
    #[arg(short, long)]
    compressed: bool,

    #[arg(short, long)]
    mime: bool,

    #[arg(required=true, num_args=1..)]
    paths: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let Opt {
        compressed,
        mime,
        paths,
    } = Opt::parse();

    let mut config = DefaultConfig::default();
    if compressed {
        config = config.set_flag(Flag::Compress);
    }
    if mime {
        config = config.set_flag(Flag::Mime);
    }
    let pool = config.build_pool()?;

    // Let's parallelise for fun, since we have a thread-safe pool available.
    for res in paths
        .into_par_iter()
        .map(|path| anyhow::Ok((path.clone(), pool.handle()?.file(path)?)))
        .collect::<Vec<_>>()
        .into_iter()
    {
        let (path, desc) = res?;
        println!("{}: {desc}", path.display());
    }

    Ok(())
}
