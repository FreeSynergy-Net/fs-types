//! Rules of rights: hidden fields carry no value, locks are shown, a missing and an unreadable record look the same (`None`),
//! an unreadable record never appears in a list.
//!
//! Pattern: one rule per behaviour (Strategy), written with `rule!`.

use super::check::{Stop, arrange, call, rule};
use super::codes::{SuiteCode, keys};
use super::fixture::Arrange;
use crate::records::{RecordQuery, RecordView};
use fs_conformance::Coded;

fn first(ids: &[String]) -> String {
    ids.first().cloned().unwrap_or_default()
}

/// The hidden field is listed as hidden and has no value, in `get` and in `find`.
fn no_leak(r: &RecordView, field: &str) -> Result<(), Stop> {
    if r.is_hidden(field) && r.value(field).is_none() && r.check_hidden().is_ok() {
        Ok(())
    } else {
        Err(Stop::Fail(
            Coded::new(SuiteCode::HiddenLeak).with_text(keys::PATH, field),
        ))
    }
}

rule!(
    /// A field the person may not read is listed as hidden and carries no value (ADR 0087 Nachtrag, mandatory test).
    HiddenNoValue, "records.hidden", |f, cx| {
        let facts = arrange(f, cx, Arrange::Hidden)?;
        let id = first(&facts.ids);
        let got = call(f.records().get(&facts.data_type, &id, &[]).await)?;
        let Some(view) = got else {
            return Err(Stop::Fail(Coded::new(SuiteCode::HiddenLeak).with_text(keys::PATH, facts.hidden.clone())));
        };
        no_leak(&view, &facts.hidden)?;
        let page = call(f.records().find(&RecordQuery::new(&facts.data_type, 10)).await)?;
        for r in page.items.iter().filter(|r| r.id == id) {
            no_leak(r, &facts.hidden)?;
        }
        Ok(())
    }
);

rule!(
    /// A locked record comes with its lock, in `get` and in `find`.
    LockVisible, "records.lock", |f, cx| {
        let facts = arrange(f, cx, Arrange::Locked)?;
        let id = first(&facts.ids);
        let missing = || Stop::Fail(Coded::new(SuiteCode::LockMissing).with_text(keys::ID, id.clone()));
        let got = call(f.records().get(&facts.data_type, &id, &[]).await)?;
        if got.and_then(|v| v.lock).is_none() {
            return Err(missing());
        }
        let page = call(f.records().find(&RecordQuery::new(&facts.data_type, 10)).await)?;
        if page.items.iter().any(|r| r.id == id && r.lock.is_none()) {
            return Err(missing());
        }
        Ok(())
    }
);

rule!(
    /// `get` of an id that does not exist answers `None`, not an error.
    MissingIsNone, "records.missing", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        match f.records().get(&facts.data_type, "urn:uuid:ffffffff-ffff-7fff-bfff-ffffffffffff", &[]).await {
            Ok(None) => Ok(()),
            Ok(Some(_)) => Err(Stop::Fail(Coded::new(SuiteCode::MissingWrong).with_text(keys::GOT, "some"))),
            Err(e) => Err(Stop::Fail(
                Coded::new(SuiteCode::MissingWrong).with_text(keys::GOT, crate::coded::message_key(e.code())),
            )),
        }
    }
);

rule!(
    /// A record the person may not read is not in any page, and `get` answers `None` for it (as for a missing one).
    UnreadableAbsent, "records.unreadable", |f, cx| {
        let facts = arrange(f, cx, Arrange::Unreadable)?;
        let id = first(&facts.ids);
        let shown = || Stop::Fail(Coded::new(SuiteCode::UnreadableShown).with_text(keys::ID, id.clone()));
        let page = call(f.records().find(&RecordQuery::new(&facts.data_type, 10)).await)?;
        if page.items.iter().any(|r| r.id == id) {
            return Err(shown());
        }
        if call(f.records().get(&facts.data_type, &id, &[]).await)?.is_some() {
            return Err(shown());
        }
        Ok(())
    }
);
