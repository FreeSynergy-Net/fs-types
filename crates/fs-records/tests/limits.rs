//! K21-K23 of ENTWURF section 5, plus the setting rule O7 (ceiling is a parameter, a value above it is refused).
use std::future::{Future, pending, ready};
use std::pin::Pin;
use std::task::{Context, Poll};

use fs_records::coded::Param;
use fs_records::{
    CEILING_DEFAULT, FieldValue, Hidden, HiddenReason, Limits, RecordPage, RecordQuery, RecordView,
    RecordsCode, within,
};
use fs_types::{BasicType, PageCursor};

fn code<T: std::fmt::Debug>(
    r: Result<T, fs_records::coded::Coded<RecordsCode>>,
) -> (RecordsCode, Vec<(String, String)>) {
    let e = r.unwrap_err();
    let ps = e
        .params()
        .map(|(k, v)| {
            let v = match v {
                Param::Text(t) => t.clone(),
                Param::Number(n) => n.to_string(),
                _ => String::new(),
            };
            (k.to_owned(), v)
        })
        .collect();
    (e.code(), ps)
}

fn has(ps: &[(String, String)], k: &str, v: &str) -> bool {
    ps.iter().any(|(a, b)| a == k && b == v)
}

fn view(values: Vec<FieldValue>, hidden: Vec<Hidden>) -> RecordView {
    RecordView {
        id: "r".into(),
        revision: 1,
        values,
        hidden,
        lock: None,
        may: vec![],
        group: None,
    }
}

fn value(path: &str, text: String) -> FieldValue {
    FieldValue {
        path: vec![path.into()],
        basic: BasicType::Text,
        values: vec![text],
    }
}

/// K21: defaults, `clamped` under a ceiling, `meet`, `check_query` (size 0 / 201 / 17 conditions / 5 sort keys / 65 fields).
#[test]
fn k21_limits() {
    let d = Limits::default();
    assert_eq!(
        (d.page_max, d.conditions_max, d.sort_max, d.fields_max),
        (200, 16, 4, 64)
    );
    assert_eq!((d.value_bytes_max, d.call_ms), (64 * 1024, 5_000));
    let zero = Limits {
        page_max: 0,
        conditions_max: 0,
        sort_max: 0,
        fields_max: 0,
        value_bytes_max: 0,
        call_ms: 0,
    };
    let c = zero.clamped(CEILING_DEFAULT);
    assert_eq!((c.page_max, c.call_ms, c.value_bytes_max), (1, 1, 1));
    let big = Limits {
        page_max: u32::MAX,
        conditions_max: usize::MAX,
        sort_max: usize::MAX,
        fields_max: usize::MAX,
        value_bytes_max: usize::MAX,
        call_ms: u64::MAX,
    };
    assert_eq!(big.clamped(CEILING_DEFAULT), CEILING_DEFAULT);
    let small = Limits { page_max: 10, ..d };
    assert_eq!(big.clamped(small).page_max, 10);
    assert_eq!(d.meet(small).page_max, 10);
    assert_eq!(small.meet(d), d.meet(small));
    let q = |n: u32| RecordQuery::new("Note", n);
    assert!(d.check_query(&q(200)).is_ok());
    let (c0, p0) = code(d.check_query(&q(0)));
    assert_eq!(c0, RecordsCode::PageSize);
    assert!(has(&p0, "size", "0") && has(&p0, "max", "200"));
    assert_eq!(code(d.check_query(&q(201))).0, RecordsCode::PageSize);
    let cond = |n: usize| {
        (0..n).fold(q(1), |q, _| {
            q.when(fs_records::Condition::IsEmpty {
                path: vec!["title".into()],
            })
        })
    };
    assert!(d.check_query(&cond(16)).is_ok());
    let (c1, p1) = code(d.check_query(&cond(17)));
    assert_eq!(c1, RecordsCode::TooMany);
    assert!(has(&p1, "what", "conditions") && has(&p1, "max", "16"));
    let sorts = (0..5).fold(q(1), |q, _| {
        q.sort_by(fs_records::SortBy::Order("rank".into()), false)
    });
    assert!(has(&code(d.check_query(&sorts)).1, "what", "sort"));
    let names: Vec<String> = (0..65).map(|i| format!("f{i}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    assert!(has(
        &code(d.check_query(&q(1).only(&refs))).1,
        "what",
        "fields"
    ));
}

/// K21 (O7): the ceiling is a parameter; a value above it (or 0) is refused with the field name, never clamped silently.
#[test]
fn k21_setting_above_ceiling_is_refused() {
    let d = Limits::default();
    assert_eq!(d.admit(CEILING_DEFAULT).unwrap(), d);
    // an instance ceiling lower than the program's default ceiling: the default itself is then above it
    let tight = Limits {
        page_max: 50,
        ..CEILING_DEFAULT
    };
    let (c, p) = code(d.admit(tight));
    assert_eq!(c, RecordsCode::LimitRefused);
    assert!(has(&p, "what", "page-max") && has(&p, "max", "50"));
    // a value one above the ceiling is refused, the ceiling itself is accepted
    let at = Limits {
        call_ms: CEILING_DEFAULT.call_ms,
        ..d
    };
    assert!(at.admit(CEILING_DEFAULT).is_ok());
    let over = Limits {
        call_ms: CEILING_DEFAULT.call_ms + 1,
        ..d
    };
    assert!(has(&code(over.admit(CEILING_DEFAULT)).1, "what", "call-ms"));
    // a higher instance ceiling admits what the program's own ceiling would refuse (not compiled in)
    let high = Limits {
        page_max: 5_000,
        ..CEILING_DEFAULT
    };
    assert!(
        Limits {
            page_max: 5_000,
            ..d
        }
        .admit(high)
        .is_ok()
    );
    assert_eq!(
        Limits { page_max: 0, ..d }.admit(high).unwrap_err().code(),
        RecordsCode::LimitRefused
    );
}

/// K22: `check_page` (3 records at size 2; 65 537 bytes; exactly 65 536; hidden field with a value).
#[test]
fn k22_check_page() {
    let d = Limits::default();
    let q = RecordQuery::new("Note", 2);
    let three = RecordPage {
        items: vec![
            view(vec![], vec![]),
            view(vec![], vec![]),
            view(vec![], vec![]),
        ],
        next: Some(PageCursor("x".into())),
    };
    let (c, p) = code(d.check_page(&q, &three));
    assert_eq!(c, RecordsCode::PageTooLong);
    assert!(has(&p, "size", "2") && has(&p, "got", "3"));
    let big = RecordPage {
        items: vec![view(vec![value("body", "x".repeat(65_537))], vec![])],
        next: None,
    };
    let (c, p) = code(d.check_page(&q, &big));
    assert_eq!(c, RecordsCode::ValueTooLarge);
    assert!(has(&p, "path", "body") && has(&p, "bytes", "65537") && has(&p, "max", "65536"));
    let exact = RecordPage {
        items: vec![view(vec![value("body", "x".repeat(65_536))], vec![])],
        next: None,
    };
    assert!(d.check_page(&q, &exact).is_ok());
    let leak = RecordPage {
        items: vec![view(
            vec![value("secret", "s".into())],
            vec![Hidden {
                path: vec!["secret".into()],
                reason: HiddenReason::NoRight,
            }],
        )],
        next: None,
    };
    let (c, p) = code(d.check_page(&q, &leak));
    assert_eq!(c, RecordsCode::HiddenHasValue);
    assert!(has(&p, "path", "secret"));
}

struct Once(bool);
impl Future for Once {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// K23: `within`: work is polled first (ready work + ready timer = Ok); never-ready work + ready timer = `timeout{ms}`;
/// work that is pending once then ready, with a timer that never fires, = Ok.
#[tokio::test]
async fn k23_within() {
    assert_eq!(within(10, ready(7), ready(())).await.unwrap(), 7);
    let (c, p) = code(within(25, pending::<u8>(), ready(())).await);
    assert_eq!(c, RecordsCode::Timeout);
    assert!(has(&p, "ms", "25"));
    assert_eq!(
        within(
            10,
            async {
                Once(false).await;
                3
            },
            pending::<()>()
        )
        .await
        .unwrap(),
        3
    );
}
