//! mojique provides a simple interface into [libmagic][libmagic] (using [`magic_sys`] underneath),
//! along with an optional [`Send`], [`Sync`] pool that makes it easier to use libmagic in
//! multi-threaded and async environments.
//!
//! To use mojique, you need to:
//!
//! 1. Create a [`Config`], which is one of:
//!    1. [`DefaultConfig`]: uses the system magic database.
//!    1. [`BufferConfig`]: uses magic database(s) provided from `&[u8]` buffers.
//!    1. [`FileConfig`]: uses magic database(s) on the filesystem.
//! 1. Build either a single [`Handle`] (which is [`Send`], but not [`Sync`]), or a [`Pool`] of
//!    handles (that is both [`Send`] and [`Sync`]), which can then be used to acquire handles via
//!    [`Pool::handle`].
//! 1. Call methods on [`Handle`] to detect file types based on content.
//!
//! ## Simple example
//!
//! A simple example that uses the system magic database to get a MIME type:
//!
//! ```
//! use mojique::{Config, DefaultConfig, Flag};
//!
//! let mut handle = DefaultConfig::default().set_flag(Flag::Mime).build_handle()?;
//! let mime_type = handle.buffer(b"#include <stdio.h>")?;
//!
//! println!("MIME type of something looking like C is: {mime_type}");
//! # anyhow::Ok(())
//! ```
//!
//! ## Pools
//!
//! Building a pool uses the same configuration:
//!
//! ```
//! use mojique::{Config, DefaultConfig, Flag};
//!
//! let pool = DefaultConfig::default().set_flag(Flag::Mime).build_pool()?;
//! # anyhow::Ok(())
//! ```
//!
//! Once you have a pool, you can [`Clone`] it as much as needed and use [`Pool::handle`] to
//! acquire handles to specific tasks or threads.
//!
//! [libmagic]: https://www.darwinsys.com/file/

pub use magic_sys;
use std::ffi::c_int;

pub use crate::{
    config::{BufferConfig, Config, DefaultConfig, FileConfig},
    error::Error,
    ffi::Flag,
    handle::{Handle, ResultType},
    pool::Pool,
};

mod config;
mod error;
mod ffi;
mod handle;
mod pool;

/// Returns the libmagic version.
pub fn version() -> c_int {
    unsafe { magic_sys::magic_version() }
}

#[cfg(test)]
mod tests {
    #[test]
    fn version() {
        assert!(super::version() != 0)
    }
}
