//! K39-K41 of ENTWURF section 14: search policy, search kinds in `find`, `changes`.
mod support;

use fs_records::{RecordQuery, Records, RecordsCode, SearchKind, SearchPolicy};
use support::{code, has, memory_with};

/// K39: a policy needs an allowed default and no kind twice; `complete_only` never opens; no choice gives the default.
#[test]
fn k39_policy() {
    let (c, p) = code(SearchPolicy::new(&[SearchKind::Complete], SearchKind::Open));
    assert_eq!(c, RecordsCode::SearchPolicyInvalid);
    assert!(has(&p, "kind", "open"));
    let (c, p) = code(SearchPolicy::new(
        &[SearchKind::Complete, SearchKind::Complete],
        SearchKind::Complete,
    ));
    assert_eq!(c, RecordsCode::SearchPolicyInvalid);
    assert!(has(&p, "kind", "complete"));
    let (c, p) = code(SearchPolicy::complete_only().choose(Some(SearchKind::Open)));
    assert_eq!(c, RecordsCode::SearchKindRefused);
    assert!(has(&p, "kind", "open"));
    assert_eq!(
        SearchPolicy::complete_only().choose(None).unwrap(),
        SearchKind::Complete
    );
    let two = SearchPolicy::new(
        &[SearchKind::Complete, SearchKind::Cancellable],
        SearchKind::Cancellable,
    )
    .unwrap();
    assert_eq!(two.choose(None).unwrap(), SearchKind::Cancellable);
    assert_eq!(
        two.choose(Some(SearchKind::Complete)).unwrap(),
        SearchKind::Complete
    );
}

/// K40: `find` with a kind the type does not allow (`live`) is refused; an allowed one works.
#[tokio::test]
async fn k40_find_kind() {
    let policy = SearchPolicy::new(
        &[SearchKind::Complete, SearchKind::Cancellable],
        SearchKind::Complete,
    )
    .unwrap();
    let m = memory_with(policy);
    let mut q = RecordQuery::new("Note", 5);
    q.search = Some(SearchKind::Live);
    let (c, p) = code(m.find(&q).await);
    assert_eq!(c, RecordsCode::SearchKindRefused);
    assert!(has(&p, "kind", "live"));
    q.search = Some(SearchKind::Cancellable);
    assert!(m.find(&q).await.is_ok());
}

/// K41: `changes` from the current state is empty with the same state; after a write by anyone, from the old state it is
/// `cannot-calculate-changes` (never a silent "nothing changed").
#[tokio::test]
async fn k41_changes() {
    let m = memory_with(SearchPolicy::complete_only());
    let q = RecordQuery::new("Note", 5);
    let state = m.find(&q).await.unwrap().state;
    let none = m.changes(&q, &state).await.unwrap();
    assert_eq!(
        (none.added.len(), none.removed.len(), none.new_state.clone()),
        (0, 0, state.clone())
    );
    m.touch();
    assert_eq!(
        m.changes(&q, &state).await.unwrap_err().code(),
        RecordsCode::CannotCalculateChanges
    );
}
