//! The codes of the frame itself (rule ids, capabilities, mutants), with their parameter keys.
//!
//! Pattern: none, an enum with total `match`es (as fs-rpc `code/kind.rs`). A code is never renamed or reused once released.

use crate::fault::ErrorCode;

/// What the frame found wrong, without a sentence. Message key: `conformance-error-` and the name.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConformanceCode {
    /// Two rules of one suite have the same id (parameter `rule`). Nothing is run.
    DuplicateRule,
    /// A rule id is not lower case ASCII words joined by `.` or `-` (parameter `rule`). Nothing is run.
    RuleIdInvalid,
    /// A full provider does not offer what a rule needs (parameter `need`); with `Options::partial` it is a skip instead.
    CapabilityMissing,
    /// A rule could not set up its own data (parameter `rule`, and the code of the setup as further parameters if any).
    SetupFailed,
    /// A mutant ran and a rule it must break stayed green (parameters `mutant`, `rule`).
    MutantNotCaught,
    /// A mutant names a rule the suite does not have (parameters `mutant`, `rule`).
    MutantUnknownRule,
    /// The fixture cannot switch this mutant on (parameter `mutant`): the fake must support every mutant it is given.
    MutantNotApplicable,
    /// No mutant names this rule: a rule that cannot fail proves nothing (parameter `rule`).
    RuleWithoutMutant,
}

impl ErrorCode for ConformanceCode {
    const PREFIX: &'static str = "conformance-error-";
    const ALL: &'static [Self] = &[
        Self::DuplicateRule,
        Self::RuleIdInvalid,
        Self::CapabilityMissing,
        Self::SetupFailed,
        Self::MutantNotCaught,
        Self::MutantUnknownRule,
        Self::MutantNotApplicable,
        Self::RuleWithoutMutant,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateRule => "duplicate-rule",
            Self::RuleIdInvalid => "rule-id-invalid",
            Self::CapabilityMissing => "capability-missing",
            Self::SetupFailed => "setup-failed",
            Self::MutantNotCaught => "mutant-not-caught",
            Self::MutantUnknownRule => "mutant-unknown-rule",
            Self::MutantNotApplicable => "mutant-not-applicable",
            Self::RuleWithoutMutant => "rule-without-mutant",
        }
    }
}

/// The parameter keys of the frame's codes.
pub mod keys {
    /// A rule id.
    pub const RULE: &str = "rule";
    /// A mutant id.
    pub const MUTANT: &str = "mutant";
    /// A need (capability or fixture setup).
    pub const NEED: &str = "need";
}
