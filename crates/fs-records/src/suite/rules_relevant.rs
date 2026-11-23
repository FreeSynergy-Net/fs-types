//! Nachzug 2 (17.11., relevant): an estimate is never shown as exact nor the reverse, and a type without `relevant` in its
//! policy refuses that kind (before any „unsupported“: the owner's policy is asked first).
//!
//! Pattern: one rule per behaviour (Strategy), written with `rule!`.

use super::check::{Stop, arrange, call, refused, rule, wrong};
use super::codes::SuiteCode;
use super::fixture::Arrange;
use crate::records::{RecordQuery, RecordsCode, SearchKind};

rule!(
    /// Every page: never `total` and `total_estimated` together; `estimate_kind` exactly when there is an estimate; the
    /// host check passes with the limits THE PROVIDER declares (`Records::limits`, not the default: Regel 57).
    EstimateHonest,
    "records.estimate-honest",
    |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let q = RecordQuery::new(&facts.data_type, 2);
        let page = call(f.records().find(&q).await)?;
        let mixed = page.total.is_some() && page.total_estimated.is_some();
        let kind_wrong = page.total_estimated.is_some() != page.estimate_kind.is_some();
        if mixed || kind_wrong || f.records().limits().check_page(&q, &page).is_err() {
            let got = format!(
                "total={:?},estimated={:?}",
                page.total, page.total_estimated
            );
            return Err(wrong(
                SuiteCode::EstimateWrong,
                "one-of-total-or-estimate",
                got,
            ));
        }
        Ok(())
    }
);

rule!(
    /// A type whose policy does not allow `relevant` refuses it with `search-kind-refused` (not `unsupported`).
    RelevantRefused,
    "records.relevant-refused",
    |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let shape = call(f.records().shape(&facts.data_type).await)?;
        if shape.search.allowed().contains(&SearchKind::Relevant) {
            return Err(Stop::Verdict(
                cx.missing(fs_conformance::Need::new("fixture.no-relevant")),
            ));
        }
        let mut q = RecordQuery::new(&facts.data_type, 10);
        q.search = Some(SearchKind::Relevant);
        refused(f.records().find(&q).await, RecordsCode::SearchKindRefused)
    }
);
