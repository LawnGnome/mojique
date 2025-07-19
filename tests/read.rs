use std::fs::File;

use common::*;
use insta::assert_snapshot;
use mojique::{Config, DefaultConfig};

mod common;

#[test]
fn read() -> anyhow::Result<()> {
    // The file tests cover a broader range of flags. We mostly just want to ensure that the basic
    // functionality works, really.

    let mut handle = DefaultConfig::default().build_handle()?;

    let file = File::open(manifest_dir().join("LICENSE"))?;
    let magic_type = handle.read(file)?;
    assert_snapshot!(magic_type, @"ASCII text");

    // Let's also ensure that an empty reader is handled correctly.
    let magic_type = handle.read(std::io::empty())?;
    assert_snapshot!(magic_type, @"empty");

    // And we also need to ensure that we handle libmagic dropping a pipe once it has hit its read
    // limit correctly.
    let magic_type = handle.read(std::io::repeat(0))?;
    assert_snapshot!(magic_type, @"data");

    Ok(())
}
