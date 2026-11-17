//! What a run and a proof found.
//!
//! Pattern: Value objects (plain data; `is_conformant` and `is_proven` are the only questions).

use crate::ids::{MutantId, Need, RuleId, SuiteId};
use fs_types::coded::Code;

/// One failed rule (or a frame check about a rule id) with the code unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    /// The rule.
    pub rule: RuleId,
    /// The code of the rule's building block (or of the frame), with its parameters.
    pub code: Code,
}

/// One rule that did not run, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skip {
    /// The rule.
    pub rule: RuleId,
    /// What was missing ([`Need::OPTION_SKIP`] if it was left out on purpose).
    pub need: Need,
}

/// The result of one run of one suite. Empty `failures` means conformant; `skipped` is not proven either.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The suite.
    pub suite: SuiteId,
    /// Failed rules, in the order they ran (frame checks first).
    pub failures: Vec<Failure>,
    /// Rules that did not run.
    pub skipped: Vec<Skip>,
    /// Rules that ran to a verdict `Pass` or `Fail` (skipped ones are not counted).
    pub checked: usize,
}

impl Report {
    /// An empty report of `suite`.
    #[must_use]
    pub fn new(suite: SuiteId) -> Self {
        Self {
            suite,
            failures: Vec::new(),
            skipped: Vec::new(),
            checked: 0,
        }
    }

    /// No rule failed.
    #[must_use]
    pub fn is_conformant(&self) -> bool {
        self.failures.is_empty()
    }

    /// Whether `rule` failed in this run.
    #[must_use]
    pub fn failed(&self, rule: RuleId) -> bool {
        self.failures.iter().any(|f| f.rule == rule)
    }
}

/// The result of proving a suite with mutants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutantReport {
    /// The suite.
    pub suite: SuiteId,
    /// Mutants all of whose rules failed as they must.
    pub caught: Vec<MutantId>,
    /// What is wrong with the proof (`mutant-not-caught`, `mutant-unknown-rule`, `mutant-not-applicable`,
    /// `rule-without-mutant`, `mutant-id-invalid`, `duplicate-mutant`, and the id checks of the run).
    pub failures: Vec<Code>,
}

impl MutantReport {
    /// Every mutant was caught and every rule has a mutant.
    #[must_use]
    pub fn is_proven(&self) -> bool {
        self.failures.is_empty()
    }
}
