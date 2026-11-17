//! One rule: a check of one behaviour against a fixture, with the verdict.
//!
//! Pattern: Strategy (`Rule`, supplied by the building block) + Context object (`Cx`, what the frame lends a rule).

use async_trait::async_trait;

use crate::codes::{ConformanceCode, keys};
use crate::ids::{Need, RuleId};
use crate::switches::Options;
use fs_types::coded::{Code, Coded};

/// What a rule found.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The behaviour is there.
    Pass,
    /// It is not: the code of the building block, unchanged (its own enum, erased; parameters kept).
    Fail(Code),
    /// The rule could not run: this need is missing. Never counted as passed.
    Skipped(Need),
}

/// One rule over the fixture type `F` of its building block (usually `dyn SomethingFixture`, defined in that block).
///
/// A fixture type written as `dyn Trait` in `impl Rule<dyn Trait>` needs `&mut (dyn Trait + 'static)` in `check`, or an
/// alias `type Fx = dyn Trait;` (an elided `&mut dyn Trait` there has a shorter object lifetime and does not match).
/// Generic rules `impl<F: Trait + ?Sized> Rule<F>` avoid the problem and are the recommended form.
///
/// A rule seeds its own data, never relies on another rule, and does not panic: a setup that fails is
/// `Verdict::Fail` with `setup-failed` (or the block's own code). `?Send`: rules may hold capnp clients across `.await`.
#[async_trait(?Send)]
pub trait Rule<F: ?Sized> {
    /// The stable id.
    fn id(&self) -> RuleId;

    /// Checks the behaviour. The fixture is `&mut`: a rule may switch its setups (clock, person) and must leave them as
    /// it found them only if a later rule needs that; the frame does not reset the fixture between rules.
    async fn check(&self, fixture: &mut F, cx: &Cx) -> Verdict;
}

/// What the frame lends a rule: the switches, and the helpers that keep verdicts uniform.
#[derive(Clone, Debug)]
pub struct Cx {
    options: Options,
}

impl Cx {
    /// A context with these switches.
    #[must_use]
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    /// The switches of the run.
    #[must_use]
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// The verdict for a need the provider or fixture does not offer: skipped under `Options::partial`, otherwise the
    /// failure `capability-missing` with the need as parameter.
    #[must_use]
    pub fn missing(&self, need: Need) -> Verdict {
        if self.options.partial {
            Verdict::Skipped(need)
        } else {
            Verdict::Fail(
                Coded::new(ConformanceCode::CapabilityMissing)
                    .with_text(keys::NEED, need.as_str())
                    .erase(),
            )
        }
    }

    /// The failure `setup-failed` of `rule`, carrying the setup's own code (prefix and name as parameters `cause`,
    /// its parameters after them).
    #[must_use]
    pub fn setup_failed(&self, rule: RuleId, cause: Code) -> Verdict {
        let mut coded = Coded::new(ConformanceCode::SetupFailed)
            .with_text(keys::RULE, rule.as_str())
            .with_text("cause", cause.message_key());
        for (k, v) in cause.params {
            coded = coded.with_param(&k, v);
        }
        Verdict::Fail(coded.erase())
    }
}
