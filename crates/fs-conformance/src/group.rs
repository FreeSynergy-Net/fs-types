//! A suite: the rules one building block ships behind its feature `conformance`.
//!
//! Pattern: Abstract factory (a suite makes its rules; the frame never knows the block).

use crate::check::Rule;
use crate::ids::SuiteId;

/// The rules of one building block over its fixture type `F`. The order of [`Suite::rules`] is the order of the run.
pub trait Suite<F: ?Sized> {
    /// The stable id of the suite.
    fn id(&self) -> SuiteId;

    /// The rules, in the order they run. Ids must be well formed and unique (the frame checks before it runs anything).
    fn rules(&self) -> Vec<Box<dyn Rule<F>>>;
}
