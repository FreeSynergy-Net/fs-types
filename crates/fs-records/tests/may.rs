//! Decision 85/86 (c0, 18.11.2026): `may` names only rights; limits of the provider are the provider's (Regel 57).
#![cfg(feature = "conformance")]
use fs_conformance::{Options, run};
use fs_records::suite::{Arrange, MemoryBench, RecordsFixture, RecordsSuite};
use fs_records::{Flaw, Limits};

/// K50: a record command she has the right to but the type marks `supported == false` is in `may`, also on a locked
/// record; one she has no right to is not. The type command `import` (not supported yet) is in `Shape::may`.
#[tokio::test]
async fn k50_may_names_only_rights() {
    let mut bench = MemoryBench::new();
    let facts = bench.arrange(Arrange::Rights).expect("rights");
    assert!(
        bench.types()[0]
            .commands
            .iter()
            .any(|c| c.name == facts.right_command && !c.supported)
    );
    for id in &facts.ids {
        let v = bench
            .records()
            .get(&facts.data_type, id, &[])
            .await
            .unwrap()
            .unwrap();
        assert_eq!(v.may, ["edit", "archive"], "{id}");
        assert!(v.may.contains(&facts.right_command));
        assert!(!v.may.contains(&facts.no_right_command));
    }
    assert!(
        bench
            .records()
            .get(&facts.data_type, "r2", &[])
            .await
            .unwrap()
            .unwrap()
            .lock
            .is_some()
    );
    let shape = bench.records().shape(&facts.data_type).await.unwrap();
    assert_eq!(shape.may, ["create", "import"]);
}

/// K51 (mutant `MayBySupported`): filtering `may` by `supported` drops `archive` and `import`; the rule is red, and so is
/// the direct check.
#[tokio::test]
async fn k51_may_filtered_by_supported_is_caught() {
    let mut bench = MemoryBench::new();
    bench.memory.flaw(Flaw::MayBySupported);
    let facts = bench.arrange(Arrange::Rights).expect("rights");
    let v = bench
        .records()
        .get(&facts.data_type, "r1", &[])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(v.may, ["edit"]);
    let shape = bench.records().shape(&facts.data_type).await.unwrap();
    assert_eq!(shape.may, ["create"]);
    let r = run(&RecordsSuite, &mut bench, &Options::default()).await;
    assert!(
        r.failures
            .iter()
            .any(|f| f.rule.as_str() == "records.may-rights-only"),
        "{:?}",
        r.failures
    );
}

/// K52 (Auflage 5): the host check of `estimate-honest` uses the limits the PROVIDER declares. A provider whose own
/// `value_bytes_max` is 2 but answers 5-byte values is caught; the default limits (larger) would let it pass.
#[tokio::test]
async fn k52_estimate_rule_uses_provider_limits() {
    let strict = Limits {
        value_bytes_max: 2,
        ..Limits::default()
    };
    let r = run(
        &RecordsSuite,
        &mut MemoryBench::with_limits(strict),
        &Options::default(),
    )
    .await;
    assert!(
        r.failures
            .iter()
            .any(|f| f.rule.as_str() == "records.estimate-honest"),
        "{:?}",
        r.failures
    );
    let r = run(&RecordsSuite, &mut MemoryBench::new(), &Options::default()).await;
    assert!(r.is_conformant(), "{:?}", r.failures);
}
