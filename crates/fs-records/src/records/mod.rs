//! `Records`: ONE shared way to read the records of any program (Entscheid b7 (1): „gemeinsame Ausführung statt
//! Bindungscode je Programm“), next to `TypeCatalog`, which only describes.
//!
//! Target: crate `fs-records` in repo fs-types (draft §1); the wire form is appended to `types.capnp` (draft file
//! `types-records.capnp`); the capnp server and client adapters come to fs-rpc beside `CatalogServer` (run F2). It lifts and
//! widens the provisional `Backend` of fs-admin-frontend (`backend.rs:53`: `list`, `change`, `run`), which then becomes an
//! adapter over it (draft §0, §10 O4).
//!
//! What F1 holds: reading (`find` with a filter tree after JMAP `/query` (Nachzug 17.11.), sort, grouping, projection and
//! pages; `get`; `groups`; `shape`;
//! `generation`), the rights rule (only what the asking person may read), the lock state, the limits. Writing is stage E4w.
//!
//! Pattern: Repository (the trait) + Value objects (query, view, shape) + Specification ([`Limits`]) + a double in memory.

mod api;
mod codes;
mod filter;
mod limits;
mod memory;
mod memory_filter;
mod memory_find;
mod memory_row;
mod memory_view;
mod query;
mod search;
mod shape;
mod view;

pub use api::{Pending, Records};
pub use codes::{RecordsCode, keys};
pub use filter::Filter;
pub use limits::{CEILING_DEFAULT, Limits, within};
pub use memory::{Flaw, MemoryRecords};
pub use memory_row::Row;
pub use query::{Condition, RecordQuery, Sort, SortBy, path};
pub use search::{QueryChanges, SearchKind, SearchPolicy};
pub use shape::{Group, GroupSet, GroupingDescription, OrderDescription, Shape};
pub use view::{
    EstimateKind, FieldValue, Hidden, HiddenReason, LockKind, RecordLock, RecordPage, RecordView,
    Snippet,
};
