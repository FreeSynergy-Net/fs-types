//! `PageCursor`, `Page`, `Capabilities`, `Problem`: what a topic call returns around its data.
//!
//! Pattern: Value object (plain results of a call). `PageCursor` is the opaque place to continue a list (the first
//! topic crates call it `Place`; here that name belongs to the `geo:` point, see `place.rs`).

use crate::TypeProblem;

/// An opaque place to continue a list from; only the provider that gave it can read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageCursor(pub String);

/// One page of a result.
#[derive(Clone, Debug)]
pub struct Page<T> {
    /// The entries of this page.
    pub items: Vec<T>,
    /// Where the next page starts; none at the end.
    pub next: Option<PageCursor>,
}

/// What a provider offers, per capability name (`event.create`, `person.read` …): `true` = offered.
#[derive(Clone, Debug, Default)]
pub struct Capabilities(pub Vec<(String, bool)>);

/// One failed check at a field. `reason` is a code, never a sentence (ADR 0057).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// The field the check failed at.
    pub field: String,
    /// The code (see [`TypeProblem::code`] for a basic type).
    pub reason: String,
}

impl Problem {
    /// A field problem from the code of a basic-type problem.
    #[must_use]
    pub fn from_type(field: &str, problem: TypeProblem) -> Self {
        Self {
            field: field.to_owned(),
            reason: problem.code().to_owned(),
        }
    }
}
