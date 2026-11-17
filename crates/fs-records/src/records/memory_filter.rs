//! Nachzug 17.11. (Suche/Filterbaum): how [`super::MemoryRecords`] evaluates the filter tree and its leaves, with the flaws
//! that prove the suite rules `records.filter-tree` and `records.hidden-filter-blind`.
//!
//! A leaf on a field the person may not read is false whatever its value; the double compares date and date-time texts
//! as texts (a real provider compares instants).
//!
//! Pattern: Interpreter over the Composite [`Filter`] (pure functions over the double's state).

use std::cmp::Ordering;

use fs_types::{BasicType, FieldDescription};

use super::codes::{RecordsCode, keys};
use super::filter::Filter;
use super::memory::{Flaw, MemoryRecords};
use super::memory_row::Row;
use super::memory_view::readable;
use super::query::Condition;
use crate::coded::Coded;

/// The tree for one record; the JMAP operators and the named `XOR` (exactly one), unless a flaw is on.
pub(super) fn eval(m: &MemoryRecords, r: &Row, f: &Filter) -> bool {
    match f {
        Filter::Is(c) => holds(m, r, c),
        Filter::And(n) => n.iter().all(|x| eval(m, r, x)),
        Filter::Or(n) => n.iter().any(|x| eval(m, r, x)),
        Filter::Not(n) if m.has(Flaw::NotDropped) => n.iter().any(|x| eval(m, r, x)),
        Filter::Not(n) => !n.iter().any(|x| eval(m, r, x)),
        Filter::Xor(n) => {
            let count = n.iter().filter(|x| eval(m, r, x)).count();
            if m.has(Flaw::XorParity) {
                count % 2 == 1
            } else {
                count == 1
            }
        }
    }
}

/// The first step of a path (v1 paths have one).
fn head(p: &[String]) -> &str {
    p.first().map_or("", String::as_str)
}

/// The values a leaf may see: only readable ones (flaw `HiddenFilterSees`: the stored ones).
fn seen<'a>(m: &MemoryRecords, r: &'a Row, name: &str) -> &'a [String] {
    if m.has(Flaw::HiddenFilterSees) {
        r.values.get(name).map_or(&[], Vec::as_slice)
    } else {
        readable(r, name)
    }
}

fn hidden(m: &MemoryRecords, r: &Row, name: &str) -> bool {
    !m.has(Flaw::HiddenFilterSees) && r.hidden.iter().any(|h| h == name)
}

fn holds(m: &MemoryRecords, r: &Row, c: &Condition) -> bool {
    match c {
        Condition::Equals { path, value } => seen(m, r, head(path)).contains(value),
        Condition::AnyOf { path, values } => {
            seen(m, r, head(path)).iter().any(|v| values.contains(v))
        }
        Condition::IsEmpty { path } => {
            !hidden(m, r, head(path)) && seen(m, r, head(path)).is_empty()
        }
        Condition::Range { path, from, to } => seen(m, r, head(path)).first().is_some_and(|v| {
            let low = from
                .as_ref()
                .is_none_or(|f| rank(m, path, f, v) != Ordering::Greater);
            let high = to
                .as_ref()
                .is_none_or(|t| rank(m, path, v, t) != Ordering::Greater);
            low && high
        }),
        Condition::Term(t) => {
            let t = t.to_lowercase();
            m.description
                .fields
                .iter()
                .filter(|f| matches!(f.basic, BasicType::Text | BasicType::MultilineText))
                .any(|f| {
                    seen(m, r, &f.name)
                        .iter()
                        .any(|v| v.to_lowercase().contains(&t))
                })
        }
    }
}

/// Order of two values of a field: a choice in list order, everything else as text.
pub(super) fn rank(m: &MemoryRecords, path: &[String], a: &str, b: &str) -> Ordering {
    let choices = path
        .first()
        .and_then(|f| m.description.find_field(f))
        .filter(|f| f.basic == BasicType::Choice);
    match choices {
        Some(f) => {
            let at = |v: &str| f.choices.iter().position(|c| c == v).unwrap_or(usize::MAX);
            at(a).cmp(&at(b))
        }
        None => a.cmp(b),
    }
}

/// A choice value must be one of the keys (typed values, ADR 0086: a value that cannot be stored cannot be asked for).
pub(super) fn choice_check(
    f: Option<&FieldDescription>,
    c: &Condition,
) -> Result<(), Coded<RecordsCode>> {
    let values: Vec<&String> = match c {
        Condition::Equals { value, .. } => vec![value],
        Condition::AnyOf { values, .. } => values.iter().collect(),
        Condition::Range { from, to, .. } => from.iter().chain(to.iter()).collect(),
        _ => Vec::new(),
    };
    let Some(f) = f.filter(|f| f.basic == BasicType::Choice) else {
        return Ok(());
    };
    if values.iter().any(|v| !f.choices.contains(*v)) {
        Err(Coded::new(RecordsCode::ValueMismatch)
            .with_text(keys::PATH, f.name.as_str())
            .with_text(keys::BASIC, "choice"))
    } else {
        Ok(())
    }
}
