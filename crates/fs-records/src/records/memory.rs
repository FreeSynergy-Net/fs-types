//! `MemoryRecords`: one data type in memory, for tests, demos and the workbench adapters of F3 (as `MemoryBoard` in
//! fs-components `board/source.rs`). It keeps rights per record and field, so the rights rules can be tested without a server.
//!
//! [`Flaw`] switches on one wrong behaviour each; they exist only so that the suite can be proven (`fs_conformance::prove`):
//! a real provider has none of them.
//!
//! Pattern: Fake object (Strategy behind [`Records`]) + switches for mutants (the form of fs-tasks `mutation.rs`).

use std::cell::{Cell, RefCell};
use std::future::ready;

use fs_types::TypeDescription;

use super::api::{Pending, Records};
use super::codes::{RecordsCode, keys};
use super::limits::Limits;
use super::memory_find;
use super::memory_row::Row;
use super::query::path;
use super::search::QueryChanges;
use super::shape::{Group, GroupSet, GroupingDescription, Shape};
use crate::coded::Coded;

/// One wrong behaviour each (only for proving the suite).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Flaw {
    /// Every page starts at the beginning.
    CursorIgnored,
    /// No page size check.
    SizeUnchecked,
    /// Unknown fields pass.
    UnknownFieldIgnored,
    /// Conditions are not applied.
    ConditionsIgnored,
    /// Ties are not broken by id.
    TiesUnordered,
    /// A hidden field carries its value.
    HiddenLeaks,
    /// Locks are not answered.
    LockDropped,
    /// Records get a group that is not in the set.
    GroupInvented,
    /// An unknown grouping gives an empty set instead of a code.
    GroupingIgnored,
    /// `get` of a missing record is an error instead of `None`.
    MissingIsError,
    /// The generation never moves.
    GenerationFrozen,
    /// The shape names a grouping over a field the type does not have.
    ShapeLies,
    /// Records the person may not read are answered.
    UnreadableShown,
    /// Nachzug 17.11.: `XOR` as odd parity instead of exactly one.
    XorParity,
    /// Nachzug: `NOT` dropped (its operands count as `OR`).
    NotDropped,
    /// Nachzug: filter depth and node limits not checked.
    DepthUnchecked,
    /// Nachzug: `total` counts records she may not read.
    TotalCountsUnreadable,
    /// Nachzug: conditions see the values of hidden fields.
    HiddenFilterSees,
    /// Nachzug (Suche-Arten): a search kind outside the allowed ones is taken.
    SearchKindIgnored,
    /// Nachzug: the query state never changes.
    StateFrozen,
    /// Nachzug 2: the exact total is also given as estimate (estimate shown as exact).
    EstimateAsExact,
    /// Nachzug 2: `relevant` is answered as unsupported before the policy is asked.
    RelevantBeforePolicy,
    /// Decision 85: `may` (of a record and of the type) is filtered by `supported`: a command she has the right to but the
    /// program does not do yet is dropped.
    MayBySupported,
    /// Decision 85: `may` of a record names every record command of the type, whatever her rights are.
    MayIgnoresRights,
    /// Decision 85: a locked record has an empty `may` (the lock is taken for a missing right).
    MayDroppedWhenLocked,
    /// Decision 86: `Shape::may` names every command of the type as a whole, whatever her rights are.
    TypeMayIgnoresRights,
}

/// One data type in memory.
pub struct MemoryRecords {
    pub(super) description: TypeDescription,
    pub(super) shape: Shape,
    pub(super) groups: Vec<Group>,
    pub(super) rows: RefCell<Vec<Row>>,
    generation: Cell<u64>,
    pub(super) limits: Limits,
    flaws: RefCell<Vec<Flaw>>,
}

impl MemoryRecords {
    /// Empty, with the type, its shape and the groups of its program-computed grouping (if any).
    #[must_use]
    pub fn new(description: TypeDescription, shape: Shape, groups: Vec<Group>) -> Self {
        Self {
            description,
            shape,
            groups,
            rows: RefCell::new(Vec::new()),
            generation: Cell::new(0),
            limits: Limits::default(),
            flaws: RefCell::new(Vec::new()),
        }
    }
    /// Other limits.
    #[must_use]
    pub fn with_limits(mut self, limits: Limits) -> Self {
        self.limits = limits;
        self
    }
    /// Stores a record (as a writer; the generation moves).
    pub fn push(&self, row: Row) {
        self.rows.borrow_mut().push(row);
        self.touch();
    }
    /// Removes every record.
    pub fn clear(&self) {
        self.rows.borrow_mut().clear();
        self.touch();
    }
    /// Another writer changed something.
    pub fn touch(&self) {
        self.generation.set(self.generation.get() + 1);
    }
    /// Switches a wrong behaviour on.
    pub fn flaw(&self, flaw: Flaw) {
        self.flaws.borrow_mut().push(flaw);
    }
    /// The query state: the generation as text (the double has one state for every query).
    pub(super) fn state(&self) -> String {
        if self.has(Flaw::StateFrozen) {
            "g:0".into()
        } else {
            format!("g:{}", self.generation.get())
        }
    }
    pub(super) fn has(&self, flaw: Flaw) -> bool {
        self.flaws.borrow().contains(&flaw)
    }
    /// Flaw `MayBySupported`: keeps the names of commands the description marks as supported.
    pub(super) fn only_supported(&self, names: &[String]) -> Vec<String> {
        names
            .iter()
            .filter(|n| {
                self.description
                    .commands
                    .iter()
                    .any(|c| &c.name == *n && c.supported)
            })
            .cloned()
            .collect()
    }
    pub(super) fn type_check(&self, data_type: &str) -> Result<(), Coded<RecordsCode>> {
        if data_type == self.description.name {
            Ok(())
        } else {
            Err(Coded::new(RecordsCode::UnknownType).with_text(keys::TYPE, data_type))
        }
    }
    pub(super) fn grouping(&self, name: &str) -> Result<&GroupingDescription, Coded<RecordsCode>> {
        self.shape
            .groupings
            .iter()
            .find(|g| g.name == name)
            .ok_or_else(|| {
                Coded::new(RecordsCode::UnknownGrouping)
                    .with_text(keys::TYPE, self.description.name.as_str())
                    .with_text(keys::GROUPING, name)
            })
    }
    fn group_set(&self, data_type: &str, name: &str) -> Result<GroupSet, Coded<RecordsCode>> {
        self.type_check(data_type)?;
        let revision = self.generation.get();
        let grouping = match self.grouping(name) {
            Ok(g) => g,
            Err(_) if self.has(Flaw::GroupingIgnored) => {
                return Ok(GroupSet {
                    grouping: name.to_owned(),
                    revision,
                    groups: Vec::new(),
                });
            }
            Err(e) => return Err(e),
        };
        let groups = match &grouping.field {
            None => self.groups.clone(),
            Some(p) => memory_find::choice_groups(self, p),
        };
        Ok(GroupSet {
            grouping: name.to_owned(),
            revision,
            groups,
        })
    }
    fn shape_now(&self, data_type: &str) -> Result<Shape, Coded<RecordsCode>> {
        self.type_check(data_type)?;
        let mut shape = self.shape.clone();
        if self.has(Flaw::MayBySupported) {
            shape.may = self.only_supported(&shape.may);
        }
        if self.has(Flaw::TypeMayIgnoresRights) {
            shape.may = self
                .description
                .type_commands()
                .iter()
                .map(|c| c.name.clone())
                .collect();
        }
        if self.has(Flaw::ShapeLies) {
            let field = Some(path("no-such-field"));
            shape.groupings.push(GroupingDescription {
                name: "lie".into(),
                label: "lie".into(),
                field,
                move_command: None,
            });
        }
        Ok(shape)
    }
}

impl Records for MemoryRecords {
    fn limits(&self) -> Limits {
        self.limits
    }
    fn shape(&self, data_type: &str) -> Pending<'_, Shape> {
        Box::pin(ready(self.shape_now(data_type)))
    }
    fn find(&self, query: &super::query::RecordQuery) -> Pending<'_, super::view::RecordPage> {
        Box::pin(ready(memory_find::find(self, query)))
    }
    fn get(
        &self,
        data_type: &str,
        id: &str,
        fields: &[Vec<String>],
    ) -> Pending<'_, Option<super::view::RecordView>> {
        Box::pin(ready(memory_find::get(self, data_type, id, fields)))
    }
    fn groups(
        &self,
        data_type: &str,
        grouping: &str,
        _scope: Option<&str>,
    ) -> Pending<'_, GroupSet> {
        Box::pin(ready(self.group_set(data_type, grouping)))
    }
    fn generation(&self, data_type: &str, _scope: Option<&str>) -> Pending<'_, u64> {
        let now = if self.has(Flaw::GenerationFrozen) {
            0
        } else {
            self.generation.get()
        };
        Box::pin(ready(self.type_check(data_type).map(|()| now)))
    }
    fn changes(
        &self,
        query: &super::query::RecordQuery,
        since_state: &str,
    ) -> Pending<'_, QueryChanges> {
        let out = memory_find::find(self, query).and_then(|page| {
            if page.state != since_state {
                return Err(Coded::new(RecordsCode::CannotCalculateChanges));
            }
            let (old_state, new_state) = (since_state.to_owned(), page.state);
            Ok(QueryChanges {
                old_state,
                new_state,
                removed: Vec::new(),
                added: Vec::new(),
                total: page.total,
            })
        });
        Box::pin(ready(out))
    }
}
