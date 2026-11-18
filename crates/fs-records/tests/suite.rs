//! K1 (suite codes), K27-K29 of ENTWURF section 5: the suite, its proof with the 26 bench mutants, `dyn` use.
#![cfg(feature = "conformance")]
use fs_conformance::{ErrorCode, Options, Suite, message_key, prove, run};
use fs_records::suite::{MemoryBench, RecordsFixture, RecordsSuite, SuiteCode, bench_mutants};
use fs_records::{RecordQuery, Records};

/// K1 (suite): every variant once, unique kebab names, `message_key` = prefix + name.
#[test]
fn k1_suite_codes() {
    let all = SuiteCode::ALL;
    let mut names: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), all.len());
    assert_eq!(all.len(), 15);
    for c in all {
        assert_eq!(
            message_key(*c),
            format!("records-suite-error-{}", c.as_str())
        );
        assert!(
            c.as_str()
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch == '-')
        );
    }
}

/// K27: the standard bench is conformant; 22 rules checked, none skipped.
#[tokio::test]
async fn k27_bench_is_conformant() {
    let r = run(&RecordsSuite, &mut MemoryBench::new(), &Options::default()).await;
    assert!(r.is_conformant(), "{:?}", r.failures);
    assert_eq!(r.checked, 22);
    assert_eq!(r.skipped.len(), 0);
}

/// K28: the suite is proven: each of the 26 bench mutants is caught, every rule has a mutant.
#[tokio::test]
async fn k28_suite_is_proven() {
    let r = prove(
        &RecordsSuite,
        &|| Box::new(MemoryBench::new()),
        &bench_mutants(),
        &Options::default(),
    )
    .await;
    assert!(r.is_proven(), "{:?}", r.failures);
    assert_eq!(r.caught.len(), 26);
}

/// K29: `dyn` use: the suite over `&mut dyn RecordsFixture`, as `&dyn Suite<MemoryBench>`, and `Box<dyn Records>`.
#[tokio::test]
async fn k29_dyn_use() {
    let mut bench = MemoryBench::new();
    let fixture: &mut dyn RecordsFixture = &mut bench;
    let r = run(&RecordsSuite, fixture, &Options::default()).await;
    assert!(r.is_conformant(), "{:?}", r.failures);
    let suite: &dyn Suite<MemoryBench> = &RecordsSuite;
    let r = run(suite, &mut MemoryBench::new(), &Options::default()).await;
    assert!(r.is_conformant(), "{:?}", r.failures);
    let boxed: Box<dyn Records> = Box::new(MemoryBench::new().memory);
    let page = boxed.find(&RecordQuery::new("Note", 5)).await.unwrap();
    assert_eq!(page.items.len(), 0);
}
