use std::{
    ffi::{CString, c_int, c_void},
    fmt::Debug,
    sync::{Arc, Mutex},
};

use magic_sys::*;

use crate::{
    Error,
    handle::{Cookie, Handle},
};

/// A thread-safe pool of [`Handle`] instances.
///
/// Pools can be cloned as needed, as the underlying handle storage is shared. Pools will try not
/// to instantiate new handles unless one is actually needed, since the initialisation cost may be
/// non-trivial depending on the magic database(s) in use.
#[derive(Clone)]
pub struct Pool(Arc<Inner>);

struct Inner {
    flags: c_int,
    source: Source,

    // This is about the stupidest possible way of implementing a free pool of handles, but it does
    // work. We'll keep a reference to the reservoir in each handle, and then hand the cookie
    // within the handle back to the reservoir on Drop.
    reservoir: Arc<Mutex<Reservoir>>,
}

impl Pool {
    pub(crate) fn new(flags: c_int, source: Source) -> Result<Self, Error> {
        Ok(Self(Arc::new(Inner {
            flags,
            source,
            reservoir: Default::default(),
        })))
    }

    /// Returns a [`Handle`], instantiating a new one if necessary.
    ///
    /// Users of async runtimes may want to consider running this on a blocking task, as loading
    /// and parsing a database — especially from disk — may cause significant blocking.
    pub fn handle(&self) -> Result<Handle, Error> {
        let mut reservoir = self.0.reservoir.lock()?;

        if let Some(cookie) = reservoir.unused.pop() {
            Ok(Handle::new(cookie, Some(self.0.reservoir.clone())))
        } else {
            // We don't need to hold the lock while we create a handle.
            drop(reservoir);

            self.0
                .source
                .create_handle(self.0.flags, Some(self.0.reservoir.clone()))
        }
    }
}

impl Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pool")
            .field("flags", &self.0.flags)
            .field("source", &self.0.source)
            .finish()
    }
}

#[derive(Default)]
pub(crate) struct Reservoir {
    pub(crate) unused: Vec<Cookie>,
}

#[derive(Debug)]
pub(crate) enum Source {
    Default,
    Buffers(Buffers),
    Files(CString),
}

impl Source {
    pub(crate) fn create_handle(
        &self,
        flags: c_int,
        reservoir: Option<Arc<Mutex<Reservoir>>>,
    ) -> Result<Handle, Error> {
        let mut cookie = Cookie::try_from(unsafe { magic_open(flags) })?;

        match &self {
            Source::Buffers(buffers) => {
                cookie.raw(|cookie| unsafe {
                    magic_load_buffers(cookie, buffers.buffers(), buffers.sizes(), buffers.len())
                })?;
            }
            Source::Files(filename) => {
                cookie.raw(|cookie| unsafe { magic_load(cookie, filename.as_ptr()) })?;
            }
            Source::Default => {
                cookie.raw(|cookie| unsafe { magic_load(cookie, std::ptr::null()) })?;
            }
        }

        Ok(Handle::new(cookie, reservoir))
    }
}

pub(crate) struct Buffers {
    storage: Vec<Vec<u8>>,

    buffers: Vec<*const c_void>,
    sizes: Vec<usize>,
}

impl Debug for Buffers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffers")
            .field("num", &self.storage.len())
            .finish()
    }
}

// These impls are based on reviewing the libmagic code — while it takes mutable pointers when
// loading from buffers, in practice it doesn't appear to modify them, so we should be OK to share
// them between multiple cookies.
unsafe impl Send for Buffers {}
unsafe impl Sync for Buffers {}

impl Buffers {
    fn len(&self) -> usize {
        self.storage.len()
    }

    fn buffers(&self) -> *mut *mut c_void {
        self.buffers.as_ptr() as *mut *mut c_void
    }

    fn sizes(&self) -> *mut usize {
        self.sizes.as_ptr() as *mut usize
    }
}

impl From<Vec<Vec<u8>>> for Buffers {
    fn from(value: Vec<Vec<u8>>) -> Self {
        let sizes = value.iter().map(|buf| buf.len()).collect();
        let buffers = value
            .iter()
            .map(|buf| buf.as_ptr() as *const c_void)
            .collect();

        Self {
            storage: value,
            buffers,
            sizes,
        }
    }
}
