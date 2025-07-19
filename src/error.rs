use std::{
    ffi::{CStr, CString, c_int},
    fmt::{Debug, Display},
    sync::{MutexGuard, PoisonError},
};

use thiserror::Error;

use crate::pool::Reservoir;

/// Errors that can be returned from mojique.
#[derive(Debug, Error)]
pub enum Error {
    #[error("cookie was previously dropped")]
    CookieNommed,

    #[error("creating magic cookie (): {0}")]
    Create(#[source] std::io::Error),

    #[error("description was not valid UTF-8: {0:?}")]
    DescriptionNotUtf8(Vec<u8>),

    #[error("one or more embedded colons in database path")]
    EmbeddedColons,

    #[error("one or more embedded NUL bytes in database path")]
    EmbeddedNuls,

    #[error("[{errno}] {message}")]
    Magic { errno: c_int, message: Message },

    #[error("libmagic call errored with code {0}; then trying to get error message also errored")]
    Nested(c_int),

    #[error("creating an anonymous pipe")]
    PipeCreate(#[source] std::io::Error),

    #[error("copying data into an anonymous pipe: {0}")]
    PipeCopy(#[source] std::io::Error),

    #[error("waiting for pipe thread")]
    PipeJoin,

    #[error("environment pool lock poisoned")]
    PoolPoisoned,
}

impl Error {
    pub(crate) fn create() -> Self {
        Self::Create(std::io::Error::last_os_error())
    }
}

impl From<PoisonError<MutexGuard<'_, Reservoir>>> for Error {
    fn from(_: PoisonError<MutexGuard<'_, Reservoir>>) -> Self {
        Self::PoolPoisoned
    }
}

#[derive(Clone)]
pub struct Message(CString);

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_tuple("Message");

        if let Ok(s) = self.0.to_str() {
            f.field(&s).finish()
        } else {
            f.field(&self.0).finish()
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(s) = self.0.to_str() {
            write!(f, "{s}")
        } else {
            write!(
                f,
                "[{} byte{}]",
                self.0.count_bytes(),
                if self.0.count_bytes() == 1 { "" } else { "s" }
            )
        }
    }
}

impl From<&CStr> for Message {
    fn from(value: &CStr) -> Self {
        Self(value.to_owned())
    }
}

impl From<CString> for Message {
    fn from(value: CString) -> Self {
        Self(value)
    }
}
