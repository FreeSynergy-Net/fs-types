//! K1-K8, K15, K17, K18, K20 of ENTWURF section 5.
mod support;

use std::cell::Cell;
use std::rc::Rc;

use fs_conformance::{
    Coded, ConformanceCode, Need, Options, Param, Rule, RuleId, Suite, Verdict, run,
};
use support::*;

async fn run_with<G: Fn() -> Vec<Box<dyn Rule<dyn Counter>>>>(
    g: G,
    o: &Options,
) -> fs_conformance::Report {
    let mut f = FakeCounter::new();
    run(&Custom(g), &mut f as &mut dyn Counter, o).await
}

/// K1: the right demo fake passes every rule.
#[tokio::test]
async fn k1_conformant_demo() {
    let mut f = FakeCounter::new();
    let r = run(&DemoSuite, &mut f as &mut dyn Counter, &Options::default()).await;
    assert!(r.is_conformant(), "{:?}", r.failures);
    assert_eq!(r.checked, DemoSuite.rules().len());
    assert_eq!(r.skipped, Vec::new());
}

/// K2: a failure carries the code of the rule unchanged.
#[tokio::test]
async fn k2_code_unchanged() {
    let r = run_with(
        || {
            let c = Coded::new(DemoCode::TooMuch)
                .with_text("who", "bob")
                .with_number("limit", 10)
                .erase();
            vec![Box::new(Fixed("a.b", Verdict::Fail(c)))]
        },
        &Options::default(),
    )
    .await;
    let code = &r.failures[0].code;
    assert_eq!(code.prefix, "demo-error-");
    assert_eq!(code.name, "too-much");
    assert_eq!(
        code.params,
        vec![
            ("who".to_owned(), text("bob")),
            ("limit".to_owned(), Param::Number(10))
        ]
    );
    assert_eq!(code.message_key(), "demo-error-too-much");
}

/// K3: a skipped rule is listed with its need and not counted.
#[tokio::test]
async fn k3_skipped_not_checked() {
    let need = Need::new("fixture.clock");
    let r = run_with(
        || vec![Box::new(Fixed("a.b", Verdict::Skipped(need)))],
        &Options::default(),
    )
    .await;
    assert_eq!(r.checked, 0);
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].rule, RuleId::new("a.b"));
    assert_eq!(r.skipped[0].need, need);
}

/// K4: `Options.skip` keeps the body from running.
#[tokio::test]
async fn k4_option_skip() {
    let n = Rc::new(Cell::new(0));
    let n2 = Rc::clone(&n);
    let o = Options {
        skip: vec![RuleId::new("a.b")],
        ..Options::default()
    };
    let r = run_with(move || vec![Box::new(Count("a.b", Rc::clone(&n2)))], &o).await;
    assert_eq!(n.get(), 0);
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].need, Need::OPTION_SKIP);
    assert_eq!(r.checked, 0);
}

/// K5: missing capability is a failure, or a skip under `partial`.
#[tokio::test]
async fn k5_missing_capability() {
    let need = Need::new("fixture.clock");
    let make = move || -> Vec<Box<dyn Rule<dyn Counter>>> {
        vec![Box::new(WithCx("a.b", move |cx| cx.missing(need)))]
    };
    let full = run_with(make, &Options::default()).await;
    assert_eq!(full.failures.len(), 1);
    assert_eq!(
        full.failures[0].code,
        frame(
            ConformanceCode::CapabilityMissing,
            &[("need", "fixture.clock")]
        )
    );
    assert_eq!(full.checked, 1);
    let part = run_with(
        make,
        &Options {
            partial: true,
            ..Options::default()
        },
    )
    .await;
    assert_eq!(part.failures, Vec::new());
    assert_eq!(part.skipped.len(), 1);
    assert_eq!(part.skipped[0].need, need);
}

/// K6: one doubled id is one failure, and nothing runs.
#[tokio::test]
async fn k6_duplicate_once() {
    let n = Rc::new(Cell::new(0));
    let n2 = Rc::clone(&n);
    let r = run_with(
        move || {
            (0..3)
                .map(|_| Box::new(Count("dup.x", Rc::clone(&n2))) as Box<dyn Rule<dyn Counter>>)
                .collect()
        },
        &Options::default(),
    )
    .await;
    assert_eq!(r.failures.len(), 1);
    assert_eq!(
        r.failures[0].code,
        frame(ConformanceCode::DuplicateRule, &[(RULE, "dup.x")])
    );
    assert_eq!(r.checked, 0);
    assert_eq!(n.get(), 0);
}

/// K7: malformed ids are refused, a good one is accepted.
#[tokio::test]
async fn k7_id_form() {
    let long: &'static str = "a".repeat(65).leak();
    let bad: [&'static str; 6] = ["A.b", "", ".a", "a.", "a b", long];
    let r = run_with(
        move || {
            bad.iter()
                .map(|b| Box::new(Fixed(b, Verdict::Pass)) as Box<dyn Rule<dyn Counter>>)
                .collect()
        },
        &Options::default(),
    )
    .await;
    assert_eq!(r.failures.len(), 6);
    assert!(r.failures.iter().all(|f| f.code.name == "rule-id-invalid"));
    assert_eq!(r.checked, 0);
    let ok = run_with(
        || vec![Box::new(Fixed("a.b-c9", Verdict::Pass))],
        &Options::default(),
    )
    .await;
    assert!(ok.is_conformant());
    assert_eq!(ok.checked, 1);
}

/// K8: rules run in the order of `rules()`.
#[tokio::test]
async fn k8_order() {
    let mut f = FakeCounter::new();
    run(&DemoSuite, &mut f as &mut dyn Counter, &Options::default()).await;
    assert_eq!(
        f.calls(),
        ["demo.add-sums", "demo.limit", "demo.zero-refused"]
    );
}

/// K15: dyn objects of rule, suite and mutant build and run.
#[tokio::test]
async fn k15_dyn() {
    let rule: Box<dyn Rule<dyn Counter>> = Box::new(Fixed("a.b", Verdict::Pass));
    let mut f = FakeCounter::new();
    let cx = fs_conformance::Cx::new(Options::default());
    assert_eq!(
        rule.check(&mut f as &mut dyn Counter, &cx).await,
        Verdict::Pass
    );
    let suite: &dyn Suite<dyn Counter> = &DemoSuite;
    assert_eq!(suite.rules().len(), 3);
    let m: Box<dyn fs_conformance::Mutant<dyn Counter>> = demo_mutants().remove(0);
    assert_eq!(m.must_fail(), vec![ADD_SUMS]);
}

struct HoldsRc(Rc<Cell<u32>>);

#[async_trait::async_trait(?Send)]
impl Rule<dyn Counter> for HoldsRc {
    fn id(&self) -> RuleId {
        RuleId::new("a.rc")
    }
    async fn check(&self, _f: &mut Fx, _cx: &fs_conformance::Cx) -> Verdict {
        let held = Rc::clone(&self.0);
        tokio::task::yield_now().await;
        held.set(held.get() + 1);
        Verdict::Pass
    }
}

/// K17: a rule may hold an `Rc` across `.await` (`?Send`), on a current-thread runtime.
#[tokio::test]
async fn k17_not_send() {
    let n = Rc::new(Cell::new(0));
    let n2 = Rc::clone(&n);
    let r = run_with(
        move || vec![Box::new(HoldsRc(Rc::clone(&n2)))],
        &Options::default(),
    )
    .await;
    assert!(r.is_conformant());
    assert_eq!(n.get(), 1);
}

/// K18: the `Display` of a code, with and without parameters.
#[test]
fn k18_display() {
    let c = frame(ConformanceCode::DuplicateRule, &[(RULE, "a.b")]);
    assert_eq!(c.to_string(), "conformance-error-duplicate-rule (rule=a.b)");
    assert_eq!(
        Coded::new(ConformanceCode::RuleWithoutMutant)
            .erase()
            .to_string(),
        "conformance-error-rule-without-mutant"
    );
}

/// K20: `setup_failed` names the rule, then the cause and its parameters.
#[tokio::test]
async fn k20_setup_failed() {
    let r = run_with(
        || {
            vec![Box::new(WithCx("a.b", |cx| {
                let cause = Coded::new(DemoCode::TooMuch)
                    .with_number("limit", 10)
                    .erase();
                cx.setup_failed(RuleId::new("a.b"), cause)
            }))]
        },
        &Options::default(),
    )
    .await;
    let code = &r.failures[0].code;
    assert_eq!(code.name, "setup-failed");
    assert_eq!(
        code.params,
        vec![
            ("rule".to_owned(), text("a.b")),
            ("cause".to_owned(), text("demo-error-too-much")),
            ("limit".to_owned(), Param::Number(10)),
        ]
    );
}
