use insta::assert_snapshot;
use mojique::{Config, DefaultConfig};

#[test]
fn buffer() -> anyhow::Result<()> {
    // The file tests cover a broader range of flags. We mostly just want to ensure that the basic
    // functionality works, really.

    let mut handle = DefaultConfig::default().build_handle()?;
    let magic_type = handle.buffer(b"#include <stdio.h>")?;
    assert_snapshot!(magic_type, @"C source, ASCII text, with no line terminators");

    let magic_type = handle.buffer(b"")?;
    assert_snapshot!(magic_type, @"empty");

    Ok(())
}
