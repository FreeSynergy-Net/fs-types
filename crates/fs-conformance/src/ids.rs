//! Stable names: of a rule, a suite, a mutant, and of what a rule needs. Saved reports and mutants refer to them, so a
//! rename is a change of the suite, never a silent one.
//!
//! Pattern: Value objects (newtypes over `&'static str`, built in `const`).

/// The id of a rule: lower case ASCII words joined by `.` or `-`, such as `starter.version` or `tasks.claim-order`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleId(&'static str);

/// The id of a suite, same form as a rule id (`starter`, `tasks-v2`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SuiteId(&'static str);

/// The id of a mutant, same form (`tasks.group-open`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MutantId(&'static str);

/// What a rule needs and did not get: a capability of the provider or a setup of the fixture (`fixture.clock`), same form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Need(&'static str);

/// Whether `id` has the form of an id: 1 to 64 bytes of `a-z`, `0-9`, `.`, `-`, starting and ending with a letter or digit.
const fn well_formed(id: &str) -> bool {
    let b = id.as_bytes();
    if b.is_empty() || b.len() > 64 {
        return false;
    }
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let word = c.is_ascii_lowercase() || c.is_ascii_digit();
        let joint = c == b'.' || c == b'-';
        if !(word || (joint && i > 0 && i + 1 < b.len())) {
            return false;
        }
        i += 1;
    }
    true
}

macro_rules! id_type {
    ($t:ident) => {
        impl $t {
            /// The id; [`Self::is_well_formed`] tells whether it has the form (the frame checks rule ids before a run).
            #[must_use]
            pub const fn new(id: &'static str) -> Self {
                Self(id)
            }

            /// The id as text.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                self.0
            }

            /// Whether the id has the form of an id.
            #[must_use]
            pub const fn is_well_formed(self) -> bool {
                well_formed(self.0)
            }
        }
    };
}

id_type!(RuleId);
id_type!(SuiteId);
id_type!(MutantId);
id_type!(Need);

impl Need {
    /// The rule was left out by [`crate::Options::skip`].
    pub const OPTION_SKIP: Self = Self("option.skip");
}
