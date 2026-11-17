//! `fs-conformance`: the frame of every conformance run in FreeSynergy, and nothing else.
//!
//! A building block (a crate) ships its own rules behind its feature `conformance`: a [`Suite`] of [`Rule`]s over a fixture
//! type `F` the block defines itself (usually `dyn SomethingFixture`). This crate only runs them ([`run`]), proves that every
//! rule can fail ([`prove`] with [`Mutant`]s), and reports codes with typed parameters, never sentences ([`Report`]).
//! It knows no program, no transport and no window: it depends on the values crate only (`coded`, stand-in [`fault`] until
//! the fs-types tag), so every layer may use it without an edge upwards (Opus review of the trait drafts, condition C1).
//!
//! Rules of the frame:
//! - A skipped rule is never a passed rule: it is listed in [`Report::skipped`] with the [`Need`] that was missing.
//! - A failure carries the code of the rule unchanged ([`Code`]: prefix, name, parameters; condition C3).
//! - A rule that no mutant breaks proves nothing: [`prove`] reports it (`rule-without-mutant`).
//! - Every mutant runs on a fresh fixture; [`run`] takes `&mut F` (condition C2).
//! - Rules run in the order of [`Suite::rules`], on one thread (`?Send`, capnp-rpc runs on a `LocalSet`).
//!
//! Pattern: Template method (`run`: check ids, then each rule) + Strategy (`Rule`, `Mutant`) + Value objects (`Report`, `Code`).

#![deny(missing_docs)]
#![warn(clippy::pedantic)]

mod check;
mod codes;
pub mod fault;
mod group;
mod ids;
mod outcome;
mod proof;
mod run;
mod switches;

pub use check::{Cx, Rule, Verdict};
pub use codes::{ConformanceCode, keys};
pub use fault::{Code, Coded, ErrorCode, Param, message_key};
pub use group::Suite;
pub use ids::{MutantId, Need, RuleId, SuiteId};
pub use outcome::{Failure, MutantReport, Report, Skip};
pub use proof::{Applied, Mutant};
pub use run::{prove, run};
pub use switches::Options;
