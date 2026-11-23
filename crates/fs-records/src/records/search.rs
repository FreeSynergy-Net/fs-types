//! Nachzug 17.11. (Suche-Arten, Plan G2f Punkt 4, N47–N48; Gespräch 4 Z. 78/79): which KINDS of search a data type allows,
//! which one is its default, and the local change feed of a query (JMAP RFC 8620 §5.6 `/queryChanges`).
//!
//! The owner decides per data type (an invoice: only `complete`; a location: `open`/`live` allowed); the person searching
//! chooses only inside what is allowed, otherwise `search-kind-refused`. Records stays LOCAL: hand-over, collecting,
//! deadlines, quorum and cancelling across several sources, and „incomplete + who is missing“, belong to `fs-search` later;
//! here a page only says whether it is `complete`.
//!
//! Where it lives: on the wire in `TypeDescription` (appended `searchKinds`, `searchDefault`); in this probe on
//! [`super::Shape`] because the Rust `TypeDescription` of fs-types 339aa24 is fixed (draft §14, O19).
//!
//! Pattern: Value objects + Specification ([`SearchPolicy::new`], [`SearchPolicy::choose`]).

use super::codes::{RecordsCode, keys};
use crate::coded::Coded;

/// One kind of search.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SearchKind {
    /// Every match, or an error: nothing is left out (an invoice, a ledger).
    Complete,
    /// Results may come in parts while the search goes on.
    Open,
    /// The person may cancel it; what came stays shown as incomplete.
    Cancellable,
    /// It ends at a deadline; what came by then is the answer.
    Deadline,
    /// It ends when enough matches came (the page size).
    Enough,
    /// It stays open and follows changes (`queryChanges`).
    Live,
    /// Nachzug 2: the best matches by relevance, total ESTIMATED, with snippets. Records does not compute it locally
    /// (`unsupported{what=relevant}`); it comes with the relevance module (texts relevance lifted; tantivy candidate) or
    /// fs-search. A type allows it only through its policy (an invoice never).
    Relevant,
}

impl SearchKind {
    /// All kinds, in this order.
    pub const ALL: [Self; 7] = [
        Self::Complete,
        Self::Open,
        Self::Cancellable,
        Self::Deadline,
        Self::Enough,
        Self::Live,
        Self::Relevant,
    ];

    /// The word (wire enumerant in lower camel case is the same word).
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Open => "open",
            Self::Cancellable => "cancellable",
            Self::Deadline => "deadline",
            Self::Enough => "enough",
            Self::Live => "live",
            Self::Relevant => "relevant",
        }
    }
}

/// What a data type allows, and its default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchPolicy {
    allowed: Vec<SearchKind>,
    default: SearchKind,
}

impl SearchPolicy {
    /// Checked: at least one kind, no kind twice, the default among them.
    ///
    /// # Errors
    /// `search-policy-invalid{kind}` (the default, or the repeated kind).
    pub fn new(allowed: &[SearchKind], default: SearchKind) -> Result<Self, Coded<RecordsCode>> {
        let bad = |k: SearchKind| {
            Coded::new(RecordsCode::SearchPolicyInvalid).with_text(keys::KIND, k.key())
        };
        if let Some(twice) = allowed
            .iter()
            .enumerate()
            .find(|(i, k)| allowed[..*i].contains(k))
            .map(|(_, k)| *k)
        {
            return Err(bad(twice));
        }
        if !allowed.contains(&default) {
            return Err(bad(default));
        }
        Ok(Self {
            allowed: allowed.to_vec(),
            default,
        })
    }

    /// Only `complete` (the strict one: what an invoice needs).
    #[must_use]
    pub fn complete_only() -> Self {
        Self {
            allowed: vec![SearchKind::Complete],
            default: SearchKind::Complete,
        }
    }

    /// The allowed kinds.
    #[must_use]
    pub fn allowed(&self) -> &[SearchKind] {
        &self.allowed
    }

    /// The default.
    #[must_use]
    pub fn default_kind(&self) -> SearchKind {
        self.default
    }

    /// The kind a query gets: the asked one if allowed, the default if none was asked.
    ///
    /// # Errors
    /// `search-kind-refused{kind}`.
    pub fn choose(&self, asked: Option<SearchKind>) -> Result<SearchKind, Coded<RecordsCode>> {
        match asked {
            None => Ok(self.default),
            Some(k) if self.allowed.contains(&k) => Ok(k),
            Some(k) => {
                Err(Coded::new(RecordsCode::SearchKindRefused).with_text(keys::KIND, k.key()))
            }
        }
    }
}

/// The changes of a query result since a state (JMAP `/queryChanges`). Only records she may read appear; a record that
/// became unreadable is `removed` like a deleted one (nothing tells which).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryChanges {
    /// The state asked from.
    pub old_state: String,
    /// The state now.
    pub new_state: String,
    /// Ids no longer in the result.
    pub removed: Vec<String>,
    /// Ids now in the result, with their index in the visible result.
    pub added: Vec<(String, u32)>,
    /// The visible total now (`None` = not counted).
    pub total: Option<u64>,
}
