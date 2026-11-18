//! `Row`: one stored record of [`super::MemoryRecords`] with what the asking person may do with it.
//!
//! Pattern: Value object with a Builder-style chain.

use std::collections::BTreeMap;

use super::view::RecordLock;

/// One stored record with what the asking person may do with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The id.
    pub id: String,
    /// The container (task list), if any.
    pub scope: Option<String>,
    /// Canonical texts by top-level field name.
    pub values: BTreeMap<String, Vec<String>>,
    /// Whether the person may read the record at all.
    pub readable: bool,
    /// The fields she may not read.
    pub hidden: Vec<String>,
    /// The lock.
    pub lock: Option<RecordLock>,
    /// The group of a grouping the program computes (one without `field`).
    pub group: Option<String>,
    /// The record commands she has the right to run (decision 85: rights only, whatever `supported` or the lock say).
    pub may: Vec<String>,
}

impl Row {
    /// A readable record without values.
    #[must_use]
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            scope: None,
            values: BTreeMap::new(),
            readable: true,
            hidden: Vec::new(),
            lock: None,
            group: None,
            may: Vec::new(),
        }
    }
    /// Sets a field.
    #[must_use]
    pub fn set(mut self, field: &str, values: &[&str]) -> Self {
        self.values.insert(
            field.to_owned(),
            values.iter().map(|v| (*v).to_owned()).collect(),
        );
        self
    }
    /// In this container.
    #[must_use]
    pub fn in_scope(mut self, scope: &str) -> Self {
        self.scope = Some(scope.to_owned());
        self
    }
    /// She has the right to run these record commands (names), whatever `supported` or the lock say.
    #[must_use]
    pub fn allowed(mut self, commands: &[&str]) -> Self {
        self.may = commands.iter().map(|c| (*c).to_owned()).collect();
        self
    }
    /// The person may not read it.
    #[must_use]
    pub fn unreadable(mut self) -> Self {
        self.readable = false;
        self
    }
    /// She may not read this field.
    #[must_use]
    pub fn hiding(mut self, field: &str) -> Self {
        self.hidden.push(field.to_owned());
        self
    }
    /// Locked.
    #[must_use]
    pub fn locked(mut self, lock: RecordLock) -> Self {
        self.lock = Some(lock);
        self
    }
    /// In this group of the program's grouping.
    #[must_use]
    pub fn in_group(mut self, key: &str) -> Self {
        self.group = Some(key.to_owned());
        self
    }
}
