//! Rules of reading: pages, limits, unknown names, conditions, ties, groups, generation, shape.
//!
//! Pattern: one rule per behaviour (Strategy), written with `rule!`.

use std::collections::BTreeSet;

use super::check::{Stop, arrange, call, refused, rule, wrong};
use super::codes::{SuiteCode, keys};
use super::fixture::Arrange;
use crate::records::{Condition, RecordQuery, RecordsCode, SortBy, path};
use fs_conformance::Coded;

fn query(facts: &super::Facts, size: u32) -> RecordQuery {
    let q = RecordQuery::new(&facts.data_type, size);
    match &facts.scope {
        Some(s) => q.scope(s),
        None => q,
    }
}

rule!(
    /// Paging by 2 gives every seeded record exactly once, no page longer than asked, the last without `next`.
    Pages, "records.pages", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let mut seen: Vec<String> = Vec::new();
        let mut q = query(&facts, 2);
        for _ in 0..10 {
            let page = call(f.records().find(&q).await)?;
            if page.items.len() > 2 {
                return Err(wrong(SuiteCode::PageWrong, "2", page.items.len().to_string()));
            }
            seen.extend(page.items.into_iter().map(|r| r.id));
            match page.next {
                Some(c) => q = q.after(c),
                None => break,
            }
        }
        let unique: BTreeSet<&String> = seen.iter().collect();
        let want: BTreeSet<&String> = facts.ids.iter().collect();
        if unique.len() != seen.len() || unique != want {
            return Err(wrong(SuiteCode::PageWrong, facts.ids.join(","), seen.join(",")));
        }
        Ok(())
    }
);

rule!(
    /// A page size of 0 or above the provider's limit is refused with `page-size`.
    PageSize, "records.page-size", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let max = f.records().limits().page_max;
        refused(f.records().find(&query(&facts, max.saturating_add(1))).await, RecordsCode::PageSize)?;
        refused(f.records().find(&query(&facts, 0)).await, RecordsCode::PageSize)
    }
);

rule!(
    /// A condition on a field the type does not have is refused with `unknown-field`, never ignored.
    UnknownField, "records.unknown-field", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let q = query(&facts, 10).when(Condition::Equals { path: path("noSuchFieldX"), value: "x".into() });
        refused(f.records().find(&q).await, RecordsCode::UnknownField)
    }
);

rule!(
    /// `Equals` on a text field answers exactly the record holding the value.
    Filter, "records.filter", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let q = query(&facts, 10).when(Condition::Equals { path: path(&facts.text), value: facts.probe.clone() });
        let page = call(f.records().find(&q).await)?;
        let got: Vec<String> = page.items.into_iter().map(|r| r.id).collect();
        let want: Vec<String> = facts.ids.iter().take(1).cloned().collect();
        if got != want {
            return Err(wrong(SuiteCode::FilterWrong, want.join(","), got.join(",")));
        }
        Ok(())
    }
);

rule!(
    /// Equal sort keys are ordered by id, ascending (stable pages).
    Ties, "records.ties", |f, cx| {
        let facts = arrange(f, cx, Arrange::Ties)?;
        let q = query(&facts, 10).sort_by(SortBy::Field(path(&facts.text)), false);
        let page = call(f.records().find(&q).await)?;
        let got: Vec<String> = page.items.into_iter().map(|r| r.id).collect();
        if got != facts.ids {
            return Err(wrong(SuiteCode::OrderWrong, facts.ids.join(","), got.join(",")));
        }
        Ok(())
    }
);

rule!(
    /// Grouped, every record lies in a group of the set (never invisible).
    GroupsCover, "records.groups-cover", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let set = call(f.records().groups(&facts.data_type, &facts.grouping, facts.scope.as_deref()).await)?;
        let page = call(f.records().find(&query(&facts, 10).grouped(&facts.grouping)).await)?;
        for r in page.items {
            let key = r.group.unwrap_or_default();
            if !set.has(&key) {
                return Err(Stop::Fail(
                    Coded::new(SuiteCode::GroupOutside).with_text(keys::ID, r.id).with_text(keys::KEY, key),
                ));
            }
        }
        Ok(())
    }
);

rule!(
    /// An unknown grouping is refused by `find` and by `groups` with `unknown-grouping`.
    UnknownGrouping, "records.unknown-grouping", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let q = query(&facts, 10).grouped("no-such-grouping");
        refused(f.records().find(&q).await, RecordsCode::UnknownGrouping)?;
        refused(f.records().groups(&facts.data_type, "no-such-grouping", None).await, RecordsCode::UnknownGrouping)
    }
);

rule!(
    /// After another writer changed a record, the generation differs.
    GenerationMoves, "records.generation", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let before = call(f.records().generation(&facts.data_type, facts.scope.as_deref()).await)?;
        if !f.touch() {
            return Err(Stop::Verdict(cx.missing(fs_conformance::Need::new("fixture.touch"))));
        }
        let after = call(f.records().generation(&facts.data_type, facts.scope.as_deref()).await)?;
        if before == after {
            return Err(Stop::Fail(Coded::new(SuiteCode::GenerationStill)));
        }
        Ok(())
    }
);

rule!(
    /// Every grouping and order of the shape that names a field names one the type catalog has.
    ShapeTrue, "records.shape", |f, cx| {
        let facts = arrange(f, cx, Arrange::Five)?;
        let shape = call(f.records().shape(&facts.data_type).await)?;
        let types = f.types();
        let Some(ty) = types.iter().find(|t| t.name == facts.data_type) else {
            return Err(Stop::Fail(Coded::new(SuiteCode::ShapeWrong).with_text(keys::PATH, facts.data_type.clone())));
        };
        let named = shape.groupings.iter().filter_map(|g| g.field.clone()).chain(shape.orders.iter().filter_map(|o| o.field.clone()));
        for p in named {
            let ok = p.len() == 1 && ty.find_field(&p[0]).is_some();
            if !ok {
                return Err(Stop::Fail(Coded::new(SuiteCode::ShapeWrong).with_text(keys::PATH, p.join("/"))));
            }
        }
        Ok(())
    }
);
