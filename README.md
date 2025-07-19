# mojique

mojique implements a safe Rust wrapper around [libmagic][libmagic], providing a
`Send`-safe `Handle` into the library, along with an optional `Pool` that can
be used to provide thread-safe access to libmagic.

## Basic usage

It's probably easiest to look [at the documentation][docs], but if you want a
very basic example just to get a taste of the API, here's how to detect the
type of a path given as a command line argument:

```rust
use std::path::PathBuf;

use mojique::{Config, DefaultConfig};

fn main() -> anyhow::Result<()> {
    let path: PathBuf = std::env::args_os()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("no path given as a command line argument"))?
        .into();

    let mut handle = DefaultConfig::default().build_handle()?;
    let desc = handle.file(path)?;

    println!("{desc}");

    Ok(())
}
```

## FAQ

(Nobody's actually asked any questions yet, but here are the questions I expect
to be asked eventually if this sees any real use.)

### Why not `magic`?

The venerable [`magic` crate][magic] has existed for over a decade, and also
provides a safe wrapper around libmagic. I elected to write my own take on this
for a few reasons:

1. I needed to be able to perform magic checks in parallel within a set of
   Tokio tasks, which implied some sort of pool or thread-local architecture.
2. I found the `magic` API a little unwieldy, especially around the hidden
   state types.
3. I wanted `Read` support within the API, which `magic` doesn't currently
   provide.

All that said, if `magic` suits your purposes, you should use `magic`. It's
considerably more battle tested.

### What parts of the libmagic API are not exposed?

For now, there are two major omissions in the API:

1. Support for database manipulation, specifically the `magic_check` and
   `magic_compile` functions. I don't need them, and my suspicion is that the
   average user also doesn't.
2. Support for getting and setting parameters using `magic_getparam` and
   `magic_setparam`, respectively.

If you do need either of these functions, note the `Handle::raw` method, which
gives you direct access to the underlying `magic_t *` in a safe way to use with
[`magic-sys`][magic-sys], which is re-exported from mojique. (Well, until you
actually make an FFI call, at which point you'll have to use `unsafe`.)

### What about async support?

In theory, it should be possible to implement a `Handle::async_read` method
that takes an `AsyncRead` and bridges it into libmagic.

Realistically, though, this is going to be executor-dependent. On Tokio, you'd
want to use `SyncIoBridge` to get in there. So, for now, I think it's best left
as an exercise for the reader.

### What about a pure Rust implementation of the magic algorithm?

This is something Trail of Bits did with [PolyFile][polyfile] for Python. I
definitely understand the impulse.

For now, I don't really have the time to implement this. But never say never.
(And if someone out there does do so, please let me know! I'd love to use it.)

[docs]: https://docs.rs/mojique
[libmagic]: https://www.darwinsys.com/file/
[magic]: https://crates.io/crates/magic
[magic-sys]: https://crates.io/crates/magic-sys
[polyfile]: https://github.com/trailofbits/polyfile
