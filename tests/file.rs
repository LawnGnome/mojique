use std::collections::BTreeSet;

use common::*;
use insta::{assert_debug_snapshot, assert_snapshot};
use itertools::Itertools;
use mojique::{Config, DefaultConfig, Flag};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod common;

#[test]
fn file() -> anyhow::Result<()> {
    let path = manifest_dir().join("tests/data/LICENSE.zst");

    // We won't try to enumerate every possible combination of flags, but let's ensure that at least
    // the obvious, common ones are handled.

    let mut handle = DefaultConfig::default().build_handle()?;
    let magic_type = handle.file(&path)?;
    assert_snapshot!(magic_type, @"Zstandard compressed data (v0.8+), Dictionary ID: None");

    let mut handle = DefaultConfig::default()
        .set_flag(Flag::Compress)
        .build_handle()?;
    let magic_type = handle.file(&path)?;
    assert_snapshot!(magic_type, @"ASCII text (Zstandard compressed data (v0.8+), Dictionary ID: None)");

    let mut handle = DefaultConfig::default()
        .set_flag(Flag::Compress)
        .set_flag(Flag::CompressTransparent)
        .build_handle()?;
    let magic_type = handle.file(&path)?;
    assert_snapshot!(magic_type, @"ASCII text");

    let mut handle = DefaultConfig::default()
        .set_flag(Flag::Mime)
        .build_handle()?;
    let magic_type = handle.file(&path)?;
    assert_snapshot!(magic_type, @"application/zstd; charset=binary");

    let mut handle = DefaultConfig::default()
        .set_flag(Flag::Compress)
        .set_flag(Flag::Mime)
        .build_handle()?;
    let magic_type = handle.file(&path)?;
    assert_snapshot!(magic_type, @"text/plain; charset=us-ascii compressed-encoding=application/zstd; charset=binary");

    let mut handle = DefaultConfig::default()
        .set_flag(Flag::Extension)
        .build_handle()?;
    let magic_type = handle.file(&path)?;
    assert_snapshot!(magic_type, @"zst");

    let mut handle = DefaultConfig::default()
        .set_flag(Flag::Compress)
        .set_flag(Flag::Extension)
        .build_handle()?;
    let magic_type = handle.file(path)?;
    assert_snapshot!(magic_type, @"??? (zst)");
    Ok(())
}

#[test]
fn not_exist() -> anyhow::Result<()> {
    let mut handle = DefaultConfig::default().build_handle()?;
    let e = handle
        .file("this-file-should-not-exist")
        .expect_err("file not found");
    assert_debug_snapshot!(e, @r#"
    Magic {
        errno: 2,
        message: Message(
            "cannot stat `this-file-should-not-exist' (No such file or directory)",
        ),
    }
    "#);

    Ok(())
}

#[test]
fn symlink() -> anyhow::Result<()> {
    let path = manifest_dir().join("tests/data/symlink");

    // Don't follow the link.
    let mut handle = DefaultConfig::default().build_handle()?;
    let magic_type = handle.file(&path)?;
    assert_snapshot!(magic_type, @"symbolic link to LICENSE.zst");

    // Follow the link.
    let mut handle = DefaultConfig::default()
        .set_flag(Flag::Symlink)
        .build_handle()?;
    let magic_type = handle.file(&path)?;
    assert_snapshot!(magic_type, @"Zstandard compressed data (v0.8+), Dictionary ID: None");

    // Follow the link and decompress.
    let mut handle = DefaultConfig::default()
        .set_flag(Flag::Compress)
        .set_flag(Flag::Symlink)
        .build_handle()?;
    let magic_type = handle.file(path)?;
    assert_snapshot!(magic_type, @"ASCII text (Zstandard compressed data (v0.8+), Dictionary ID: None)");

    Ok(())
}

#[test]
fn parallel() -> anyhow::Result<()> {
    const ITERATIONS: usize = 1000;

    let pool = DefaultConfig::default().build_pool()?;

    // Basically, we'll use rayon to create a pool of shared handles, one per thread, and then make
    // sure that every iteration returned the same value.
    let types: BTreeSet<String> = (0..ITERATIONS)
        .map(|_| manifest_dir().join("LICENSE"))
        .collect_vec()
        .into_par_iter()
        .map(|path| -> Result<String, mojique::Error> { pool.handle()?.file(path) })
        .collect::<Vec<_>>()
        .into_iter()
        .try_collect()?;

    assert_debug_snapshot!(types, @r#"
    {
        "ASCII text",
    }
    "#);

    Ok(())
}
