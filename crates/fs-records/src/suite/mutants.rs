//! The mutants of the bench: one [`Flaw`] each, with the rules it must break. Every rule of the suite is named at least once.
//!
//! Pattern: Strategy (`Mutant` over [`MemoryBench`]); ids `records-bench.<flaw>` (well-formed, unique; review A3).

use fs_conformance::{Applied, Mutant, MutantId, RuleId};

use super::bench::MemoryBench;
use crate::records::Flaw;

/// One flaw switched on, and the rules that must then fail.
#[derive(Clone, Debug)]
pub struct FlawMutant {
    flaw: Flaw,
    id: &'static str,
    rules: &'static [&'static str],
}

impl Mutant<MemoryBench> for FlawMutant {
    fn id(&self) -> MutantId {
        MutantId::new(self.id)
    }

    fn must_fail(&self) -> Vec<RuleId> {
        self.rules.iter().copied().map(RuleId::new).collect()
    }

    fn apply(&self, fixture: &mut MemoryBench) -> Applied {
        fixture.memory.flaw(self.flaw);
        Applied::Yes
    }
}

/// The 13 mutants of the bench.
#[must_use]
pub fn bench_mutants() -> Vec<Box<dyn Mutant<MemoryBench>>> {
    let m = |flaw: Flaw, id: &'static str, rules: &'static [&'static str]| {
        Box::new(FlawMutant { flaw, id, rules }) as Box<dyn Mutant<MemoryBench>>
    };
    vec![
        m(
            Flaw::CursorIgnored,
            "records-bench.cursor-ignored",
            &["records.pages"],
        ),
        m(
            Flaw::SizeUnchecked,
            "records-bench.size-unchecked",
            &["records.page-size"],
        ),
        m(
            Flaw::UnknownFieldIgnored,
            "records-bench.unknown-field-ignored",
            &["records.unknown-field"],
        ),
        m(
            Flaw::ConditionsIgnored,
            "records-bench.conditions-ignored",
            &["records.filter"],
        ),
        m(
            Flaw::TiesUnordered,
            "records-bench.ties-unordered",
            &["records.ties"],
        ),
        m(
            Flaw::HiddenLeaks,
            "records-bench.hidden-leaks",
            &["records.hidden"],
        ),
        m(
            Flaw::LockDropped,
            "records-bench.lock-dropped",
            &["records.lock"],
        ),
        m(
            Flaw::GroupInvented,
            "records-bench.group-invented",
            &["records.groups-cover"],
        ),
        m(
            Flaw::GroupingIgnored,
            "records-bench.grouping-ignored",
            &["records.unknown-grouping"],
        ),
        m(
            Flaw::MissingIsError,
            "records-bench.missing-is-error",
            &["records.missing"],
        ),
        m(
            Flaw::GenerationFrozen,
            "records-bench.generation-frozen",
            &["records.generation"],
        ),
        m(
            Flaw::ShapeLies,
            "records-bench.shape-lies",
            &["records.shape"],
        ),
        m(
            Flaw::UnreadableShown,
            "records-bench.unreadable-shown",
            &["records.unreadable"],
        ),
    ]
}
