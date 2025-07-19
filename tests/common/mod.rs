#![allow(dead_code)]

use std::{path::Path, sync::LazyLock};

static MANIFEST_DIR: LazyLock<&'static Path> =
    LazyLock::new(|| Path::new(env!("CARGO_MANIFEST_DIR")));

pub(crate) fn manifest_dir() -> &'static Path {
    &MANIFEST_DIR
}
