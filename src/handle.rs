use std::{
    ffi::{CStr, CString, c_char, c_int},
    fmt::Debug,
    io::{BufReader, ErrorKind, Read, Write},
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::Path,
    sync::{Arc, Mutex},
};

use magic_sys::*;

use crate::{Error, pool::Reservoir};

/// A handle to a single libmagic "cookie", which is better thought of as an instance of the
/// libmagic database.
///
/// Note that handles are [`Send`], but not [`Sync`] — libmagic is not thread safe, but also doesn't
/// use any thread local storage that would prevent magic cookies from moving from one thread to
/// another.
///
/// The exact format of the "textual descriptions" below depends on the flags that were set on the
/// [`crate::Config`]: most notably [`Extension`][`crate::Flag::Extension`],
/// [`Mime`][`crate::Flag::Mime`], [`MimeEncoding`][`crate::Flag::MimeEncoding`],
/// [`MimeType`][`crate::Flag::MimeType`], and [`Continue`][`crate::Flag::Continue`].
pub struct Handle {
    cookie: Option<Cookie>,
    reservoir: Option<Arc<Mutex<Reservoir>>>,
}

impl Handle {
    pub(crate) fn new(cookie: Cookie, reservoir: Option<Arc<Mutex<Reservoir>>>) -> Self {
        Self {
            cookie: Some(cookie),
            reservoir,
        }
    }

    /// Returns a textual description of the given buffer.
    pub fn buffer(&mut self, buf: &[u8]) -> Result<String, Error> {
        description_to_str(
            self.raw(|cookie| unsafe { magic_buffer(cookie, buf.as_ptr(), buf.len()) })?,
        )
    }

    /// Returns a textual description of the given file.
    pub fn file(&mut self, path: impl AsRef<Path>) -> Result<String, Error> {
        let path =
            CString::new(path.as_ref().as_os_str().as_bytes()).map_err(|_| Error::EmbeddedNuls)?;
        description_to_str(self.raw(|cookie| unsafe { magic_file(cookie, path.as_ptr()) })?)
    }

    /// Returns a textual description of the given [`Read`].
    ///
    /// Note that this function has to spawn a thread while reading, so if that isn't desirable,
    /// you should choose another function.
    ///
    /// Also note that this function tends to return simply `data` once libmagic's file size limit
    /// has been hit, regardless of what data is actually in the reader. That limit defaults to
    /// approximately 7 MiB; consider writing larger inputs out to a file and then using
    /// [`Handle::file`].
    pub fn read(&mut self, read: impl Read) -> Result<String, Error> {
        // Our options to handle an arbitrary `Read` are basically either to buffer the entire input
        // or to feed it in via a file descriptor, which means an anonymous pipe. The latter is
        // definitely more efficient, but requires us to spawn a thread to drive the anonymous pipe.
        //
        // Since cookies are `Send`, we'll move the cookie within the handle into another thread,
        // and drive the pipe from this thread, thereby not requiring `read` to be `Send`.

        let (reader, mut writer) = std::io::pipe().map_err(Error::PipeCreate)?;
        let mut cookie = self.cookie.take().ok_or(Error::CookieNommed)?;

        let cookie_handle = std::thread::spawn(move || {
            match cookie.raw(|cookie| unsafe { magic_descriptor(cookie, reader.as_raw_fd()) }) {
                Ok(desc) => (description_to_str(desc), cookie),
                Err(e) => (Err(e), cookie),
            }
        });

        // We can't use std::io::copy here because libmagic will unceremoniously drop the fd once
        // it has enough data or has hit its limit, so we need to handle a broken pipe as a
        // successful termination.
        let mut read = BufReader::new(read);
        let mut buf = vec![0u8; 8192];
        loop {
            let r = read.read(&mut buf).map_err(Error::PipeCopy)?;
            if r == 0 {
                break;
            }

            match writer.write_all(&buf[0..r]) {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                    break;
                }
                Err(e) => return Err(Error::PipeCopy(e)),
            }
        }

        // Drop the writer, just to ensure that our spawned thread terminates.
        drop(writer);

        let (result, cookie) = cookie_handle.join().map_err(|_| Error::PipeJoin)?;
        self.cookie.replace(cookie);

        result
    }

    /// Returns a textual description of the given raw file descriptor.
    pub fn raw_fd(&mut self, fd: impl AsRawFd) -> Result<String, Error> {
        description_to_str(self.raw(|cookie| unsafe { magic_descriptor(cookie, fd.as_raw_fd()) })?)
    }

    /// Allows a raw libmagic function to be invoked on the [`magic_t`] cookie within the handle.
    ///
    /// Normal users should not need to use this, but it's available as an escape hatch if
    /// necessary.
    ///
    /// This crate re-exports [`magic_sys`][crate::magic_sys], so any required functions and
    /// constants are accessible that way.
    pub fn raw<F, R>(&mut self, f: F) -> Result<R, Error>
    where
        F: FnOnce(magic_t) -> R,
        R: ResultType,
    {
        match self.cookie.as_mut() {
            Some(cookie) => cookie.raw(f),
            None => Err(Error::CookieNommed),
        }
    }
}

impl Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("magic", &self.cookie)
            .finish()
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if let Some(cookie) = self.cookie.take()
            && let Some(reservoir) = self.reservoir.take()
        {
            reservoir
                .lock()
                .expect("magic pool inner lock")
                .unused
                .push(cookie);
        }
    }
}

#[derive(Debug)]
pub(crate) struct Cookie(magic_t);

impl Cookie {
    pub(crate) fn raw<F, R>(&mut self, f: F) -> Result<R, Error>
    where
        F: FnOnce(magic_t) -> R,
        R: ResultType,
    {
        let result = f(self.0);
        if result.is_error() {
            let errno = unsafe { magic_errno(self.0) };
            let error = unsafe { magic_error(self.0) };
            if error.is_null() {
                Err(Error::Nested(errno))
            } else {
                Err(Error::Magic {
                    errno,
                    message: unsafe { CStr::from_ptr(error) }.into(),
                })
            }
        } else {
            Ok(result)
        }
    }
}

impl Drop for Cookie {
    fn drop(&mut self) {
        unsafe { magic_close(self.0) };
    }
}

impl TryFrom<magic_t> for Cookie {
    type Error = Error;

    fn try_from(cookie: magic_t) -> Result<Self, Self::Error> {
        if cookie.is_null() {
            Err(Error::create())
        } else {
            Ok(Self(cookie))
        }
    }
}

unsafe impl Send for Cookie {}

/// A raw result from the libmagic C API, which can be either a [`c_int`] or a [`*const
/// c_char`][c_char].
pub trait ResultType {
    fn is_error(&self) -> bool;
}

impl ResultType for c_int {
    fn is_error(&self) -> bool {
        *self == -1
    }
}

impl ResultType for *const c_char {
    fn is_error(&self) -> bool {
        self.is_null()
    }
}

fn description_to_str(desc: *const c_char) -> Result<String, Error> {
    let cstr = unsafe { CStr::from_ptr(desc) };

    match cstr.to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(Error::DescriptionNotUtf8(cstr.to_bytes().to_vec())),
    }
}
