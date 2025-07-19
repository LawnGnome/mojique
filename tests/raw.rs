use std::ffi::c_void;

use magic_sys::{MAGIC_PARAM_BYTES_MAX, magic_getparam};
use mojique::{Config, DefaultConfig};

mod common;

#[test]
fn raw() -> anyhow::Result<()> {
    // Just make sure that a raw call does _something_ useful. We'll go get the file size limit.
    let mut handle = DefaultConfig::default().build_handle()?;
    let mut limit = 0usize;
    handle.raw(|cookie| unsafe {
        magic_getparam(cookie, MAGIC_PARAM_BYTES_MAX, &raw mut limit as *mut c_void)
    })?;
    assert!(limit > 0);

    Ok(())
}
