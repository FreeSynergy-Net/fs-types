//! Reading of [`MemoryRecords`]: checks, filter, sort, page, projection. The rules are the trait's (draft §4.3); the double
//! compares date and date-time texts as texts (a real provider compares instants).
//!
//! Pattern: pure functions over the double's state.

use std::cmp::Ordering;

use fs_types::{BasicType, FieldDescription, PageCursor};

use super::codes::{RecordsCode, keys};
use super::memory::{Flaw, MemoryRecords};
use super::memory_row::Row;
use super::memory_view::{readable, view};
use super::query::{Condition, RecordQuery, SortBy};
use super::shape::Group;
use super::view::{RecordPage, RecordView};
use crate::coded::Coded;

type Found<T> = Result<T, Coded<RecordsCode>>;

pub(super) fn find(m: &MemoryRecords, q: &RecordQuery) -> Found<RecordPage> {
    m.type_check(&q.data_type)?;
    if !m.has(Flaw::SizeUnchecked) {
        m.limits.check_query(q)?;
    }
    let sort_paths = q.sort.iter().filter_map(|s| match &s.by {
        SortBy::Field(p) => Some(p.as_slice()),
        SortBy::Order(_) => None,
    });
    let cond_paths = q.conditions.iter().filter_map(Condition::path);
    for p in cond_paths
        .chain(sort_paths)
        .chain(q.fields.iter().map(Vec::as_slice))
    {
        field(m, p)?;
    }
    for s in &q.sort {
        if let SortBy::Order(name) = &s.by
            && !m.shape.orders.iter().any(|o| &o.name == name)
        {
            return Err(Coded::new(RecordsCode::UnknownOrder).with_text(keys::ORDER, name.as_str()));
        }
    }
    for c in &q.conditions {
        choice_check(m, c)?;
    }
    let grouping = match &q.group {
        None => None,
        Some(g) => Some(m.grouping(g)?),
    };
    let rows = m.rows.borrow();
    let mut hits: Vec<(usize, &Row)> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| visible(m, r) && (q.scope.is_none() || r.scope == q.scope))
        .filter(|(_, r)| {
            m.has(Flaw::ConditionsIgnored) || q.conditions.iter().all(|c| holds(m, r, c))
        })
        .collect();
    hits.sort_by(|a, b| compare(m, q, *a, *b));
    let start = match (&q.after, m.has(Flaw::CursorIgnored)) {
        (Some(c), false) => {
            c.0.strip_prefix("m:")
                .and_then(|n| n.parse::<usize>().ok())
                .ok_or_else(|| Coded::new(RecordsCode::CursorInvalid))?
        }
        _ => 0,
    };
    let end = start.saturating_add(q.size as usize).min(hits.len());
    let items = hits
        .get(start..end)
        .unwrap_or(&[])
        .iter()
        .map(|(_, r)| view(m, r, &q.fields, grouping))
        .collect();
    let next = (end < hits.len()).then(|| PageCursor(format!("m:{end}")));
    Ok(RecordPage { items, next })
}

pub(super) fn get(
    m: &MemoryRecords,
    data_type: &str,
    id: &str,
    fields: &[Vec<String>],
) -> Found<Option<RecordView>> {
    m.type_check(data_type)?;
    for p in fields {
        field(m, p)?;
    }
    let rows = m.rows.borrow();
    match rows.iter().find(|r| r.id == id && visible(m, r)) {
        Some(r) => Ok(Some(view(m, r, fields, None))),
        None if m.has(Flaw::MissingIsError) => {
            Err(Coded::new(RecordsCode::Unavailable).with_text(keys::CAUSE, "missing"))
        }
        None => Ok(None),
    }
}

/// The groups of a `choice` field: one per key, then the group with the empty key for records without a value.
pub(super) fn choice_groups(m: &MemoryRecords, path: &[String]) -> Vec<Group> {
    let keys = path
        .first()
        .and_then(|f| m.description.find_field(f))
        .map(|f| f.choices.clone())
        .unwrap_or_default();
    keys.into_iter()
        .chain(std::iter::once(String::new()))
        .map(|key| Group {
            key,
            title: None,
            hint_limit: None,
        })
        .collect()
}

fn visible(m: &MemoryRecords, r: &Row) -> bool {
    r.readable || m.has(Flaw::UnreadableShown)
}

fn field<'a>(m: &'a MemoryRecords, path: &[String]) -> Found<Option<&'a FieldDescription>> {
    let found = match path {
        [one] => m.description.find_field(one),
        _ => None,
    };
    match found {
        Some(f) => Ok(Some(f)),
        None if m.has(Flaw::UnknownFieldIgnored) => Ok(None),
        None => Err(Coded::new(RecordsCode::UnknownField)
            .with_text(keys::TYPE, m.description.name.as_str())
            .with_text(keys::PATH, path.join("/"))),
    }
}

/// A choice value must be one of the keys.
fn choice_check(m: &MemoryRecords, c: &Condition) -> Found<()> {
    let values: Vec<&String> = match c {
        Condition::Equals { value, .. } => vec![value],
        Condition::AnyOf { values, .. } => values.iter().collect(),
        _ => Vec::new(),
    };
    let Some(f) = c.path().and_then(|p| field(m, p).ok().flatten()) else {
        return Ok(());
    };
    match values
        .iter()
        .find(|v| f.basic == BasicType::Choice && !f.choices.contains(v))
    {
        Some(_) => Err(Coded::new(RecordsCode::ValueMismatch)
            .with_text(keys::PATH, f.name.as_str())
            .with_text(keys::BASIC, "choice")),
        None => Ok(()),
    }
}

/// The first step of a path (v1 paths have one).
fn head(p: &[String]) -> &str {
    p.first().map_or("", String::as_str)
}

fn holds(m: &MemoryRecords, r: &Row, c: &Condition) -> bool {
    let first = head;
    match c {
        Condition::Equals { path, value } => readable(r, first(path)).contains(value),
        Condition::AnyOf { path, values } => {
            readable(r, first(path)).iter().any(|v| values.contains(v))
        }
        Condition::IsEmpty { path } => {
            !r.hidden.iter().any(|h| h == first(path)) && readable(r, first(path)).is_empty()
        }
        Condition::Range { path, from, to } => readable(r, first(path)).first().is_some_and(|v| {
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
                    readable(r, &f.name)
                        .iter()
                        .any(|v| v.to_lowercase().contains(&t))
                })
        }
    }
}

/// Order of two values of a field: a choice in list order, everything else as text.
fn rank(m: &MemoryRecords, path: &[String], a: &str, b: &str) -> Ordering {
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

/// One sort key on two rows: the order, and whether a descending key may reverse it (empty values stay last whatever the
/// direction, so a comparison with an empty value is never reversed).
fn key_order(
    mem: &MemoryRecords,
    path: Option<&[String]>,
    (index_a, row_a): (usize, &Row),
    (index_b, row_b): (usize, &Row),
) -> (Ordering, bool) {
    let Some(path) = path else {
        return (index_a.cmp(&index_b), true);
    };
    let name = path.first().map_or("", String::as_str);
    match (readable(row_a, name).first(), readable(row_b, name).first()) {
        (None, None) => (Ordering::Equal, false),
        (None, Some(_)) => (Ordering::Greater, false),
        (Some(_), None) => (Ordering::Less, false),
        (Some(left), Some(right)) => (rank(mem, path, left, right), true),
    }
}

fn compare(
    mem: &MemoryRecords,
    query: &RecordQuery,
    first: (usize, &Row),
    second: (usize, &Row),
) -> Ordering {
    for key in &query.sort {
        let path = match &key.by {
            SortBy::Field(field) => Some(field.clone()),
            SortBy::Order(name) => mem
                .shape
                .orders
                .iter()
                .find(|order| &order.name == name)
                .and_then(|order| order.field.clone()),
        };
        let (order, may_turn) = key_order(mem, path.as_deref(), first, second);
        let order = if key.descending && may_turn {
            order.reverse()
        } else {
            order
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    if mem.has(Flaw::TiesUnordered) {
        Ordering::Equal
    } else {
        first.1.id.cmp(&second.1.id)
    }
}
