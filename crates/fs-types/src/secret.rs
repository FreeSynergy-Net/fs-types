//! `Secret<T>`: a value that cannot be printed, copied or serialised by accident, and is wiped when it is dropped.
//!
//! Pattern: Newtype with a guard: `Debug` prints a placeholder, there is no `Display`, no `Clone`, no `Serialize`, and the
//! value is reachable only through [`Secret::expose_secret`], a call that is easy to find and to forbid (`clippy.toml`,
//! `disallowed-methods`, see `DESIGN.md`). On drop the value is overwritten with zeros (`zeroize`, RustCrypto).
//!
//! It is for credentials, keys and tokens: what must never reach an answer, an error, a log line or an export. It does not
//! hide a value that was copied out before (`expose_secret` gives a reference: do not clone it into a `String` that outlives
//! the call), and a wiped value can still have been moved or swapped to disk before the drop: the wipe is a floor, not a vault.

use std::fmt;

use zeroize::{Zeroize, Zeroizing};

/// A secret value of type `T` (a `String`, a `Vec<u8>`, an array of bytes ...).
///
/// `Debug` hides it, also inside `Option`, `Vec` and `{:#?}`:
///
/// ```
/// let s = fs_types::Secret::new(String::from("hunter2"));
/// assert_eq!(format!("{s:?}"), "Secret(<hidden>)");
/// assert!(!format!("{:#?}", Some(&s)).contains("hunter2"));
/// assert_eq!(s.expose_secret(), "hunter2");
/// ```
///
/// It cannot be cloned (a copy is a second place to wipe, and to leak):
///
/// ```compile_fail
/// let s = fs_types::Secret::new(String::from("x"));
/// let _copy = s.clone();
/// ```
///
/// It cannot be shown with `{}`:
///
/// ```compile_fail
/// let s = fs_types::Secret::new(String::from("x"));
/// let _text = format!("{s}");
/// ```
///
/// It cannot be serialised. The helper compiles for a `String` (so that the bound works), and not for a `Secret`:
///
/// ```
/// fn needs_serialize<T: serde::Serialize>(_: &T) {}
/// needs_serialize(&String::new());
/// ```
///
/// ```compile_fail
/// fn needs_serialize<T: serde::Serialize>(_: &T) {}
/// needs_serialize(&fs_types::Secret::new(String::new()));
/// ```
pub struct Secret<T: Zeroize>(Zeroizing<T>);

impl<T: Zeroize> Secret<T> {
    /// Wrap a value; it is wiped when the `Secret` is dropped.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(Zeroizing::new(value))
    }

    /// The value. Call this only where the secret must really be used (handed to a transport, compared, derived from): it
    /// is the one call that a lint or a text search can find.
    #[must_use]
    pub fn expose_secret(&self) -> &T {
        &self.0
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<hidden>)")
    }
}

impl<T: Zeroize> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use zeroize::Zeroize;

    use super::Secret;

    /// A value that says when it was wiped.
    struct Probe(Rc<Cell<u32>>);

    impl Zeroize for Probe {
        fn zeroize(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn the_value_is_wiped_once_when_the_secret_is_dropped_and_not_before() {
        let wiped = Rc::new(Cell::new(0));
        let s = Secret::new(Probe(wiped.clone()));
        assert_eq!(wiped.get(), 0);
        let _ = s.expose_secret();
        assert_eq!(wiped.get(), 0, "reading does not wipe");
        drop(s);
        assert_eq!(wiped.get(), 1);
    }

    #[test]
    fn debug_never_shows_the_value_in_any_container_or_form() {
        let s = Secret::new(String::from("hunter2-very-secret"));
        let all = [
            format!("{s:?}"),
            format!("{s:#?}"),
            format!("{:?}", Some(&s)),
            format!("{:#?}", vec![&s, &s]),
            format!("{:?}", (&s, 1)),
        ];
        for text in all {
            assert!(!text.contains("hunter2"), "{text}");
            assert!(text.contains("<hidden>"), "{text}");
        }
        let bytes = Secret::new(vec![0x68_u8, 0x75, 0x6e]);
        assert!(!format!("{bytes:?}").contains("104") && !format!("{bytes:?}").contains("68"));
    }

    #[test]
    fn the_value_is_read_only_through_expose_secret() {
        let s: Secret<String> = String::from("abc").into();
        assert_eq!(s.expose_secret(), "abc");
        let b = Secret::new([1_u8, 2, 3]);
        assert_eq!(b.expose_secret(), &[1, 2, 3]);
    }
}
