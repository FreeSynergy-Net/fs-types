//! The switches of a run.
//!
//! Pattern: Value object (plain data, `Default` = a full provider, the quick run).

use crate::ids::RuleId;

/// Switches of a run. The default is the strict one: a full provider, every rule, the short form of long rules.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// The provider offers only part of the capabilities (a staged build): a rule whose need is missing is skipped
    /// (listed, never passed). Off, a missing need is the failure `capability-missing`.
    pub partial: bool,
    /// The long run (night): rules that write a lot may write more. Rules read it through [`crate::Cx::options`].
    pub heavy: bool,
    /// Rules left out on purpose; each is listed as skipped with [`crate::Need::OPTION_SKIP`], never run, never passed.
    pub skip: Vec<RuleId>,
}
