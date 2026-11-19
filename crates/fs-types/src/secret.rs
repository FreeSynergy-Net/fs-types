//! `Secret<T>`: a value that never shows itself.
//!
//! Pattern: Value object (wrapper, Newtype). `Debug` and `Display` print a fixed marker, never the value; there is no
//! `Serialize`, no `Hash`, no `PartialEq` and no `AsRef`/`Deref` to the inside: the one way to the value is the named
//! call [`Secret::expose`] (or [`Secret::into_exposed`]), so every use is visible in a search. Comparing is
//! [`Secret::ct_eq`] (constant time for bytes). The value is NOT wiped on drop: the crate has no dependency for that
//! (`zeroize` is not allowed here yet); a program that needs wiping wraps its own type in `Secret`.

use std::fmt;

/// What `Debug` and `Display` print instead of the value.
const HIDDEN: &str = "<hidden>";

/// A secret value (password, token, key bytes).
#[derive(Clone)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Wrap a value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Look at the value: the deliberate way out.
    #[must_use]
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Take the value out: the deliberate way out, by value.
    #[must_use]
    pub fn into_exposed(self) -> T {
        self.0
    }

    /// Change the value; the result is a secret again.
    #[must_use]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Secret<U> {
        Secret(f(self.0))
    }
}

impl<T: AsRef<[u8]>> Secret<T> {
    /// Equal bytes, in time that depends only on the lengths, not on where the first difference is.
    #[must_use]
    pub fn ct_eq(&self, other: &Self) -> bool {
        let (a, b) = (self.0.as_ref(), other.0.as_ref());
        if a.len() != b.len() {
            return false;
        }
        a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret({HIDDEN})")
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(HIDDEN)
    }
}
