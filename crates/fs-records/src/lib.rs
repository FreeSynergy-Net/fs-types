//! One shared way to READ the records of any program (`Records`), so that a forms description binds data without code per
//! program (ADR 0087 Nachtrag, stage E3, trait-delta F1).
//!
//! * [`Records`]: the trait every program offers next to each capability, beside `TypeCatalog`; filter, sort, page, group and
//!   project, always as the asking person.
//! * [`RecordQuery`], [`RecordView`], [`RecordPage`], [`Shape`], [`GroupSet`]: what is asked and answered; values are the
//!   canonical texts of their basic type, never JSON (ADR 0086).
//! * [`Limits`]: size and time limits as settings (default plus ceiling), [`within`] bounds a call in time.
//! * [`MemoryRecords`]: the double for tests and for the workbench adapters.
//! * `suite` (feature `conformance`): the 13 rules every `Records` provider runs, with the mutants that prove them.
//!
//! Rules of this crate:
//! 1. No sentence in code: every problem is a code with parameters ([`coded`]); the words live in MF2 files (ADR 0057).
//! 2. Read only what the person may read: a record she may not read does not exist for her (no count, no error that tells it
//!    exists); a field she may not read is listed as hidden with a reason and carries no value.
//! 3. Never silent: a failure is returned as a code; the host shows the part locked with the reason.
//! 4. No JSON as storage or on this boundary (ADR 0086).
//! 5. Writing is not here (stage E4w): this crate has no write method.
//! 6. fs-conformance never in the normal graph: the suite exists only with the feature `conformance` (review A4).
//!
//! Written with Claude (Anthropic), as the FreeSynergy READMEs say; an AI is a tool here, not an author of record.

#![forbid(unsafe_code)]

mod records;

pub use fs_types::coded;
pub use records::*;

#[cfg(feature = "conformance")]
pub mod suite;
