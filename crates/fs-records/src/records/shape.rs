//! The shape of a listable type: its groupings and its orders (static, from `Records.shape`), and the groups of one grouping
//! now (dynamic, from `Records.groups`; board columns are data of the program and change at run time).
//!
//! Why not in `TypeCatalog`: `TypeCatalog` stays as it is (ADR 0066 §3, ADR 0045: an interface is a target, only appended);
//! groupings and orders belong to reading, so they come with `Records` (draft §4.4).
//!
//! Pattern: Value objects.

use super::search::SearchPolicy;

/// One way to group the records of a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupingDescription {
    /// Machine name, unique per type (`board`, `progress`, `priority`).
    pub name: String,
    /// Translation key (MF2), never a sentence.
    pub label: String,
    /// The `choice` field the groups are the keys of; `None` = the program computes the group (tasks `board`: columns of
    /// `board.columns`, a card's column by rule B7).
    pub field: Option<Vec<String>>,
    /// The record command that moves a record to another group (E4w); `None` = moving is `change` of `field`, or not possible.
    pub move_command: Option<String>,
}

/// One way to order the records of a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderDescription {
    /// Machine name, unique per type (`rank`, `due`, `priority`).
    pub name: String,
    /// Translation key.
    pub label: String,
    /// The field it sorts by; `None` = an order the program keeps itself (the order by hand, `rank`).
    pub field: Option<Vec<String>>,
    /// Whether people set it by hand (drag within a group); a hand order is changed by a command in E4w.
    pub manual: bool,
}

/// What a listable type offers beyond its fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    /// The data type.
    pub data_type: String,
    /// Its groupings.
    pub groupings: Vec<GroupingDescription>,
    /// Its orders.
    pub orders: Vec<OrderDescription>,
    /// Nachzug 17.11.: the search kinds it allows and its default (wire: `TypeDescription.searchKinds/searchDefault`).
    pub search: SearchPolicy,
}

/// One group now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    /// Stable key (a choice key, or the program's key: tasks `ColumnKey`).
    pub key: String,
    /// The title as the owner wrote it (user data: a column named „Testing“); `None` = the host shows the label key of the
    /// choice (`<field label>.<key>`).
    pub title: Option<String>,
    /// A limit of records that is a HINT, never a lock (tasks `wip`, b7 question 3).
    pub hint_limit: Option<u32>,
}

/// The groups of one grouping, left to right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupSet {
    /// The grouping.
    pub grouping: String,
    /// Changes when the set changes (tasks `ColumnSet::revision`).
    pub revision: u64,
    /// The groups; every record of a grouped page lies in exactly one of them (tasks rule B7: never invisible).
    pub groups: Vec<Group>,
}

impl GroupSet {
    /// Whether `key` is a group of the set.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.groups.iter().any(|g| g.key == key)
    }
}
