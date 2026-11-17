//! `Integer`, `Whole` and `Flag`: numbers and yes/no.
//!
//! Pattern: Value object, Template method (`BasicValue`). No external standard parser is needed: the rule is
//! "optional sign, then ASCII digits", checked once in `digits`; the range check is the integer type's own
//! (`i64`, `u64`), never a wrap-around. The canonical text is plain decimal (no `+`, no leading zeros, no `-0`), the
//! way it travels as `Text` (`schema/types.capnp`).

use crate::types_capnp::BasicType;

use crate::{BasicValue, TypeProblem};

/// Split `typed` (trimmed, not empty) into a sign (`true` = minus) and its digits; digits only, at least one.
fn digits(typed: &str) -> Result<(bool, &str), TypeProblem> {
    let t = typed.trim();
    if t.is_empty() {
        return Err(TypeProblem::Empty);
    }
    let (minus, rest) = match t.as_bytes()[0] {
        b'-' => (true, &t[1..]),
        b'+' => (false, &t[1..]),
        _ => (false, t),
    };
    if rest.is_empty() || !rest.bytes().all(|c| c.is_ascii_digit()) {
        return Err(TypeProblem::BadNumber);
    }
    Ok((minus, rest))
}

/// A whole number with a sign, 64 bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Integer(i64);

impl Integer {
    /// The number.
    #[must_use]
    pub fn get(&self) -> i64 {
        self.0
    }
}

impl From<i64> for Integer {
    fn from(n: i64) -> Self {
        Self(n)
    }
}

impl BasicValue for Integer {
    const KIND: BasicType = BasicType::Integer;

    fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let (minus, d) = digits(typed)?;
        // Parse with the sign, so that `i64::MIN` (whose magnitude does not fit `i64`) is a value.
        let signed = if minus { format!("-{d}") } else { d.to_owned() };
        signed
            .parse::<i64>()
            .map(Self)
            .map_err(|_| TypeProblem::OutOfRange)
    }

    fn canonical(&self) -> String {
        self.0.to_string()
    }
}

/// A whole number from 0 up: a count, a position, a percentage before its bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Whole(u64);

impl Whole {
    /// The number.
    #[must_use]
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for Whole {
    fn from(n: u64) -> Self {
        Self(n)
    }
}

impl BasicValue for Whole {
    const KIND: BasicType = BasicType::Whole;

    fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let (minus, d) = digits(typed)?;
        let n = d.parse::<u64>().map_err(|_| TypeProblem::OutOfRange)?;
        // `-0` is zero; any other negative number is below what the type holds.
        if minus && n != 0 {
            return Err(TypeProblem::OutOfRange);
        }
        Ok(Self(n))
    }

    fn canonical(&self) -> String {
        self.0.to_string()
    }
}

/// Yes or no. Canonical text is `true` or `false`; `yes`, `no` (any case) are read as well.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Flag(bool);

impl Flag {
    /// The answer.
    #[must_use]
    pub fn get(&self) -> bool {
        self.0
    }
}

impl From<bool> for Flag {
    fn from(b: bool) -> Self {
        Self(b)
    }
}

impl BasicValue for Flag {
    const KIND: BasicType = BasicType::Flag;

    fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let t = typed.trim();
        if t.is_empty() {
            return Err(TypeProblem::Empty);
        }
        match t.to_ascii_lowercase().as_str() {
            "true" | "yes" => Ok(Self(true)),
            "false" | "no" => Ok(Self(false)),
            _ => Err(TypeProblem::BadFlag),
        }
    }

    fn canonical(&self) -> String {
        self.0.to_string()
    }
}
