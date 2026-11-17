//! Shared by the tests: the demo type `Note`, its shape, a small memory with one unreadable record (`0`, `x`) and one hidden
//! field (`secret` of `a`), and helpers to read the parameters of a code.
#![allow(dead_code)]

use fs_records::coded::{Coded, Param};
use fs_records::{
    Group, GroupingDescription, MemoryRecords, OrderDescription, RecordsCode, Row, SearchPolicy,
    Shape, path,
};
use fs_types::{BasicType, CommandDescription, FieldDescription, TypeDescription};

pub const S1: &str = "list-1";
pub const S2: &str = "list-2";

pub fn note_type() -> TypeDescription {
    TypeDescription::new("Note", "fs-demo", "AS2 Note", "note")
        .field(
            FieldDescription::new("title", BasicType::Text)
                .required()
                .label("note.title"),
        )
        .field(
            FieldDescription::new("state", BasicType::Choice)
                .choices(&["todo", "doing", "done"])
                .label("note.state"),
        )
        .field(FieldDescription::new("secret", BasicType::Text).label("note.secret"))
        .command(CommandDescription::standard_find("note.find", "Note"))
}

pub fn shape(search: SearchPolicy) -> Shape {
    Shape {
        data_type: "Note".into(),
        groupings: vec![GroupingDescription {
            name: "state".into(),
            label: "note.state".into(),
            field: Some(path("state")),
            move_command: None,
        }],
        orders: vec![OrderDescription {
            name: "manual".into(),
            label: "note.manual".into(),
            field: None,
            manual: true,
        }],
        search,
    }
}

pub fn memory_with(search: SearchPolicy) -> MemoryRecords {
    let m = MemoryRecords::new(note_type(), shape(search), Vec::<Group>::new());
    m.push(
        Row::new("a")
            .in_scope(S1)
            .set("title", &["Alpha"])
            .set("state", &["todo"])
            .set("secret", &["s-alpha"])
            .hiding("secret"),
    );
    m.push(
        Row::new("b")
            .in_scope(S1)
            .set("title", &["Beta"])
            .set("state", &["doing"]),
    );
    m.push(
        Row::new("c")
            .in_scope(S2)
            .set("title", &["gamma"])
            .set("state", &["done"]),
    );
    m.push(Row::new("d").in_scope(S1).set("title", &["Delta"]));
    m.push(
        Row::new("x")
            .in_scope(S1)
            .set("title", &["Hidden record"])
            .unreadable(),
    );
    m
}

pub fn memory() -> MemoryRecords {
    memory_with(SearchPolicy::complete_only())
}

/// The code and its parameters of a failed call.
pub fn code<T: std::fmt::Debug>(
    r: Result<T, Coded<RecordsCode>>,
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

pub fn has(ps: &[(String, String)], k: &str, v: &str) -> bool {
    ps.iter().any(|(a, b)| a == k && b == v)
}
