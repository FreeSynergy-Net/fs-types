//! K24-K26 and K30 of ENTWURF section 5: `MemoryRecords` and `RecordView`.
mod support;

use fs_records::{
    Condition, Flaw, HiddenReason, Limits, MemoryRecords, RecordQuery, RecordView, Records,
    RecordsCode, SearchPolicy, SortBy, path,
};
use fs_types::{BasicType, PageCursor};
use support::{S2, memory, note_type, shape};

fn ids(page: &fs_records::RecordPage) -> Vec<&str> {
    page.items.iter().map(|r| r.id.as_str()).collect()
}

async fn find(m: &MemoryRecords, q: &RecordQuery) -> fs_records::RecordPage {
    m.find(q).await.unwrap()
}

fn st(name: &str) -> Vec<String> {
    path(name)
}

/// K24: filter, sort, page, projection, cursor, scope, unreadable records.
#[tokio::test]
async fn k24_find() {
    let m = memory();
    let q = |n| RecordQuery::new("Note", n);
    assert_eq!(
        m.find(&RecordQuery::new("Nope", 5))
            .await
            .unwrap_err()
            .code(),
        RecordsCode::UnknownType
    );
    // an unreadable record is not there: not listed, not counted
    assert_eq!(ids(&find(&m, &q(10)).await), ["a", "b", "c", "d"]);
    let any = q(10).when(Condition::AnyOf {
        path: st("state"),
        values: vec!["todo".into(), "done".into()],
    });
    assert_eq!(ids(&find(&m, &any).await), ["a", "c"]);
    assert_eq!(
        ids(&find(&m, &q(10).when(Condition::IsEmpty { path: st("state") })).await),
        ["d"]
    );
    let range = q(10).when(Condition::Range {
        path: st("state"),
        from: Some("todo".into()),
        to: Some("doing".into()),
    });
    assert_eq!(ids(&find(&m, &range).await), ["a", "b"]);
    assert_eq!(
        ids(&find(&m, &q(10).when(Condition::Term("ALPHA".into()))).await),
        ["a"]
    );
    assert_eq!(
        ids(&find(&m, &q(10).when(Condition::Term("gamma".into()))).await),
        ["c"]
    );
    // a hidden field is never searched: the term "s-alpha" is only in the hidden field
    assert_eq!(
        ids(&find(&m, &q(10).when(Condition::Term("s-alpha".into()))).await),
        Vec::<&str>::new()
    );
    // a hidden field satisfies no condition, not even IsEmpty
    let empty_secret = q(10).when(Condition::IsEmpty { path: st("secret") });
    assert_eq!(ids(&find(&m, &empty_secret).await), ["b", "c", "d"]);
    let eq_secret = q(10).when(Condition::Equals {
        path: st("secret"),
        value: "s-alpha".into(),
    });
    assert_eq!(ids(&find(&m, &eq_secret).await), Vec::<&str>::new());
    // descending: the empty value stays last
    let desc = q(10).sort_by(SortBy::Field(st("state")), true);
    assert_eq!(ids(&find(&m, &desc).await), ["c", "b", "a", "d"]);
    let asc = q(10).sort_by(SortBy::Field(st("state")), false);
    assert_eq!(ids(&find(&m, &asc).await), ["a", "b", "c", "d"]);
}

/// K24 (second part): projection, pages and cursors, scope, unknown names, `get`, provider limits.
#[tokio::test]
async fn k24_pages_and_names() {
    let m = memory();
    let q = |n| RecordQuery::new("Note", n);
    // projection
    let only = find(&m, &q(10).only(&["title"])).await;
    assert!(
        only.items
            .iter()
            .all(|r| r.values.iter().all(|v| v.path == ["title"]) && r.hidden.is_empty())
    );
    // pages and cursors
    let p1 = find(&m, &q(3)).await;
    assert_eq!(ids(&p1), ["a", "b", "c"]);
    let p2 = find(&m, &q(3).after(p1.next.clone().unwrap())).await;
    assert_eq!(ids(&p2), ["d"]);
    assert!(p2.next.is_none());
    let bad = m
        .find(&q(3).after(PageCursor("zz".into())))
        .await
        .unwrap_err();
    assert_eq!(bad.code(), RecordsCode::CursorInvalid);
    // scope
    assert_eq!(ids(&find(&m, &q(10).scope(S2)).await), ["c"]);
    // unknown things
    assert_eq!(
        m.find(&q(10).sort_by(SortBy::Order("nope".into()), false))
            .await
            .unwrap_err()
            .code(),
        RecordsCode::UnknownOrder
    );
    assert_eq!(
        m.find(&q(10).grouped("nope")).await.unwrap_err().code(),
        RecordsCode::UnknownGrouping
    );
    let mismatch = q(10).when(Condition::Equals {
        path: st("state"),
        value: "nope".into(),
    });
    assert_eq!(
        m.find(&mismatch).await.unwrap_err().code(),
        RecordsCode::ValueMismatch
    );
    // get: missing and unreadable are the same answer
    assert!(m.get("Note", "zzz", &[]).await.unwrap().is_none());
    assert!(m.get("Note", "x", &[]).await.unwrap().is_none());
    assert!(m.get("Note", "b", &[]).await.unwrap().is_some());
    // limits come from the provider
    let small = MemoryRecords::new(note_type(), shape(SearchPolicy::complete_only()), vec![])
        .with_limits(Limits {
            page_max: 2,
            ..Limits::default()
        });
    assert_eq!(
        small.find(&q(3)).await.unwrap_err().code(),
        RecordsCode::PageSize
    );
}

/// K25: the groups of a choice grouping, left to right, the empty key last; a grouped page carries that key.
#[tokio::test]
async fn k25_groups() {
    let m = memory();
    let set = m.groups("Note", "state", None).await.unwrap();
    let keys: Vec<&str> = set.groups.iter().map(|g| g.key.as_str()).collect();
    assert_eq!(keys, ["todo", "doing", "done", ""]);
    let page = find(&m, &RecordQuery::new("Note", 10).grouped("state")).await;
    let of = |id: &str| {
        page.items
            .iter()
            .find(|r| r.id == id)
            .and_then(|r| r.group.clone())
    };
    assert_eq!(of("a").as_deref(), Some("todo"));
    assert_eq!(of("d").as_deref(), Some(""));
    assert!(
        page.items
            .iter()
            .all(|r| r.group.as_deref().is_some_and(|g| set.has(g)))
    );
    assert_eq!(
        m.groups("Note", "nope", None).await.unwrap_err().code(),
        RecordsCode::UnknownGrouping
    );
    assert!(
        find(&m, &RecordQuery::new("Note", 10))
            .await
            .items
            .iter()
            .all(|r| r.group.is_none())
    );
}

/// The generation moves with every write of anyone and is typed by the data type.
#[tokio::test]
async fn generation_moves() {
    let m = memory();
    let g = m.generation("Note", None).await.unwrap();
    m.touch();
    assert!(m.generation("Note", None).await.unwrap() > g);
    assert_eq!(
        m.generation("Nope", None).await.unwrap_err().code(),
        RecordsCode::UnknownType
    );
}

/// K26: `RecordView::value`, `is_hidden`, `check_hidden`.
#[tokio::test]
async fn k26_view_helpers() {
    let m = memory();
    let v: RecordView = m.get("Note", "a", &[]).await.unwrap().unwrap();
    assert_eq!(v.value("title").unwrap().values, ["Alpha"]);
    assert!(v.value("secret").is_none());
    assert!(v.is_hidden("secret"));
    assert!(!v.is_hidden("title"));
    assert_eq!(v.hidden[0].reason, HiddenReason::NoRight);
    assert!(v.check_hidden().is_ok());
    let mut bad = v.clone();
    bad.values.push(fs_records::FieldValue {
        path: st("secret"),
        basic: BasicType::Text,
        values: vec!["leak".into()],
    });
    assert_eq!(
        bad.check_hidden().unwrap_err().code(),
        RecordsCode::HiddenHasValue
    );
}

/// K30: the host defence: a provider that leaks a hidden value is caught by `Limits::check_page`.
#[tokio::test]
async fn k30_host_rejects_leaking_page() {
    let m = memory();
    m.flaw(Flaw::HiddenLeaks);
    let q = RecordQuery::new("Note", 10);
    let page = m.find(&q).await.unwrap();
    assert_eq!(
        Limits::default().check_page(&q, &page).unwrap_err().code(),
        RecordsCode::HiddenHasValue
    );
}
