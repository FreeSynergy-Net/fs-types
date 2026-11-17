//! `MemoryBench`: the standard fixture, a [`MemoryRecords`] of the demo type `Note`, seeded per situation.
//!
//! Pattern: Fixture over a Fake object.

use fs_types::{BasicType, CommandDescription, FieldDescription, TypeDescription};

use super::fixture::{Arrange, Facts, RecordsFixture};
use crate::records::{
    Group, GroupingDescription, LockKind, MemoryRecords, OrderDescription, RecordLock, Records,
    Row, Shape, path,
};

const NOTE: &str = "Note";
const SCOPE: &str = "urn:uuid:00000000-0000-7000-8000-000000000001";

/// The demo type: a title, a state, a secret, a body.
fn note_type() -> TypeDescription {
    TypeDescription::new(NOTE, "fs-demo", "AS2 Note", "note")
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
        .field(FieldDescription::new("body", BasicType::MultilineText).label("note.body"))
        .command(CommandDescription::standard_find("note.find", NOTE))
}

fn note_shape() -> Shape {
    Shape {
        data_type: NOTE.into(),
        groupings: vec![
            GroupingDescription {
                name: "state".into(),
                label: "note.state".into(),
                field: Some(path("state")),
                move_command: None,
            },
            GroupingDescription {
                name: "lane".into(),
                label: "note.lane".into(),
                field: None,
                move_command: None,
            },
        ],
        orders: vec![
            OrderDescription {
                name: "manual".into(),
                label: "note.manual".into(),
                field: None,
                manual: true,
            },
            OrderDescription {
                name: "state".into(),
                label: "note.state".into(),
                field: Some(path("state")),
                manual: false,
            },
        ],
    }
}

/// The bench.
pub struct MemoryBench {
    /// The provider; mutants switch its flaws.
    pub memory: MemoryRecords,
}

impl MemoryBench {
    /// A fresh bench (no records, no flaws).
    #[must_use]
    pub fn new() -> Self {
        let lanes = ["a", "b"].map(|k| Group {
            key: k.into(),
            title: Some(k.to_uppercase()),
            hint_limit: None,
        });
        Self {
            memory: MemoryRecords::new(note_type(), note_shape(), lanes.to_vec()),
        }
    }
}

impl Default for MemoryBench {
    fn default() -> Self {
        Self::new()
    }
}

fn row(id: &str, title: &str, state: &str, lane: &str) -> Row {
    Row::new(id)
        .in_scope(SCOPE)
        .set("title", &[title])
        .set("state", &[state])
        .in_group(lane)
}

impl RecordsFixture for MemoryBench {
    fn records(&self) -> &dyn Records {
        &self.memory
    }

    fn types(&self) -> Vec<TypeDescription> {
        vec![note_type()]
    }

    fn arrange(&mut self, situation: Arrange) -> Option<Facts> {
        let m = &self.memory;
        m.clear();
        let ids: Vec<String> = match situation {
            Arrange::Five => {
                let titles = ["alpha", "beta", "gamma", "delta", "epsilon"];
                for (i, t) in titles.iter().enumerate() {
                    let state = if i % 2 == 0 { "todo" } else { "doing" };
                    m.push(row(
                        &format!("n{i}"),
                        t,
                        state,
                        if i < 3 { "a" } else { "b" },
                    ));
                }
                (0..5).map(|i| format!("n{i}")).collect()
            }
            Arrange::Ties => {
                for id in ["t3", "t2", "t1"] {
                    m.push(row(id, "same", "todo", "a"));
                }
                vec!["t1".into(), "t2".into(), "t3".into()]
            }
            Arrange::Hidden => {
                m.push(
                    row("h1", "open", "todo", "a")
                        .set("secret", &["s3cr3t"])
                        .hiding("secret"),
                );
                vec!["h1".into()]
            }
            Arrange::Locked => {
                let lock = RecordLock {
                    kind: LockKind::Claimed,
                    by: Some("urn:uuid:00000000-0000-7000-8000-0000000000aa".into()),
                    until_ms: Some(1_800_000_000_000),
                    reason: None,
                };
                m.push(row("l1", "held", "doing", "a").locked(lock));
                vec!["l1".into()]
            }
            Arrange::Unreadable => {
                m.push(row("u1", "not yours", "todo", "a").unreadable());
                m.push(row("u2", "yours", "todo", "b"));
                vec!["u1".into(), "u2".into()]
            }
        };
        Some(Facts {
            data_type: NOTE.into(),
            scope: Some(SCOPE.into()),
            ids,
            text: "title".into(),
            probe: "alpha".into(),
            hidden: "secret".into(),
            grouping: "lane".into(),
        })
    }

    fn touch(&mut self) -> bool {
        self.memory.touch();
        true
    }
}
