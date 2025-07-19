use std::ffi::c_int;

use magic_sys::*;
use static_assertions::assert_eq_size;

// XXX: this will break if sizeof(int) != 4, since we can't repr(c_int)
//
// Refreshing my memory with Wikipedia, it seems that the only non-embedded platforms with
// sizeof(int) == 8 are a specific SPARC64 port of Solaris and UNICOS. I'm probably willing to roll
// the dice on just not supporting those. (As for embedded, well, practically libmagic probably
// isn't going to be in use, but PRs welcome if there's a reasonable use case and this can be cfg'd
// away somehow.)
//
// So, for now, let's use the nifty static_assertions crate to at least fail to build if this isn't
// true.
assert_eq_size!(c_int, i32);

/// libmagic flags.
///
/// The flag descriptions below are reproduced directly from the `libmagic(3)` man page, which is
/// the authoritative source of any behavioural information.
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum Flag {
    /// Print debugging messages to stderr.
    Debug = MAGIC_DEBUG,

    /// If the file queried is a symlink, follow it.
    Symlink = MAGIC_SYMLINK,

    /// If the file is compressed, unpack it and look at the contents.
    Compress = MAGIC_COMPRESS,

    /// If the file is a block or character special device, then open the device and try to look in
    /// its contents.
    Devices = MAGIC_DEVICES,

    /// Return a MIME type string, instead of a textual description.
    MimeType = MAGIC_MIME_TYPE,

    /// Return a MIME encoding, instead of a textual description.
    MimeEncoding = MAGIC_MIME_ENCODING,

    /// A shorthand for [`Flag::MimeType`] | [`Flag::MimeEncoding`].
    Mime = MAGIC_MIME,

    /// Return all matches, not just the first.
    Continue = MAGIC_CONTINUE,

    /// Check the magic database for consistency and print warnings to stderr.
    Check = MAGIC_CHECK,

    /// On systems that support `utime(3)` or `utimes(2)`, attempt to preserve the access time of
    /// files analysed.
    PreserveAccessTime = MAGIC_PRESERVE_ATIME,

    /// Don't translate unprintable characters to a `\ooo` octal representation.
    Raw = MAGIC_RAW,

    /// Treat operating system errors while trying to open files and follow symlinks as real
    /// errors, instead of printing them in the magic buffer.
    Error = MAGIC_ERROR,

    /// Return the Apple creator and type.
    Apple = MAGIC_APPLE,

    /// Return a slash-separated list of extensions for this file type.
    #[cfg(feature = "v5-23")]
    Extension = MAGIC_EXTENSION,

    /// Don't report on compression, only report about the uncompressed data.
    #[cfg(feature = "v5-23")]
    CompressTransparent = MAGIC_COMPRESS_TRANSP,

    /// Don't check for `EMX` application type (only on EMX).
    NoCheckAppType = MAGIC_NO_CHECK_APPTYPE,

    /// Don't get extra information on MS Composite Document Files.
    NoCheckCDF = MAGIC_NO_CHECK_CDF,

    /// Don't look inside compressed files.
    NoCheckCompress = MAGIC_NO_CHECK_COMPRESS,

    /// Don't print ELF details.
    NoCheckELF = MAGIC_NO_CHECK_ELF,

    /// Don't check text encodings.
    NoCheckEncoding = MAGIC_NO_CHECK_ENCODING,

    /// Don't consult magic files.
    NoCheckSoft = MAGIC_NO_CHECK_SOFT,

    /// Don't examine tar files.
    NoCheckTar = MAGIC_NO_CHECK_TAR,

    /// Don't check for various types of text files.
    NoCheckText = MAGIC_NO_CHECK_TEXT,

    /// Don't look for known tokens inside ascii files.
    NoCheckTokens = MAGIC_NO_CHECK_TOKENS,

    /// Don't examine JSON files.
    #[cfg(feature = "v5-35")]
    NoCheckJSON = MAGIC_NO_CHECK_JSON,

    /// Don't examine CSV files.
    #[cfg(feature = "v5-38")]
    NoCheckCSV = MAGIC_NO_CHECK_CSV,
}
