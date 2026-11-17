//! A mutant: one deliberately wrong behaviour of a fixture, and the rules that must catch it.
//!
//! Pattern: Strategy (`Mutant`, supplied next to the fake of the building block); one name for the idea (condition C2:
//! "Mutant", no "Mutation"/"Mutator" beside it). Today's form in fs-tasks is a switch on the fake (`FakeTasks::with_mutation`,
//! `fs-tasks-testdata/src/mutation.rs`); a `Mutant` is that switch with an id and the rules it must break.

use crate::ids::{MutantId, RuleId};

/// Whether the fixture could switch a mutant on.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    /// The wrong behaviour is on.
    Yes,
    /// This fixture cannot do it (a real provider, or a fake without the switch): reported, never silently passed.
    NotApplicable,
}

/// One wrong behaviour over the fixture type `F`.
pub trait Mutant<F: ?Sized> {
    /// The stable id.
    fn id(&self) -> MutantId;

    /// The rules that must fail while this mutant is on (at least one).
    fn must_fail(&self) -> Vec<RuleId>;

    /// Switches the wrong behaviour on, on a fresh fixture the frame made for this mutant only.
    fn apply(&self, fixture: &mut F) -> Applied;
}
