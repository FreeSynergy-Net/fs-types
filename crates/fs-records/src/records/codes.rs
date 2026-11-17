//! The codes of `Records`, with their parameter keys. Message key: `records-error-` and the name.
//!
//! On the wire a provider answers them as `Problem` (common.capnp, RFC 9457 shape) with the code name in `type` and the
//! parameters as extensions; this crate only knows the enum (draft §4.6).
//!
//! Pattern: none, an enum with total `match`es. Never renamed or reused; append to the enum AND to `ALL` (K-case K1).

use crate::coded::ErrorCode;

/// What went wrong while reading records, without a sentence.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecordsCode {
    /// The provider owns no listable type of this name (parameter `type`).
    UnknownType,
    /// A condition, sort, projection or `get` names a field the type does not have (parameters `type`, `path`).
    UnknownField,
    /// The query names a grouping the shape does not have (parameters `type`, `grouping`).
    UnknownGrouping,
    /// The query sorts by an order the shape does not have (parameters `type`, `order`).
    UnknownOrder,
    /// A condition value does not fit the field (a choice key not in its list; parameters `path`, `basic`).
    ValueMismatch,
    /// More conditions, sorts or projected fields than allowed (parameters `what`, `max`).
    TooMany,
    /// The page size is 0 or above the limit (parameters `size`, `max`).
    PageSize,
    /// A value is longer than allowed (parameters `path`, `bytes`, `max`).
    ValueTooLarge,
    /// The call took longer than allowed (parameter `ms`).
    Timeout,
    /// The cursor was not made by this provider for this query.
    CursorInvalid,
    /// The provider cannot be reached or failed (parameter `cause` = its own code key).
    Unavailable,
    /// The provider answered a value for a field it lists as hidden (parameter `path`): the host drops the page.
    HiddenHasValue,
    /// The provider put a record into a group that is not in the group set (parameter `key`).
    GroupUnknown,
    /// The provider answered more records than asked (parameters `size`, `got`).
    PageTooLong,
    /// The provider cannot do this kind of condition (parameter `what`); provider rule of ADR 0045 made visible.
    Unsupported,
    /// Nachzug 17.11.: the filter tree is deeper than allowed (parameters `depth`, `max`).
    FilterTooDeep,
    /// Nachzug 17.11.: the filter tree has a wrong form (parameter `what` = `empty-operator`).
    FilterInvalid,
    /// Nachzug 17.11.: a query gives both `position` and a cursor (`after`); JMAP takes one start only.
    PositionWithCursor,
    /// Nachzug (Suche-Arten): the asked search kind is not allowed for this type (parameter `kind`).
    SearchKindRefused,
    /// Nachzug: a search policy without its default, or with a kind twice (parameter `kind`).
    SearchPolicyInvalid,
    /// Nachzug: `changes` cannot say what changed since this state (JMAP `cannotCalculateChanges`): query again.
    CannotCalculateChanges,
    /// Nachzug 2: a page mixes exact and estimated totals, or names an estimate kind without an estimate (or the reverse).
    EstimateMixed,
    /// A limit setting is 0 or above the instance's ceiling (parameters `what`, `max`); added by the probe (O7).
    LimitRefused,
}

impl ErrorCode for RecordsCode {
    const PREFIX: &'static str = "records-error-";
    const ALL: &'static [Self] = &[
        Self::UnknownType,
        Self::UnknownField,
        Self::UnknownGrouping,
        Self::UnknownOrder,
        Self::ValueMismatch,
        Self::TooMany,
        Self::PageSize,
        Self::ValueTooLarge,
        Self::Timeout,
        Self::CursorInvalid,
        Self::Unavailable,
        Self::HiddenHasValue,
        Self::GroupUnknown,
        Self::PageTooLong,
        Self::Unsupported,
        Self::FilterTooDeep,
        Self::FilterInvalid,
        Self::PositionWithCursor,
        Self::SearchKindRefused,
        Self::SearchPolicyInvalid,
        Self::CannotCalculateChanges,
        Self::EstimateMixed,
        Self::LimitRefused,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::UnknownType => "unknown-type",
            Self::UnknownField => "unknown-field",
            Self::UnknownGrouping => "unknown-grouping",
            Self::UnknownOrder => "unknown-order",
            Self::ValueMismatch => "value-mismatch",
            Self::TooMany => "too-many",
            Self::PageSize => "page-size",
            Self::ValueTooLarge => "value-too-large",
            Self::Timeout => "timeout",
            Self::CursorInvalid => "cursor-invalid",
            Self::Unavailable => "unavailable",
            Self::HiddenHasValue => "hidden-has-value",
            Self::GroupUnknown => "group-unknown",
            Self::PageTooLong => "page-too-long",
            Self::Unsupported => "unsupported",
            Self::FilterTooDeep => "filter-too-deep",
            Self::FilterInvalid => "filter-invalid",
            Self::PositionWithCursor => "position-with-cursor",
            Self::SearchKindRefused => "search-kind-refused",
            Self::SearchPolicyInvalid => "search-policy-invalid",
            Self::CannotCalculateChanges => "cannot-calculate-changes",
            Self::EstimateMixed => "estimate-mixed",
            Self::LimitRefused => "limit-refused",
        }
    }
}

/// The parameter keys of [`RecordsCode`].
pub mod keys {
    /// A data type name.
    pub const TYPE: &str = "type";
    /// A field path, steps joined by `/`.
    pub const PATH: &str = "path";
    /// A grouping name.
    pub const GROUPING: &str = "grouping";
    /// An order name.
    pub const ORDER: &str = "order";
    /// A basic type word.
    pub const BASIC: &str = "basic";
    /// What there are too many of: `nodes` (of the filter tree), `sort`, `fields`; or what is wrong (`empty-operator`).
    pub const WHAT: &str = "what";
    /// A limit.
    pub const MAX: &str = "max";
    /// A page size asked.
    pub const SIZE: &str = "size";
    /// How many came.
    pub const GOT: &str = "got";
    /// A length in bytes.
    pub const BYTES: &str = "bytes";
    /// Milliseconds.
    pub const MS: &str = "ms";
    /// The message key of the cause.
    pub const CAUSE: &str = "cause";
    /// A group key.
    pub const KEY: &str = "key";
    /// The depth of a filter tree.
    pub const DEPTH: &str = "depth";
    /// A search kind word.
    pub const KIND: &str = "kind";
}
