//! The codes of the suite (prefix `records-suite-error-`), on fs-conformance's own code type (one type after the lift, L1).
//!
//! Pattern: none, an enum with total `match`es; append only.

use fs_conformance::ErrorCode;

/// What a rule found wrong.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SuiteCode {
    /// A call the rule needed failed (parameter `got` = the provider's message key).
    CallFailed,
    /// Paging lost, repeated or over-filled records (parameters `expected`, `got`).
    PageWrong,
    /// A call answered another code than the rule demands (parameters `expected`, `got`; `got = ok` for no code).
    CodeWrong,
    /// A condition was not applied (parameters `expected`, `got`).
    FilterWrong,
    /// Ties were not broken by id (parameters `expected`, `got`).
    OrderWrong,
    /// A hidden field carried a value, or was not listed as hidden (parameter `path`).
    HiddenLeak,
    /// A locked record came without its lock (parameter `id`).
    LockMissing,
    /// A grouped record had no group or one outside the set (parameters `id`, `key`).
    GroupOutside,
    /// `get` of a missing record was not `None` (parameter `got`).
    MissingWrong,
    /// The generation did not move after a write.
    GenerationStill,
    /// The shape names a field the type does not have (parameter `path`).
    ShapeWrong,
    /// A record the person may not read was answered (parameter `id`).
    UnreadableShown,
}

impl ErrorCode for SuiteCode {
    const PREFIX: &'static str = "records-suite-error-";
    const ALL: &'static [Self] = &[
        Self::CallFailed,
        Self::PageWrong,
        Self::CodeWrong,
        Self::FilterWrong,
        Self::OrderWrong,
        Self::HiddenLeak,
        Self::LockMissing,
        Self::GroupOutside,
        Self::MissingWrong,
        Self::GenerationStill,
        Self::ShapeWrong,
        Self::UnreadableShown,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::CallFailed => "call-failed",
            Self::PageWrong => "page-wrong",
            Self::CodeWrong => "code-wrong",
            Self::FilterWrong => "filter-wrong",
            Self::OrderWrong => "order-wrong",
            Self::HiddenLeak => "hidden-leak",
            Self::LockMissing => "lock-missing",
            Self::GroupOutside => "group-outside",
            Self::MissingWrong => "missing-wrong",
            Self::GenerationStill => "generation-still",
            Self::ShapeWrong => "shape-wrong",
            Self::UnreadableShown => "unreadable-shown",
        }
    }
}

/// The parameter keys of [`SuiteCode`].
pub mod keys {
    /// What the rule expected.
    pub const EXPECTED: &str = "expected";
    /// What it got.
    pub const GOT: &str = "got";
    /// A record id.
    pub const ID: &str = "id";
    /// A field path.
    pub const PATH: &str = "path";
    /// A group key.
    pub const KEY: &str = "key";
}
