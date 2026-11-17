//! `RecordsSuite`: the 13 rules in order.
//!
//! Pattern: Abstract factory (the suite makes its rules).

use fs_conformance::{Rule, Suite, SuiteId};

use super::fixture::RecordsFixture;
use super::{rules_read, rules_rights};

/// The suite `records`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RecordsSuite;

impl<F: RecordsFixture + ?Sized> Suite<F> for RecordsSuite {
    fn id(&self) -> SuiteId {
        SuiteId::new("records")
    }

    fn rules(&self) -> Vec<Box<dyn Rule<F>>> {
        vec![
            Box::new(rules_read::Pages),
            Box::new(rules_read::PageSize),
            Box::new(rules_read::UnknownField),
            Box::new(rules_read::Filter),
            Box::new(rules_read::Ties),
            Box::new(rules_rights::HiddenNoValue),
            Box::new(rules_rights::LockVisible),
            Box::new(rules_read::GroupsCover),
            Box::new(rules_read::UnknownGrouping),
            Box::new(rules_rights::MissingIsNone),
            Box::new(rules_read::GenerationMoves),
            Box::new(rules_read::ShapeTrue),
            Box::new(rules_rights::UnreadableAbsent),
        ]
    }
}
