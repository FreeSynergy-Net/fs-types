//! Nachzug 17.11. (Suche/Filterbaum): rules of the filter tree and of „a hidden entry is never revealed“ (not in the list,
//! not in `total`, not in a position, not through a condition on a hidden value), and of search kinds and `changes`.
//!
//! Pattern: one rule per behaviour (Strategy), written with `rule!`.

use std::collections::BTreeSet;

use super::check::{Stop, arrange, call, refused, rule, wrong};
use super::codes::{SuiteCode, keys};
use super::fixture::Arrange;
use crate::records::{Condition, Filter, RecordQuery, RecordsCode, SearchKind, path};
use fs_conformance::Coded;

fn is(c: Condition) -> Filter {
    Filter::Is(c)
}

fn equals(field: &str, value: &str) -> Filter {
    is(Condition::Equals {
        path: path(field),
        value: value.to_owned(),
    })
}

fn ids_of(page: crate::records::RecordPage) -> BTreeSet<String> {
    page.items.into_iter().map(|r| r.id).collect()
}

rule!(
    /// AND, OR, NOT (none of) and XOR (exactly one) over five records with five different values `v0..v4`:
    /// `AND[ XOR[ OR[v0, v1], ANY[v0, v2], NOT[v1] ], NOT[ IS-EMPTY ] ]` holds for exactly ids 1, 3, 4
    /// (id 0 has three true operands: odd parity would take it).
    FilterTree,
    "records.filter-tree",
    |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let (Some(v), Some(_)) = (facts.values.get(..3), facts.ids.get(4)) else {
            return Err(Stop::Verdict(
                cx.missing(fs_conformance::Need::new("fixture.five-values")),
            ));
        };
        let t = facts.text.as_str();
        let xor = Filter::Xor(vec![
            Filter::Or(vec![equals(t, &v[0]), equals(t, &v[1])]),
            is(Condition::AnyOf {
                path: path(t),
                values: vec![v[0].clone(), v[2].clone()],
            }),
            Filter::Not(vec![equals(t, &v[1])]),
        ]);
        let tree = Filter::And(vec![
            xor,
            Filter::Not(vec![is(Condition::IsEmpty { path: path(t) })]),
        ]);
        let page = call(
            f.records()
                .find(&RecordQuery::new(&facts.data_type, 10).filtered(tree))
                .await,
        )?;
        let got = ids_of(page);
        let want: BTreeSet<String> = [1, 3, 4].iter().map(|i| facts.ids[*i].clone()).collect();
        if got != want {
            let join = |s: &BTreeSet<String>| s.iter().cloned().collect::<Vec<_>>().join(",");
            return Err(wrong(SuiteCode::FilterWrong, join(&want), join(&got)));
        }
        Ok(())
    }
);

rule!(
    /// A tree one level deeper than the provider's `filter_depth_max` is refused with `filter-too-deep`; an operator
    /// without operands with `filter-invalid`.
    FilterLimits,
    "records.filter-limits",
    |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let mut tree = equals(&facts.text, &facts.probe);
        for _ in 0..f.records().limits().filter_depth_max {
            tree = Filter::Not(vec![tree]);
        }
        let deep = RecordQuery::new(&facts.data_type, 10).filtered(tree);
        refused(f.records().find(&deep).await, RecordsCode::FilterTooDeep)?;
        let empty = RecordQuery::new(&facts.data_type, 10).filtered(Filter::And(Vec::new()));
        refused(f.records().find(&empty).await, RecordsCode::FilterInvalid)
    }
);

rule!(
    /// A record she may not read is not counted in `total` and takes no position: position 0 is the first readable one.
    UnreadableUncounted,
    "records.unreadable-uncounted",
    |f, cx| {
        let facts = arrange(f, cx, Arrange::Unreadable)?;
        let page = call(
            f.records()
                .find(&RecordQuery::new(&facts.data_type, 10))
                .await,
        )?;
        let shown = page.items.len() as u64;
        if let Some(total) = page.total.filter(|t| *t != shown) {
            return Err(wrong(
                SuiteCode::CountWrong,
                shown.to_string(),
                total.to_string(),
            ));
        }
        let first = call(
            f.records()
                .find(&RecordQuery::new(&facts.data_type, 1).at(0))
                .await,
        )?;
        let got: Vec<String> = first.items.iter().map(|r| r.id.clone()).collect();
        let want: Vec<String> = facts.ids.get(1).cloned().into_iter().collect();
        if got != want || first.next.is_some() || first.position != 0 {
            return Err(wrong(SuiteCode::CountWrong, want.join(","), got.join(",")));
        }
        Ok(())
    }
);

rule!(
    /// A condition on a hidden value never changes a result: `EQUALS(hidden, its value)` and a term with it do not find the
    /// record, and `NOT EQUALS` gives the same records for the true value and for another one.
    HiddenFilterBlind,
    "records.hidden-filter-blind",
    |f, cx| {
        let facts = arrange(f, cx, Arrange::Hidden)?;
        let leak = || {
            Stop::Fail(
                Coded::new(SuiteCode::HiddenLeak).with_text(keys::PATH, facts.hidden.clone()),
            )
        };
        let q = |filter: Filter| RecordQuery::new(&facts.data_type, 10).filtered(filter);
        let id = facts.ids.first().cloned().unwrap_or_default();
        let hit = ids_of(call(
            f.records()
                .find(&q(equals(&facts.hidden, &facts.probe)))
                .await,
        )?);
        let term = ids_of(call(
            f.records()
                .find(&q(is(Condition::Term(facts.probe.clone()))))
                .await,
        )?);
        if hit.contains(&id) || term.contains(&id) {
            return Err(leak());
        }
        let other = format!("{}-other", facts.probe);
        let not_true = ids_of(call(
            f.records()
                .find(&q(Filter::Not(vec![equals(&facts.hidden, &facts.probe)])))
                .await,
        )?);
        let not_other = ids_of(call(
            f.records()
                .find(&q(Filter::Not(vec![equals(&facts.hidden, &other)])))
                .await,
        )?);
        if not_true != not_other {
            return Err(leak());
        }
        Ok(())
    }
);

rule!(
    /// Nachzug (Suche-Arten): a kind outside the type's allowed ones is refused with `search-kind-refused`; without a kind
    /// the default applies and the page comes; the default asked by name comes too.
    SearchKindAllowed,
    "records.search-kind",
    |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let shape = call(f.records().shape(&facts.data_type).await)?;
        let Some(outside) = SearchKind::ALL
            .into_iter()
            .find(|k| !shape.search.allowed().contains(k))
        else {
            return Err(Stop::Verdict(
                cx.missing(fs_conformance::Need::new("fixture.restricted-search")),
            ));
        };
        let mut q = RecordQuery::new(&facts.data_type, 10);
        q.search = Some(outside);
        refused(f.records().find(&q).await, RecordsCode::SearchKindRefused)?;
        q.search = None;
        call(f.records().find(&q).await)?;
        q.search = Some(shape.search.default_kind());
        call(f.records().find(&q).await)?;
        Ok(())
    }
);

rule!(
    /// Nachzug (`queryChanges`): from the current state nothing changed and the state stays; after another writer, `changes`
    /// from the old state either answers a new state or `cannot-calculate-changes`, never „nothing changed“ with the old one.
    ChangesHonest,
    "records.query-changes",
    |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let q = RecordQuery::new(&facts.data_type, 10);
        let old = call(f.records().find(&q).await)?.state;
        let same = call(f.records().changes(&q, &old).await)?;
        if same.new_state != old || !same.removed.is_empty() || !same.added.is_empty() {
            return Err(wrong(SuiteCode::CodeWrong, "unchanged", same.new_state));
        }
        if !f.touch() {
            return Err(Stop::Verdict(
                cx.missing(fs_conformance::Need::new("fixture.touch")),
            ));
        }
        match f.records().changes(&q, &old).await {
            Err(e) if e.code() == RecordsCode::CannotCalculateChanges => Ok(()),
            Ok(c) if c.new_state != old => Ok(()),
            Ok(c) => Err(wrong(SuiteCode::CodeWrong, "new-state", c.new_state)),
            Err(e) => Err(wrong(
                SuiteCode::CallFailed,
                "changes",
                crate::coded::message_key(e.code()),
            )),
        }
    }
);
