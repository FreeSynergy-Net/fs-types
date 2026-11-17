//! Reading of [`MemoryRecords`]: checks, filter tree (`memory_filter.rs`), sort, page (`position` or cursor, `total`),
//! projection. The rules are the trait's (draft §3.4, Nachzug §14). Records she may not read are dropped FIRST: they are never
//! counted, take no index and never move another record's place.
//!
//! Pattern: pure functions over the double's state.

use std::cmp::Ordering;

use fs_types::{FieldDescription, PageCursor};

use super::codes::{RecordsCode, keys};
use super::memory::{Flaw, MemoryRecords};
use super::memory_filter::{choice_check, eval, rank};
use super::memory_row::Row;
use super::memory_view::{readable, view};
use super::query::{Condition, RecordQuery, SortBy};
use super::search::SearchKind;
use super::shape::Group;
use super::view::{EstimateKind, RecordPage, RecordView};
use crate::coded::Coded;

type Found<T> = Result<T, Coded<RecordsCode>>;

pub(super) fn find(m: &MemoryRecords, q: &RecordQuery) -> Found<RecordPage> {
    m.type_check(&q.data_type)?;
    admit_query(m, q)?;
    let sort_paths = q.sort.iter().filter_map(|s| match &s.by {
        SortBy::Field(p) => Some(p.as_slice()),
        SortBy::Order(_) => None,
    });
    let leaves: Vec<&Condition> = q
        .filter
        .as_ref()
        .map(|f| f.conditions())
        .unwrap_or_default();
    let cond_paths = leaves.iter().filter_map(|c| c.path());
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
    for c in &leaves {
        choice_check(c.path().and_then(|p| field(m, p).ok().flatten()), c)?;
    }
    let grouping = match &q.group {
        None => None,
        Some(g) => Some(m.grouping(g)?),
    };
    let rows = m.rows.borrow();
    let matches = |r: &Row| {
        (q.scope.is_none() || r.scope == q.scope)
            && (m.has(Flaw::ConditionsIgnored) || q.filter.as_ref().is_none_or(|f| eval(m, r, f)))
    };
    let mut hits: Vec<(usize, &Row)> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| visible(m, r) && matches(r))
        .collect();
    hits.sort_by(|a, b| compare(m, q, *a, *b));
    let start = match (&q.after, q.position, m.has(Flaw::CursorIgnored)) {
        (Some(c), _, false) => {
            c.0.strip_prefix("m:")
                .and_then(|n| n.parse::<usize>().ok())
                .ok_or_else(|| Coded::new(RecordsCode::CursorInvalid))?
        }
        (None, Some(p), _) => p as usize,
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
    let hidden_too = if m.has(Flaw::TotalCountsUnreadable) {
        rows.iter().filter(|r| !visible(m, r) && matches(r)).count()
    } else {
        0
    };
    let total = Some((hits.len() + hidden_too) as u64);
    let position = u32::try_from(start).unwrap_or(u32::MAX);
    let (total_estimated, estimate_kind) = if m.has(Flaw::EstimateAsExact) {
        (total, Some(EstimateKind::Approximate))
    } else {
        (None, None)
    };
    let state = m.state();
    Ok(RecordPage {
        items,
        next,
        position,
        total,
        state,
        complete: true,
        total_estimated,
        estimate_kind,
        snippets: Vec::new(),
    })
}

/// The checks before any record is read: the limits of the double, the search kind and its policy.
fn admit_query(m: &MemoryRecords, q: &RecordQuery) -> Found<()> {
    if !m.has(Flaw::SizeUnchecked) {
        let mut limits = m.limits;
        if m.has(Flaw::DepthUnchecked) {
            limits.filter_depth_max = usize::MAX;
            limits.filter_nodes_max = usize::MAX;
        }
        limits.check_query(q)?;
    }
    if m.has(Flaw::RelevantBeforePolicy) && q.search == Some(SearchKind::Relevant) {
        return Err(Coded::new(RecordsCode::Unsupported).with_text(keys::WHAT, "relevant"));
    }
    let kind = if m.has(Flaw::SearchKindIgnored) {
        q.search.unwrap_or(SearchKind::Complete)
    } else {
        m.shape.search.choose(q.search)?
    };
    if kind == SearchKind::Relevant {
        // Nachzug 2: the double computes no relevance (that is the relevance module / fs-search).
        return Err(Coded::new(RecordsCode::Unsupported).with_text(keys::WHAT, "relevant"));
    }
    Ok(())
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
