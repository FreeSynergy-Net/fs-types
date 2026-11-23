//! What a provider answers: records as typed values, with what the person may not read, the lock and the commands she may
//! run now. No JSON (ADR 0086): every value is the canonical text of its basic type.
//!
//! Pattern: Value objects.

use fs_types::{BasicType, PageCursor};

use super::codes::{RecordsCode, keys};
use crate::coded::Coded;

/// The values of one field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldValue {
    /// The field.
    pub path: Vec<String>,
    /// Its basic type (so a reader can check without the description).
    pub basic: BasicType,
    /// Canonical texts; empty = the field has no value.
    pub values: Vec<String>,
}

/// Why the person does not see a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HiddenReason {
    /// She has no right to read it.
    NoRight,
    /// It is marked sensitive and not shared with her (ADR 0029).
    Sensitive,
}

/// A field the person does not see. It is SHOWN as locked with its reason (never silently missing) and carries no value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hidden {
    /// The field.
    pub path: Vec<String>,
    /// Why.
    pub reason: HiddenReason,
}

/// What kind of lock a record carries. The program decides it; the lock component only shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LockKind {
    /// Someone has taken it over until a time (tasks `claim`).
    Claimed,
    /// The person may read but not change it (no right to write).
    ReadOnly,
    /// An admin locked it (with a reason, ADR 0021: admin locks are never silent).
    Admin,
}

/// The lock state of a record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordLock {
    /// What kind.
    pub kind: LockKind,
    /// Who holds it: the IRI of a person (`PersonRef` id); `None` for `ReadOnly`.
    pub by: Option<String>,
    /// Until when, milliseconds since 1970 in UTC; `None` = until released.
    pub until_ms: Option<i64>,
    /// The reason as the locker wrote it (user text, shown as written; not a code).
    pub reason: Option<String>,
}

/// One record as the asking person may see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordView {
    /// The identity (AS2 `id`, an IRI); a host never makes one up.
    pub id: String,
    /// Changes at every write; compared for equality only (compare-and-set in E4w).
    pub revision: u64,
    /// The readable fields asked for, in description order. A field without value may be left out.
    pub values: Vec<FieldValue>,
    /// The fields asked for that she may not read.
    pub hidden: Vec<Hidden>,
    /// The lock, if any.
    pub lock: Option<RecordLock>,
    /// The record commands she may run on it now (names of the description). Decision 85 (c0, 18.11.2026): it names ONLY
    /// rights, the program decides by her rights alone: a command she has the right to is in it even when the program does
    /// not do it yet (`CommandDescription::supported == false`, the host shows it as „coming“) and even while the record is
    /// locked (the lock is `lock`, the host shows it as a lock); a command she has no right to is never in it. Commands of the
    /// type as a whole (`create` ...) are in `Shape::may` (decision 86).
    pub may: Vec<String>,
    /// The group key in the grouping of the query; `None` when the query has no grouping.
    pub group: Option<String>,
}

impl RecordView {
    /// The values of a top-level field.
    #[must_use]
    pub fn value(&self, field: &str) -> Option<&FieldValue> {
        self.values
            .iter()
            .find(|v| v.path.len() == 1 && v.path[0] == field)
    }

    /// Whether the field is hidden.
    #[must_use]
    pub fn is_hidden(&self, field: &str) -> bool {
        self.hidden
            .iter()
            .any(|h| h.path.len() == 1 && h.path[0] == field)
    }

    /// The rule „a hidden field has no value“, checked by the host on every record it receives.
    ///
    /// # Errors
    /// `hidden-has-value` with the path.
    pub fn check_hidden(&self) -> Result<(), Coded<RecordsCode>> {
        match self
            .hidden
            .iter()
            .find(|h| self.values.iter().any(|v| v.path == h.path))
        {
            Some(h) => {
                Err(Coded::new(RecordsCode::HiddenHasValue).with_text(keys::PATH, h.path.join("/")))
            }
            None => Ok(()),
        }
    }
}

/// One page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordPage {
    /// At most `size` records.
    pub items: Vec<RecordView>,
    /// Where the next page starts; `None` = this was the last.
    pub next: Option<PageCursor>,
    /// Nachzug 17.11.: the index of the first item in the visible result (JMAP `position`).
    pub position: u32,
    /// Nachzug: JMAP `total`, the number of matching records SHE MAY READ; `None` = the provider does not count. A record she
    /// may not read is never counted, never takes an index, never moves another record's place (suite rule
    /// `records.unreadable-uncounted`).
    pub total: Option<u64>,
    /// Nachzug (Suche-Arten): the query state of this result (JMAP `queryState`), for `changes`.
    pub state: String,
    /// Nachzug: whether nothing was left out (a `complete` search is always true or an error; locally always true today).
    pub complete: bool,
    /// Nachzug 2: an ESTIMATED number of matches (search kind `relevant`); never together with `total` (exact).
    pub total_estimated: Option<u64>,
    /// Nachzug 2: what kind of estimate it is; set exactly when `total_estimated` is.
    pub estimate_kind: Option<EstimateKind>,
    /// Nachzug 2: text snippets of the matches (only from fields she may read); empty outside `relevant`.
    pub snippets: Vec<Snippet>,
}

/// Nachzug 2: how an estimated total is to be read (named honestly, never shown as exact).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EstimateKind {
    /// At least this many.
    AtLeast,
    /// About this many.
    Approximate,
    /// At most this many.
    AtMost,
}

/// Nachzug 2: a piece of text of one match that shows why it matched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snippet {
    /// The record.
    pub id: String,
    /// The field the text is from (must be readable for her).
    pub path: Vec<String>,
    /// The piece of text (plain text; the matched words are found again by the frontend, no markup).
    pub text: String,
}
