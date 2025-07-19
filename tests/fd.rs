use std::fs::File;

use common::*;
use insta::assert_snapshot;
use mojique::{Config, DefaultConfig};

mod common;

#[test]
fn fd() -> anyhow::Result<()> {
    // The file tests cover a broader range of flags. We mostly just want to ensure that the basic
    // functionality works, really.

    let file = File::open(manifest_dir().join("LICENSE"))?;

    let magic_type = DefaultConfig::default().build_handle()?.raw_fd(file)?;
    assert_snapshot!(magic_type, @"ASCII text");

    Ok(())
}
