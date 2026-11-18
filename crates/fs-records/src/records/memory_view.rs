//! How [`super::MemoryRecords`] answers one record: projection, hidden fields, group key, lock.
//!
//! Pattern: pure functions over the double's state.

use super::memory::{Flaw, MemoryRecords};
use super::memory_row::Row;
use super::shape::GroupingDescription;
use super::view::{FieldValue, Hidden, HiddenReason, RecordView};

/// The values of a top-level field the person may read; a hidden field has none (and never matches a condition).
pub(super) fn readable<'a>(r: &'a Row, name: &str) -> &'a [String] {
    if r.hidden.iter().any(|h| h == name) {
        &[]
    } else {
        r.values.get(name).map_or(&[], Vec::as_slice)
    }
}

pub(super) fn view(
    m: &MemoryRecords,
    r: &Row,
    fields: &[Vec<String>],
    grouping: Option<&GroupingDescription>,
) -> RecordView {
    let names: Vec<&str> = if fields.is_empty() {
        m.description
            .fields
            .iter()
            .map(|f| f.name.as_str())
            .collect()
    } else {
        fields
            .iter()
            .filter_map(|p| p.first().map(String::as_str))
            .collect()
    };
    let mut values = Vec::new();
    let mut hidden = Vec::new();
    for name in names {
        let Some(f) = m.description.find_field(name) else {
            continue;
        };
        let is_hidden = r.hidden.iter().any(|h| h == name);
        if is_hidden {
            hidden.push(Hidden {
                path: vec![name.to_owned()],
                reason: HiddenReason::NoRight,
            });
        }
        let stored = r.values.get(name).filter(|v| !v.is_empty());
        if let Some(v) = stored.filter(|_| !is_hidden || m.has(Flaw::HiddenLeaks)) {
            values.push(FieldValue {
                path: vec![name.to_owned()],
                basic: f.basic,
                values: v.clone(),
            });
        }
    }
    let group = grouping.map(|g| match (&g.field, m.has(Flaw::GroupInvented)) {
        (_, true) => "invented".to_owned(),
        (None, false) => r.group.clone().unwrap_or_default(),
        (Some(p), false) => p
            .first()
            .and_then(|n| readable(r, n).first().cloned())
            .unwrap_or_default(),
    });
    let lock = if m.has(Flaw::LockDropped) {
        None
    } else {
        r.lock.clone()
    };
    RecordView {
        id: r.id.clone(),
        revision: 0,
        values,
        hidden,
        lock,
        may: record_may(m, r),
        group,
    }
}

/// The `may` of one record: her rights, unless a flaw is on (decision 85).
fn record_may(m: &MemoryRecords, r: &Row) -> Vec<String> {
    if m.has(Flaw::MayIgnoresRights) {
        m.description
            .record_commands()
            .iter()
            .map(|c| c.name.clone())
            .collect()
    } else if m.has(Flaw::MayDroppedWhenLocked) && r.lock.is_some() {
        Vec::new()
    } else if m.has(Flaw::MayBySupported) {
        m.only_supported(&r.may)
    } else {
        r.may.clone()
    }
}
