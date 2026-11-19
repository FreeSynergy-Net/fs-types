//! `Time`: a time of day, RFC 3339 `partial-time` (no date, no zone, no offset).
//!
//! Pattern: Value object, Template method (`BasicValue`). Strict: `HH:MM:SS` with an optional fraction of one to nine
//! digits (`.5`, `.123456789`); two-digit hour 0-23, minute 0-59, second 0-59 (a leap second `60` is refused, as in
//! [`crate::LocalDateTime`]); ASCII digits only, no `T` in front, no offset. The canonical text has the fraction
//! without trailing zeros and without the dot when it is zero, so equal times have equal text.

use crate::types_capnp::BasicType;

use crate::{BasicValue, TypeProblem};

/// A time of day with nanosecond precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
    nano: u32,
}

impl Time {
    /// A time from its parts.
    ///
    /// # Errors
    /// [`TypeProblem::BadTimeOfDay`] when an hour above 23, a minute or second above 59, or a nanosecond of one second
    /// or more is given.
    pub fn new(hour: u8, minute: u8, second: u8, nano: u32) -> Result<Self, TypeProblem> {
        if hour > 23 || minute > 59 || second > 59 || nano >= 1_000_000_000 {
            return Err(TypeProblem::BadTimeOfDay);
        }
        Ok(Self {
            hour,
            minute,
            second,
            nano,
        })
    }

    /// Hour, 0-23.
    #[must_use]
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Minute, 0-59.
    #[must_use]
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Second, 0-59.
    #[must_use]
    pub fn second(&self) -> u8 {
        self.second
    }

    /// The fraction of the second in nanoseconds.
    #[must_use]
    pub fn nanosecond(&self) -> u32 {
        self.nano
    }
}

/// Exactly two ASCII digits as a number.
fn two(part: &str) -> Option<u8> {
    let b = part.as_bytes();
    if b.len() == 2 && b.iter().all(u8::is_ascii_digit) {
        Some((b[0] - b'0') * 10 + (b[1] - b'0'))
    } else {
        None
    }
}

/// The digits after the dot as nanoseconds (one to nine digits).
fn fraction(digits: &str) -> Option<u32> {
    if digits.is_empty() || digits.len() > 9 || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: u32 = digits.parse().ok()?;
    // `digits.len()` is at most 9, so the exponent fits.
    Some(n * 10u32.pow(u32::try_from(9 - digits.len()).ok()?))
}

impl BasicValue for Time {
    const KIND: BasicType = BasicType::Time;

    fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let text = typed.trim();
        if text.is_empty() {
            return Err(TypeProblem::Empty);
        }
        let (clock, frac) = match text.split_once('.') {
            Some((clock, frac)) => (clock, Some(frac)),
            None => (text, None),
        };
        let mut parts = clock.split(':');
        let parsed = (
            parts.next().and_then(two),
            parts.next().and_then(two),
            parts.next().and_then(two),
            parts.next(),
        );
        let (Some(hour), Some(minute), Some(second), None) = parsed else {
            return Err(TypeProblem::BadTimeOfDay);
        };
        let nano = match frac {
            Some(digits) => fraction(digits).ok_or(TypeProblem::BadTimeOfDay)?,
            None => 0,
        };
        Self::new(hour, minute, second, nano)
    }

    fn canonical(&self) -> String {
        let base = format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second);
        if self.nano == 0 {
            return base;
        }
        let frac = format!("{:09}", self.nano);
        format!("{base}.{}", frac.trim_end_matches('0'))
    }
}
