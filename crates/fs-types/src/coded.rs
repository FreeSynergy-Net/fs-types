//! What went wrong, as a code with typed parameters: never a sentence (ADR 0057; the line of fs-rpc `docs/errors.md`).
//!
//! Every crate that reports a failure names its own enum of stable codes ([`ErrorCode`]) and carries it with typed
//! parameters ([`Coded`]); the program words it in its MessageFormat 2 files under the key [`message_key`]. A report that
//! holds codes of many crates erases the enum and keeps prefix, name and parameters ([`Code`]). Parameter keys are `String`
//! as in `RpcError` (decision T1 of the scaffold drafts); `message_key` is a free function (no default methods in the trait).
//! Users: fs-conformance (its reports), fs-starter, fs-setting. Added in the first change after the tag `v0.0.1`;
//! nothing existing is touched.
//!
//! Pattern: Value object (`Coded`, `Code`, `Param`) + a catalog per enum (`ErrorCode`).

use std::fmt;

/// A typed parameter: text or number. The program words it in its MessageFormat 2 files.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Param {
    /// A text, such as a rule id.
    Text(String),
    /// A number, such as a size in bytes.
    Number(u64),
}

/// The stable codes of one crate: an enum whose variants are never renamed or reused once released.
pub trait ErrorCode: Copy + Eq + fmt::Debug + 'static {
    /// The prefix of every message key of this enum, ending in `-`, such as `conformance-error-`.
    const PREFIX: &'static str;
    /// Every code, in the order of the enum.
    const ALL: &'static [Self];
    /// The kebab-case name, such as `duplicate-rule`.
    fn as_str(self) -> &'static str;
}

/// The MessageFormat 2 key of a code: [`ErrorCode::PREFIX`] and [`ErrorCode::as_str`].
#[must_use]
pub fn message_key<C: ErrorCode>(code: C) -> String {
    format!("{}{}", C::PREFIX, code.as_str())
}

/// A code of a known enum with its parameters, in the order they were added.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coded<C: ErrorCode> {
    code: C,
    params: Vec<(String, Param)>,
}

impl<C: ErrorCode> Coded<C> {
    /// The code without parameters.
    #[must_use]
    pub fn new(code: C) -> Self {
        Self {
            code,
            params: Vec::new(),
        }
    }

    /// Adds a parameter of any kind, unchanged (for code that copies the parameters of another code).
    #[must_use]
    pub fn with_param(mut self, key: &str, value: Param) -> Self {
        self.params.push((key.to_owned(), value));
        self
    }

    /// Adds a text parameter.
    #[must_use]
    pub fn with_text(self, key: &str, value: impl Into<String>) -> Self {
        self.with_param(key, Param::Text(value.into()))
    }

    /// Adds a number parameter.
    #[must_use]
    pub fn with_number(self, key: &str, value: u64) -> Self {
        self.with_param(key, Param::Number(value))
    }

    /// The code.
    #[must_use]
    pub fn code(&self) -> C {
        self.code
    }

    /// The parameters, in the order they were added.
    pub fn params(&self) -> impl Iterator<Item = (&str, &Param)> {
        self.params.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// The same code without its enum type, for a report that holds codes of many crates. Nothing is lost.
    #[must_use]
    pub fn erase(self) -> Code {
        Code {
            prefix: C::PREFIX,
            name: self.code.as_str(),
            params: self.params,
        }
    }
}

impl<C: ErrorCode> From<Coded<C>> for Code {
    fn from(coded: Coded<C>) -> Self {
        coded.erase()
    }
}

/// A code whose enum is no longer known: prefix, name and parameters, unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code {
    /// The prefix of the enum, such as `tasks-error-`.
    pub prefix: &'static str,
    /// The kebab-case name.
    pub name: &'static str,
    /// The parameters, in order.
    pub params: Vec<(String, Param)>,
}

impl Code {
    /// The MessageFormat 2 key: prefix and name.
    #[must_use]
    pub fn message_key(&self) -> String {
        format!("{}{}", self.prefix, self.name)
    }

    /// The parameter with this key, if there is one (the first one).
    #[must_use]
    pub fn param(&self, key: &str) -> Option<&Param> {
        self.params.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

/// A technical text for logs only (`conformance-error-duplicate-rule (rule=a.b)`); a window must not show it.
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix, self.name)?;
        for (i, (k, v)) in self.params.iter().enumerate() {
            let sep = if i == 0 { " (" } else { ", " };
            match v {
                Param::Text(t) => write!(f, "{sep}{k}={t}")?,
                Param::Number(n) => write!(f, "{sep}{k}={n}")?,
            }
        }
        if self.params.is_empty() {
            Ok(())
        } else {
            write!(f, ")")
        }
    }
}
