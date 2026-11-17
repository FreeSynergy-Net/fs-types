//! What a host asks: one data type, a scope, a filter tree (Nachzug 17.11.: JMAP `/query`, [`Filter`]), sort keys, an
//! optional grouping, the fields it shows, a page (`position` or cursor, and `size` = JMAP `limit`).
//!
//! Closed forms only (ADR 0087 Nachtrag points 5, 12): no expression, no query language. Values are canonical texts of the
//! field's basic type (as `fs_types::FieldEdit::values`), never JSON.
//!
//! Pattern: Value object with a Builder-style chain.

use fs_types::PageCursor;

use super::filter::Filter;
use super::search::SearchKind;

/// One condition on one field (top-level path in v1): a leaf of the [`Filter`] tree (JMAP `FilterCondition`). On a field the
/// person may not read it is FALSE whatever the value (Nachzug: a hidden value never changes a result; under `Not` the
/// leaf stays false, so the result does not depend on the value either).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    /// The field holds this value (for a `multiple` field: among its values).
    Equals {
        /// The field.
        path: Vec<String>,
        /// The canonical text.
        value: String,
    },
    /// The field holds one of these values.
    AnyOf {
        /// The field.
        path: Vec<String>,
        /// The canonical texts.
        values: Vec<String>,
    },
    /// The field lies in `[from, to]` (both included, `None` = open): dates and date-times in time order, choices in the
    /// order of their list, texts in code point order.
    Range {
        /// The field.
        path: Vec<String>,
        /// The lower end.
        from: Option<String>,
        /// The upper end.
        to: Option<String>,
    },
    /// The field has no value.
    IsEmpty {
        /// The field.
        path: Vec<String>,
    },
    /// Some readable text field contains this, case-insensitive. A hidden field is never searched (no leak through search).
    Term(String),
}

impl Condition {
    /// The field the condition is about (`None` for `Term`).
    #[must_use]
    pub fn path(&self) -> Option<&[String]> {
        match self {
            Self::Equals { path, .. }
            | Self::AnyOf { path, .. }
            | Self::Range { path, .. }
            | Self::IsEmpty { path } => Some(path),
            Self::Term(_) => None,
        }
    }
}

/// What a sort key sorts by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SortBy {
    /// A field (empty values last, whatever the direction).
    Field(Vec<String>),
    /// A named order of the shape (`rank` = the order by hand, `due`, `priority`).
    Order(String),
}

/// One sort key. After all keys, ties are broken by record id, ascending, always (stable pages).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sort {
    /// What it sorts by.
    pub by: SortBy,
    /// Largest first.
    pub descending: bool,
}

/// One read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordQuery {
    /// The data type (`Task`).
    pub data_type: String,
    /// The container record (a task list) as its id; `None` = all the person may see.
    pub scope: Option<String>,
    /// The filter tree; `None` = every record (Nachzug 17.11.; replaces the flat AND list of the first draft).
    pub filter: Option<Filter>,
    /// In order; ties by id.
    pub sort: Vec<Sort>,
    /// A grouping of the shape: every record of the page then carries its group key.
    pub group: Option<String>,
    /// The fields to answer; empty = every field the person may read.
    pub fields: Vec<Vec<String>>,
    /// Where the page starts: `None` = at the beginning; otherwise the `next` of the page before, with the same query.
    pub after: Option<PageCursor>,
    /// Nachzug: JMAP `position`, the 0-based index in the VISIBLE result (records she may not read take no index);
    /// together with `after` it is `position-with-cursor`.
    pub position: Option<u32>,
    /// At most this many records (1 ..= the limit).
    pub size: u32,
    /// Nachzug: the search kind asked; `None` = the type's default; outside the allowed ones = `search-kind-refused`.
    pub search: Option<SearchKind>,
}

impl RecordQuery {
    /// All records of `data_type`, `size` per page, no condition, no sort but the id.
    #[must_use]
    pub fn new(data_type: &str, size: u32) -> Self {
        Self {
            data_type: data_type.to_owned(),
            scope: None,
            filter: None,
            sort: Vec::new(),
            group: None,
            fields: Vec::new(),
            after: None,
            position: None,
            size,
            search: None,
        }
    }

    /// Only records in this container.
    #[must_use]
    pub fn scope(mut self, id: &str) -> Self {
        self.scope = Some(id.to_owned());
        self
    }

    /// One more condition, AND-ed to the tree.
    #[must_use]
    pub fn when(self, condition: Condition) -> Self {
        self.filtered(Filter::Is(condition))
    }

    /// One more filter, AND-ed to the tree.
    #[must_use]
    pub fn filtered(mut self, filter: Filter) -> Self {
        self.filter = Some(match self.filter.take() {
            None => filter,
            Some(f) => f.and(filter),
        });
        self
    }

    /// Start at this index of the visible result (JMAP `position`).
    #[must_use]
    pub fn at(mut self, position: u32) -> Self {
        self.position = Some(position);
        self
    }

    /// One more sort key.
    #[must_use]
    pub fn sort_by(mut self, by: SortBy, descending: bool) -> Self {
        self.sort.push(Sort { by, descending });
        self
    }

    /// Group by this grouping.
    #[must_use]
    pub fn grouped(mut self, grouping: &str) -> Self {
        self.group = Some(grouping.to_owned());
        self
    }

    /// Answer only these top-level fields.
    #[must_use]
    pub fn only(mut self, fields: &[&str]) -> Self {
        self.fields = fields.iter().map(|f| vec![(*f).to_owned()]).collect();
        self
    }

    /// The page after `cursor`.
    #[must_use]
    pub fn after(mut self, cursor: PageCursor) -> Self {
        self.after = Some(cursor);
        self
    }
}

/// A one-step path from a field name.
#[must_use]
pub fn path(name: &str) -> Vec<String> {
    vec![name.to_owned()]
}
