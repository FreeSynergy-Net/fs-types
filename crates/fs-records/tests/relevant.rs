//! K43-K47 of ENTWURF section 15: the search kind `relevant`, estimates, snippets.
mod support;

use fs_records::{
    EstimateKind, FieldValue, Hidden, HiddenReason, Limits, RecordPage, RecordQuery, RecordView,
    Records, RecordsCode, SearchKind, SearchPolicy, Snippet,
};
use support::{code, has, memory_with};

fn page(
    total: Option<u64>,
    est: Option<u64>,
    kind: Option<EstimateKind>,
    snippets: Vec<Snippet>,
) -> RecordPage {
    let view = RecordView {
        id: "r".into(),
        revision: 1,
        values: vec![FieldValue {
            path: vec!["title".into()],
            basic: fs_types::BasicType::Text,
            values: vec!["x".into()],
        }],
        hidden: vec![Hidden {
            path: vec!["secret".into()],
            reason: HiddenReason::NoRight,
        }],
        lock: None,
        may: vec![],
        group: None,
    };
    RecordPage {
        items: vec![view],
        next: None,
        position: 0,
        total,
        state: String::new(),
        complete: true,
        total_estimated: est,
        estimate_kind: kind,
        snippets,
    }
}

/// K43: seven kinds, unique words, `relevant` is one of them.
#[test]
fn k43_kinds() {
    assert_eq!(SearchKind::ALL.len(), 7);
    let mut keys: Vec<&str> = SearchKind::ALL.iter().map(|k| k.key()).collect();
    assert!(keys.contains(&"relevant"));
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), 7);
}

/// K44: exact and estimated total together, an estimate without its kind, a kind without an estimate: `estimate-mixed`.
#[test]
fn k44_estimate() {
    let d = Limits::default();
    let q = RecordQuery::new("Note", 5);
    let k = Some(EstimateKind::AtLeast);
    for bad in [
        page(Some(1), Some(2), k, vec![]),
        page(None, Some(2), None, vec![]),
        page(None, None, k, vec![]),
    ] {
        assert_eq!(code(d.check_page(&q, &bad)).0, RecordsCode::EstimateMixed);
    }
    assert!(d.check_page(&q, &page(None, Some(2), k, vec![])).is_ok());
    assert!(d.check_page(&q, &page(Some(1), None, None, vec![])).is_ok());
}

/// K45: a snippet from a hidden field is refused as `hidden-has-value{path}`; one from a readable field passes.
#[test]
fn k45_snippet() {
    let d = Limits::default();
    let q = RecordQuery::new("Note", 5);
    let snip = |f: &str| Snippet {
        id: "r".into(),
        path: vec![f.into()],
        text: "t".into(),
    };
    let (c, p) = code(d.check_page(&q, &page(Some(1), None, None, vec![snip("secret")])));
    assert_eq!(c, RecordsCode::HiddenHasValue);
    assert!(has(&p, "path", "secret"));
    assert!(
        d.check_page(&q, &page(Some(1), None, None, vec![snip("title")]))
            .is_ok()
    );
}

/// K46: the double does not allow `relevant` for a `complete_only` type: refused (a calculation is never relevant).
#[tokio::test]
async fn k46_relevant_refused() {
    let m = memory_with(SearchPolicy::complete_only());
    let mut q = RecordQuery::new("Note", 5);
    q.search = Some(SearchKind::Relevant);
    let (c, p) = code(m.find(&q).await);
    assert_eq!(c, RecordsCode::SearchKindRefused);
    assert!(has(&p, "kind", "relevant"));
}

/// K47: a type that allows `relevant` gets `unsupported{what=relevant}` from the double (it computes no relevance).
#[tokio::test]
async fn k47_relevant_unsupported() {
    let policy = SearchPolicy::new(
        &[SearchKind::Complete, SearchKind::Relevant],
        SearchKind::Complete,
    )
    .unwrap();
    let m = memory_with(policy);
    let mut q = RecordQuery::new("Note", 5);
    q.search = Some(SearchKind::Relevant);
    let (c, p) = code(m.find(&q).await);
    assert_eq!(c, RecordsCode::Unsupported);
    assert!(has(&p, "what", "relevant"));
}
