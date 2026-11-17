//! K9-K14, K19 of ENTWURF section 5, and the three demo mutants.
mod support;

use std::cell::Cell;

use fs_conformance::{Applied, ConformanceCode, Mutant, MutantId, Options, prove, run};
use support::*;

fn fresh() -> Box<dyn Counter> {
    Box::new(FakeCounter::new())
}

fn inert(
    id: &'static str,
    must: Vec<fs_conformance::RuleId>,
    applied: Applied,
) -> Vec<Box<dyn Mutant<dyn Counter>>> {
    vec![Box::new(Inert { id, must, applied })]
}

/// K9: the three demo mutants are caught; the proof is complete.
#[tokio::test]
async fn k9_demo_mutants_caught() {
    let r = prove(&DemoSuite, &fresh, &demo_mutants(), &Options::default()).await;
    assert!(r.is_proven(), "{:?}", r.failures);
    assert_eq!(
        r.caught,
        vec![
            MutantId::new("demo.sum-off-by-one"),
            MutantId::new("demo.no-limit"),
            MutantId::new("demo.zero-accepted")
        ]
    );
}

/// K10: a mutant whose rule stays green is not caught.
#[tokio::test]
async fn k10_not_caught() {
    let mut ms = demo_mutants();
    ms.push(Box::new(Inert {
        id: "demo.inert",
        must: vec![ADD_SUMS],
        applied: Applied::Yes,
    }));
    let r = prove(&DemoSuite, &fresh, &ms, &Options::default()).await;
    assert!(r.failures.contains(&frame(
        ConformanceCode::MutantNotCaught,
        &[("mutant", "demo.inert"), (RULE, "demo.add-sums")]
    )));
    assert!(!r.caught.contains(&MutantId::new("demo.inert")));
    assert!(!r.is_proven());
}

/// K11: a rule without a mutant is reported.
#[tokio::test]
async fn k11_rule_without_mutant() {
    let mut ms = demo_mutants();
    ms.pop();
    let r = prove(&DemoSuite, &fresh, &ms, &Options::default()).await;
    assert_eq!(
        r.failures,
        vec![frame(
            ConformanceCode::RuleWithoutMutant,
            &[(RULE, "demo.zero-refused")]
        )]
    );
}

/// K12: every mutant gets its own fresh fixture; a fresh fake is conformant afterwards.
#[tokio::test]
async fn k12_fresh_per_mutant() {
    let n = Cell::new(0);
    let counting = || -> Box<dyn Counter> {
        n.set(n.get() + 1);
        Box::new(FakeCounter::new())
    };
    let ms = demo_mutants();
    let r = prove(&DemoSuite, &counting, &ms, &Options::default()).await;
    assert!(r.is_proven());
    assert_eq!(n.get(), ms.len());
    let mut f = fresh();
    assert!(
        run(&DemoSuite, &mut *f, &Options::default())
            .await
            .is_conformant()
    );
}

/// K13: a mutant the fixture cannot apply is reported, not caught.
#[tokio::test]
async fn k13_not_applicable() {
    let r = prove(
        &DemoSuite,
        &fresh,
        &inert("demo.na", vec![ADD_SUMS], Applied::NotApplicable),
        &Options::default(),
    )
    .await;
    assert!(r.failures.contains(&frame(
        ConformanceCode::MutantNotApplicable,
        &[("mutant", "demo.na")]
    )));
    assert_eq!(r.caught, Vec::new());
}

/// K14: a mutant naming an unknown rule is reported, not caught.
#[tokio::test]
async fn k14_unknown_rule() {
    let ms = inert(
        "demo.u",
        vec![fs_conformance::RuleId::new("demo.nope")],
        Applied::Yes,
    );
    let r = prove(&DemoSuite, &fresh, &ms, &Options::default()).await;
    assert!(r.failures.contains(&frame(
        ConformanceCode::MutantUnknownRule,
        &[("mutant", "demo.u"), (RULE, "demo.nope")]
    )));
    assert_eq!(r.caught, Vec::new());
}

/// K19: a mutant with an empty `must_fail` is not caught.
#[tokio::test]
async fn k19_empty_must_fail() {
    let r = prove(
        &DemoSuite,
        &fresh,
        &inert("demo.e", vec![], Applied::Yes),
        &Options::default(),
    )
    .await;
    assert!(r.failures.contains(&frame(
        ConformanceCode::MutantNotCaught,
        &[("mutant", "demo.e")]
    )));
    assert_eq!(r.caught, Vec::new());
}
