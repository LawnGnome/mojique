#![allow(private_interfaces)]

use std::{
    ffi::{CString, c_int},
    path::PathBuf,
};

use crate::{
    Error, Handle,
    config::private::ConfigPrivateExt,
    ffi::Flag,
    pool::{Pool, Source},
};

/// A configuration that sets libmagic flags on any created [`Handle`] instances.
///
/// By default, the only flag that is set is [`Flag::Error`].
pub trait Config: ConfigPrivateExt + Sized {
    /// Builds a single [`Handle`] from the configuration.
    fn build_handle(self) -> Result<Handle, Error> {
        let flags = self.flags();
        self.into_source()?.create_handle(flags, None)
    }

    /// Builds a [`Pool`] of handles from the configuration.
    fn build_pool(self) -> Result<Pool, Error> {
        Pool::new(self.flags(), self.into_source()?)
    }

    /// Removes a flag from the configuration.
    fn remove_flag(self, flag: Flag) -> Self;

    /// Sets a flag on the configuration.
    fn set_flag(self, flag: Flag) -> Self;
}

pub(crate) mod private {
    use super::*;

    pub trait ConfigPrivateExt {
        fn flags(&self) -> c_int;
        fn into_source(self) -> Result<Source, Error>;
    }
}

/// A configuration using the default magic database installed on the system.
#[derive(Debug, Clone)]
pub struct DefaultConfig {
    flags: c_int,
}

impl DefaultConfig {
    fn _remove_flag(&mut self, flag: Flag) {
        self.flags &= !(flag as c_int);
    }

    fn _set_flag(&mut self, flag: Flag) {
        self.flags |= flag as c_int;
    }
}

impl Config for DefaultConfig {
    fn remove_flag(mut self, flag: Flag) -> Self {
        self._remove_flag(flag);
        self
    }

    fn set_flag(mut self, flag: Flag) -> Self {
        self._set_flag(flag);
        self
    }
}

impl ConfigPrivateExt for DefaultConfig {
    fn flags(&self) -> c_int {
        self.flags
    }

    fn into_source(self) -> Result<Source, Error> {
        Ok(Source::Default)
    }
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            flags: Flag::Error as c_int,
        }
    }
}

/// A configuration using one or more magic databases provided as `[u8]` buffers.
#[derive(Debug, Clone, Default)]
pub struct BufferConfig {
    config: DefaultConfig,
    buffers: Vec<Vec<u8>>,
}

impl BufferConfig {
    pub fn with_buffer(mut self, buffer: &[u8]) -> Self {
        self.buffers.push(buffer.to_vec());
        self
    }
}

impl Config for BufferConfig {
    fn remove_flag(mut self, flag: Flag) -> Self {
        self.config._remove_flag(flag);
        self
    }

    fn set_flag(mut self, flag: Flag) -> Self {
        self.config._set_flag(flag);
        self
    }
}

impl ConfigPrivateExt for BufferConfig {
    fn flags(&self) -> c_int {
        self.config.flags
    }

    fn into_source(self) -> Result<Source, Error> {
        Ok(Source::Buffers(self.buffers.into()))
    }
}

/// A configuration using one or more magic databases on the filesystem, but not including the
/// default database.
#[derive(Debug, Clone, Default)]
pub struct FileConfig {
    config: DefaultConfig,
    paths: Vec<PathBuf>,
}

impl FileConfig {
    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.paths.push(path.into());
        self
    }
}

impl Config for FileConfig {
    fn remove_flag(mut self, flag: Flag) -> Self {
        self.config._remove_flag(flag);
        self
    }

    fn set_flag(mut self, flag: Flag) -> Self {
        self.config._set_flag(flag);
        self
    }
}

impl ConfigPrivateExt for FileConfig {
    fn flags(&self) -> c_int {
        self.config.flags
    }

    fn into_source(self) -> Result<Source, Error> {
        // libmagic only accepts a colon-separated set of paths, so we have to take our Rust
        // PathBufs and turn them into that. An obvious corollary here is that no path can include
        // a colon, which will probably make Windows support spicy.
        self.paths
            .into_iter()
            .try_fold(Vec::new(), |mut acc, path| {
                if !acc.is_empty() {
                    acc.push(b':');
                }

                let bytes = path.into_os_string().into_encoded_bytes();
                if bytes.contains(&b':') {
                    Err(Error::EmbeddedColons)
                } else {
                    acc.extend(bytes);
                    Ok(acc)
                }
            })
            .map(|bytes| {
                CString::new(bytes)
                    .map(Source::Files)
                    .map_err(|_| Error::EmbeddedNuls)
            })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags() {
        // Most testing happens in integration tests, but let's just exercise the flag handling
        // here.

        let config = DefaultConfig::default();
        assert_eq!(config.flags, Flag::Error as c_int);

        let builder = config.set_flag(Flag::Apple).set_flag(Flag::Check);
        assert_eq!(
            builder.flags,
            (Flag::Error as c_int) | (Flag::Apple as c_int) | (Flag::Check as c_int)
        );

        let builder = builder.remove_flag(Flag::Error);
        assert_eq!(
            builder.flags,
            (Flag::Apple as c_int) | (Flag::Check as c_int)
        );
    }
}
