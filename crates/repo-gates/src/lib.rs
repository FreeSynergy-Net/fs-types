//! Checks of the repository itself: the tests in `tests/` read the repository root (`CHANGELOG.md`, the licence files,
//! the manifests of all members). They are not part of any published `.crate` (`publish = false`): a packed crate has no
//! repository around it, and a test that needs one would be red in every unpacked copy.
