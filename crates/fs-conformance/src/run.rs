//! Running a suite, and proving it with mutants.
//!
//! Pattern: Template method (ids first, then every rule in order; for a proof: one fresh fixture per mutant).

use std::collections::BTreeSet;

use crate::check::{Cx, Rule, Verdict};
use crate::codes::{ConformanceCode, keys};
use crate::fault::Coded;
use crate::group::Suite;
use crate::ids::{Need, RuleId};
use crate::outcome::{Failure, MutantReport, Report, Skip};
use crate::proof::{Applied, Mutant};
use crate::switches::Options;

/// The id failures of a list of rules: malformed ids, then duplicates (each duplicate once). Empty = the ids are fine.
fn id_failures<F: ?Sized>(rules: &[Box<dyn Rule<F>>]) -> Vec<Failure> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for rule in rules {
        let id = rule.id();
        if !id.is_well_formed() {
            out.push(Failure {
                rule: id,
                code: Coded::new(ConformanceCode::RuleIdInvalid)
                    .with_text(keys::RULE, id.as_str())
                    .erase(),
            });
        } else if !seen.insert(id) && !out.iter().any(|f: &Failure| f.rule == id) {
            out.push(Failure {
                rule: id,
                code: Coded::new(ConformanceCode::DuplicateRule)
                    .with_text(keys::RULE, id.as_str())
                    .erase(),
            });
        }
    }
    out
}

/// Runs every rule of `suite` against `fixture`, in order. If a rule id is malformed or doubled, nothing runs and the
/// report holds only those failures (`checked == 0`).
pub async fn run<F: ?Sized>(suite: &dyn Suite<F>, fixture: &mut F, options: &Options) -> Report {
    let mut report = Report::new(suite.id());
    let rules = suite.rules();
    report.failures = id_failures(&rules);
    if !report.failures.is_empty() {
        return report;
    }
    let cx = Cx::new(options.clone());
    for rule in &rules {
        let id = rule.id();
        if options.skip.contains(&id) {
            report.skipped.push(Skip {
                rule: id,
                need: Need::OPTION_SKIP,
            });
            continue;
        }
        match rule.check(fixture, &cx).await {
            Verdict::Pass => report.checked += 1,
            Verdict::Fail(code) => {
                report.checked += 1;
                report.failures.push(Failure { rule: id, code });
            }
            Verdict::Skipped(need) => report.skipped.push(Skip { rule: id, need }),
        }
    }
    report
}

/// Proves `suite`: every rule is named by at least one mutant, and every mutant, switched on a FRESH fixture from
/// `fresh`, makes each of its `must_fail` rules fail. `options.skip` applies; a skipped `must_fail` rule counts as
/// not caught (a proof must not be skipped away).
pub async fn prove<F: ?Sized>(
    suite: &dyn Suite<F>,
    fresh: &dyn Fn() -> Box<F>,
    mutants: &[Box<dyn Mutant<F>>],
    options: &Options,
) -> MutantReport {
    let mut out = MutantReport {
        suite: suite.id(),
        caught: Vec::new(),
        failures: Vec::new(),
    };
    let rules = suite.rules();
    let ids = id_failures(&rules);
    if !ids.is_empty() {
        out.failures = ids.into_iter().map(|f| f.code).collect();
        return out;
    }
    let known: BTreeSet<RuleId> = rules.iter().map(|r| r.id()).collect();
    let named: BTreeSet<RuleId> = mutants.iter().flat_map(|m| m.must_fail()).collect();
    for id in known.difference(&named) {
        out.failures.push(
            Coded::new(ConformanceCode::RuleWithoutMutant)
                .with_text(keys::RULE, id.as_str())
                .erase(),
        );
    }
    for mutant in mutants {
        let mid = mutant.id();
        let wanted = mutant.must_fail();
        let unknown: Vec<RuleId> = wanted
            .iter()
            .copied()
            .filter(|r| !known.contains(r))
            .collect();
        for r in &unknown {
            out.failures.push(
                Coded::new(ConformanceCode::MutantUnknownRule)
                    .with_text(keys::MUTANT, mid.as_str())
                    .with_text(keys::RULE, r.as_str())
                    .erase(),
            );
        }
        let mut fixture = fresh();
        if mutant.apply(&mut *fixture) == Applied::NotApplicable {
            out.failures.push(
                Coded::new(ConformanceCode::MutantNotApplicable)
                    .with_text(keys::MUTANT, mid.as_str())
                    .erase(),
            );
            continue;
        }
        let report = run(suite, &mut *fixture, options).await;
        if wanted.is_empty() {
            // A mutant that names no rule proves nothing.
            out.failures.push(
                Coded::new(ConformanceCode::MutantNotCaught)
                    .with_text(keys::MUTANT, mid.as_str())
                    .erase(),
            );
            continue;
        }
        let mut all = unknown.is_empty();
        for r in wanted.iter().filter(|r| known.contains(r)) {
            if !report.failed(*r) {
                all = false;
                out.failures.push(
                    Coded::new(ConformanceCode::MutantNotCaught)
                        .with_text(keys::MUTANT, mid.as_str())
                        .with_text(keys::RULE, r.as_str())
                        .erase(),
                );
            }
        }
        if all {
            out.caught.push(mid);
        }
    }
    out
}
