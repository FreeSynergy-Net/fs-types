//! K31-K38 of ENTWURF section 14: the filter tree, `position`, unreadable records and hidden fields in filters.
mod support;

use fs_records::{Condition, Filter, RecordQuery, Records, RecordsCode, SortBy, path};
use support::{code, has, memory};

fn term(t: &str) -> Filter {
    Filter::Is(Condition::Term(t.into()))
}

fn ids(page: &fs_records::RecordPage) -> Vec<&str> {
    page.items.iter().map(|r| r.id.as_str()).collect()
}

/// K31: depth, nodes and conditions of a leaf and of `And[Is, Not[Is]]`.
#[test]
fn k31_measures() {
    assert_eq!(
        (
            term("a").depth(),
            term("a").nodes(),
            term("a").conditions().len()
        ),
        (1, 1, 1)
    );
    let tree = Filter::And(vec![term("a"), Filter::Not(vec![term("b")])]);
    assert_eq!(
        (tree.depth(), tree.nodes(), tree.conditions().len()),
        (3, 4, 2)
    );
}

/// K32: truth table of three operands (0 to 3 true) for And, Or, Not, Xor: Xor only at exactly one, Not only at none.
#[test]
fn k32_truth_table() {
    let leaf = |c: &Condition| matches!(c, Condition::Term(t) if t == "t");
    for k in 0..=3usize {
        let ops: Vec<Filter> = (0..3)
            .map(|i| term(if i < k { "t" } else { "f" }))
            .collect();
        assert_eq!(Filter::And(ops.clone()).holds(&leaf), k == 3, "and {k}");
        assert_eq!(Filter::Or(ops.clone()).holds(&leaf), k >= 1, "or {k}");
        assert_eq!(Filter::Not(ops.clone()).holds(&leaf), k == 0, "not {k}");
        assert_eq!(Filter::Xor(ops).holds(&leaf), k == 1, "xor {k}");
    }
}

/// K33: `check`: depth max+1, nodes max+1, empty operators, exactly at the limits.
#[test]
fn k33_check() {
    let deep = Filter::Not(vec![Filter::Not(vec![term("a")])]);
    let (c, p) = code(deep.check(2, 64));
    assert_eq!(c, RecordsCode::FilterTooDeep);
    assert!(has(&p, "depth", "3") && has(&p, "max", "2"));
    assert!(deep.check(3, 64).is_ok());
    let wide = Filter::And(vec![term("a"), term("b"), term("c")]);
    let (c, p) = code(wide.check(8, 3));
    assert_eq!(c, RecordsCode::TooMany);
    assert!(has(&p, "what", "nodes") && has(&p, "max", "3"));
    assert!(wide.check(8, 4).is_ok());
    for bad in [Filter::And(vec![]), Filter::Or(vec![Filter::Not(vec![])])] {
        let (c, p) = code(bad.check(8, 64));
        assert_eq!(c, RecordsCode::FilterInvalid);
        assert!(has(&p, "what", "empty-operator"));
    }
}

/// K34: `when` twice and `filtered` give one flat `And` with three operands.
#[test]
fn k34_when_is_flat_and() {
    let q = RecordQuery::new("Note", 5)
        .when(Condition::Term("a".into()))
        .when(Condition::Term("b".into()))
        .filtered(term("c"));
    assert_eq!(
        q.filter,
        Some(Filter::And(vec![term("a"), term("b"), term("c")]))
    );
}

/// K35: the tree of the suite rule `filter-tree` over the five bench records holds for n1, n3, n4.
#[cfg(feature = "conformance")]
#[tokio::test]
async fn k35_tree_on_bench() {
    use fs_records::suite::{Arrange, MemoryBench, RecordsFixture};
    let mut bench = MemoryBench::new();
    let facts = bench.arrange(Arrange::Five).unwrap();
    let v = &facts.values;
    let eq = |s: &String| {
        Filter::Is(Condition::Equals {
            path: path("title"),
            value: s.clone(),
        })
    };
    let xor = Filter::Xor(vec![
        Filter::Or(vec![eq(&v[0]), eq(&v[1])]),
        Filter::Is(Condition::AnyOf {
            path: path("title"),
            values: vec![v[0].clone(), v[2].clone()],
        }),
        Filter::Not(vec![eq(&v[1])]),
    ]);
    let tree = Filter::And(vec![
        xor,
        Filter::Not(vec![Filter::Is(Condition::IsEmpty {
            path: path("title"),
        })]),
    ]);
    let page = bench
        .memory
        .find(&RecordQuery::new("Note", 10).filtered(tree))
        .await
        .unwrap();
    assert_eq!(ids(&page), ["n1", "n3", "n4"]);
}

/// K36: `position` 2 with size 2 gives the third and fourth with `next`; behind the end nothing; `position` with a cursor is refused.
#[cfg(feature = "conformance")]
#[tokio::test]
async fn k36_position() {
    use fs_records::suite::{Arrange, MemoryBench, RecordsFixture};
    let mut bench = MemoryBench::new();
    bench.arrange(Arrange::Five).unwrap();
    let q = RecordQuery::new("Note", 2);
    let page = bench.memory.find(&q.clone().at(2)).await.unwrap();
    assert_eq!(
        (ids(&page), page.position, page.next.is_some()),
        (vec!["n2", "n3"], 2, true)
    );
    let past = bench.memory.find(&q.clone().at(9)).await.unwrap();
    assert_eq!((past.items.len(), past.next.is_none()), (0, true));
    let first = bench.memory.find(&q.clone()).await.unwrap();
    let both = q.at(1).after(first.next.unwrap());
    assert_eq!(
        bench.memory.find(&both).await.unwrap_err().code(),
        RecordsCode::PositionWithCursor
    );
}

/// K37: an unreadable record is not counted in `total`, takes no position, moves no sort place; `get` says `None`.
#[tokio::test]
async fn k37_unreadable_is_invisible() {
    let m = memory();
    let by_title = RecordQuery::new("Note", 10).sort_by(SortBy::Field(path("title")), false);
    let page = m.find(&by_title).await.unwrap();
    assert_eq!(page.total, Some(4));
    assert_eq!(ids(&page), ["a", "b", "d", "c"]);
    let at0 = m.find(&RecordQuery::new("Note", 1).at(0)).await.unwrap();
    assert_eq!(ids(&at0), ["a"]);
    assert!(m.get("Note", "0", &[]).await.unwrap().is_none());
    let past = m.find(&RecordQuery::new("Note", 10).at(4)).await.unwrap();
    assert_eq!((past.items.len(), past.total), (0, Some(4)));
}

/// K38: a leaf on a hidden field is always false: no value gives a hit, `Not` does not turn it into a probe.
#[tokio::test]
async fn k38_hidden_leaf_is_blind() {
    let m = memory();
    let find = |f: Filter| {
        let q = RecordQuery::new("Note", 10).filtered(f);
        let m = &m;
        async move {
            m.find(&q)
                .await
                .unwrap()
                .items
                .into_iter()
                .map(|r| r.id)
                .collect::<Vec<_>>()
        }
    };
    let eq = |v: &str| {
        Filter::Is(Condition::Equals {
            path: path("secret"),
            value: v.into(),
        })
    };
    assert_eq!(find(eq("s-alpha")).await, Vec::<String>::new());
    assert!(
        !find(Filter::Is(Condition::IsEmpty {
            path: path("secret")
        }))
        .await
        .contains(&"a".to_owned())
    );
    let real = find(Filter::Not(vec![eq("s-alpha")])).await;
    let other = find(Filter::Not(vec![eq("something-else")])).await;
    assert_eq!(real, other);
    assert_eq!(find(term("s-alpha")).await, Vec::<String>::new());
}
