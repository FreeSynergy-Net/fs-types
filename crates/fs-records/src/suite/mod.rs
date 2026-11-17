//! The conformance suite `records` (feature `conformance`): what every `Records` provider must do, checked through a fixture
//! ([`RecordsFixture`]); the standard one is [`MemoryBench`] over [`crate::records::MemoryRecords`]. Every rule has a mutant of
//! the bench that makes it fail ([`bench_mutants`], `fs_conformance::prove`).
//!
//! A program runs it in its tests with its own fixture (fs-tasks: a fixture over `LocalTasks` behind its `Records` adapter,
//! `tasks_v2` follow-up). fs-conformance is never in the normal graph: this module exists only with the feature (review A4).
//!
//! Pattern: Abstract factory (`RecordsSuite` makes the rules) + generic rules `impl<F: RecordsFixture + ?Sized> Rule<F>`
//! (fs-conformance review A1).

mod bench;
mod check;
mod codes;
mod fixture;
mod mutants;
mod records_suite;
mod rules_read;
mod rules_rights;

pub use bench::MemoryBench;
pub use codes::{SuiteCode, keys as suite_keys};
pub use fixture::{Arrange, Facts, RecordsFixture};
pub use mutants::{FlawMutant, bench_mutants};
pub use records_suite::RecordsSuite;
