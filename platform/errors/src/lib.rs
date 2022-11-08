//! This package is mostly just a reexport of
//! [`thiserror`](https://github.com/dtolnay/thiserror) and
//! [`anyhow`](https://github.com/dtolnay/anyhow) crates, the de-facto standard
//! for defining and propagating errors, respectively.
//!
//! ### Howto
//!
//! Everything you need to know about idiomatic error handling in Rust has been
//! succintly explained in this [blog post](https://nick.groenen.me/posts/rust-error-handling/).
//!
//! The TL;DR is:
//! - Use `thiserror` to define strongly-typed public errors for your libraries.
//! - Use `eyre` to define loosely-typed errors that are private to your
//!   executables and/or libraries.
//! - Use `eyre` to propagate errors and their (optional) context up the
//!   callstack.
//! - Read the blog post above.
//!
//! This crate doesn't provide examples of using `thiserror` and `eyre`:
//! you'll find plenty in their respective docs.
//! Also, _read the blog post_.
//!
//! ### Extensions
//!
//! This crate comes with a set of traits, `ErrorExt` and `ResultExt`, that
//! transparently extend all types that implement the standard `Error` trait.
//! These extensions allow for turning anything error-like into something
//! error-like _and_ thread-safe.
//!
//! This added thread-safety is a requirement to integrate with the rest of the
//! `eyre`/`thiserror` ecosystem.

pub use eyre::{bail, ensure, Context};

pub type AnyError = eyre::Error;
pub type AnyResult<T> = eyre::Result<T>;

#[cfg(feature = "serde-integration")]
mod serde;
#[cfg(feature = "serde-integration")]
pub use self::serde::SerializableError;

pub mod prelude {
    #[cfg(feature = "serde-integration")]
    pub use super::SerializableError;
    pub use super::{anyerror, AnyError, AnyResult};
    pub use eyre::{bail, ensure, Context as _, ContextCompat as _};
}

// -----------------------------------------------------------------------------

pub mod reexports {
    pub use eyre;
    pub use thiserror;
}

#[macro_export]
macro_rules! anyerror {
    ($msg:literal $(,)?) => {
        $crate::reexports::eyre::eyre!($msg)
    };
    ($err:expr $(,)?) => {{
        $crate::reexports::eyre::eyre!($err)
    }};
    ($fmt:expr, $($arg:tt)*) => {
        $crate::reexports::eyre::eyre!(format!($fmt, $($arg)*))
    };
}
