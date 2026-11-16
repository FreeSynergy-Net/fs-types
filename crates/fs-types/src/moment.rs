//! `Utc`, `LocalDateTime`, `Zone`: the time forms of ADR 0085 (instant, wall clock, zone).
//!
//! Pattern: Value object (valid on construction, equal values equal text). An instant is stored as UTC milliseconds,
//! a wall-clock time as its parts with no zone and no fraction, a zone as its kind and id. They are not
//! [`crate::BasicValue`]s (no `BasicType` kind yet; appending one is a schema change of its own), so `parse` and the
//! canonical text are methods of the types themselves.

use chrono::{DateTime as ChronoDateTime, SecondsFormat, Timelike};

use crate::{BasicValue, Date, TypeProblem};

/// An instant in UTC: milliseconds since 1970-01-01T00:00:00Z (ADR 0085). Compared and sorted as numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utc(i64);

impl Utc {
    /// The instant `ms` milliseconds after the epoch (negative: before it).
    #[must_use]
    pub fn from_millis(ms: i64) -> Self {
        Self(ms)
    }

    /// Milliseconds since the epoch.
    #[must_use]
    pub fn millis(&self) -> i64 {
        self.0
    }

    /// Read an RFC 3339 date-time with an offset (`…Z`, `…+01:00`), to the millisecond; finer fractions are cut off
    /// (never rounded). Spaces around it are ignored; `t` and `z` in lower case and `-00:00` (RFC 3339 section 4.3:
    /// UTC, offset unknown) are accepted; a leap second (`23:59:60`) is refused, because an instant is a count of
    /// milliseconds and has no place for it.
    ///
    /// # Errors
    /// `Empty`, `BadDateTime` (not RFC 3339, no offset, leap second), `OutOfRange` (the instant in UTC is not in the
    /// years 0 to 9999, which `to_rfc3339` could show).
    pub fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let t = typed.trim();
        if t.is_empty() {
            return Err(TypeProblem::Empty);
        }
        let dt = ChronoDateTime::parse_from_rfc3339(t).map_err(|_| TypeProblem::BadDateTime)?;
        if dt.nanosecond() >= 1_000_000_000 {
            return Err(TypeProblem::BadDateTime);
        }
        let u = Self(dt.timestamp_millis());
        if u.to_rfc3339().is_none() {
            return Err(TypeProblem::OutOfRange);
        }
        Ok(u)
    }

    /// The canonical text, `YYYY-MM-DDTHH:MM:SS.mmmZ`; `None` for an instant no calendar year 0-9999 can show.
    #[must_use]
    pub fn to_rfc3339(&self) -> Option<String> {
        let dt = ChronoDateTime::from_timestamp_millis(self.0)?;
        if !(0..=9999).contains(&chrono::Datelike::year(&dt)) {
            return None;
        }
        Some(dt.to_rfc3339_opts(SecondsFormat::Millis, true))
    }
}

/// A wall-clock time with no zone (JSCalendar `LocalDateTime`, RFC 8984 section 1.4.4): date, hour, minute, second.
/// No zone, no fraction of a second.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalDateTime {
    date: Date,
    hour: u8,
    minute: u8,
    second: u8,
}

impl LocalDateTime {
    /// From its parts.
    ///
    /// # Errors
    /// `BadTime` when the hour is over 23 or the minute or second over 59 (no leap second).
    pub fn new(date: Date, hour: u8, minute: u8, second: u8) -> Result<Self, TypeProblem> {
        if hour > 23 || minute > 59 || second > 59 {
            return Err(TypeProblem::BadTime);
        }
        Ok(Self {
            date,
            hour,
            minute,
            second,
        })
    }

    /// The date part.
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    /// Hour, 0 to 23.
    #[must_use]
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Minute, 0 to 59.
    #[must_use]
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Second, 0 to 59.
    #[must_use]
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Read `YYYY-MM-DDTHH:MM:SS` (nothing after the seconds).
    ///
    /// # Errors
    /// `BadDate` for the date part, `BadTime` for the rest (also a sign, a zone or a fraction).
    pub fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let t = typed.trim();
        let b = t.as_bytes();
        if b.len() != 19 || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
            return Err(TypeProblem::BadTime);
        }
        let digits = |a: usize, z: usize| -> Result<u8, TypeProblem> {
            let part = &b[a..z];
            if !part.iter().all(u8::is_ascii_digit) {
                return Err(TypeProblem::BadTime);
            }
            Ok((part[0] - b'0') * 10 + (part[1] - b'0'))
        };
        // The slice is cut at ASCII positions only after the shape above is known to be ASCII.
        let date = Date::parse(t.get(0..10).ok_or(TypeProblem::BadTime)?)?;
        Self::new(date, digits(11, 13)?, digits(14, 16)?, digits(17, 19)?)
    }

    /// The canonical text, `YYYY-MM-DDTHH:MM:SS`.
    #[must_use]
    pub fn canonical(&self) -> String {
        format!(
            "{}T{:02}:{:02}:{:02}",
            self.date.canonical(),
            self.hour,
            self.minute,
            self.second
        )
    }
}

/// Which kind of zone an id names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZoneKind {
    /// A name of the IANA time zone database (`Europe/Vienna`).
    Iana,
    /// A zone defined by a `VTIMEZONE` of the object itself; the id starts with `/` (RFC 8984 section 4.7.2).
    Custom,
}

/// A time zone: an IANA name or a custom zone. A custom zone is only valid with its object; without it a reader says
/// "unknown zone" and never guesses (ADR 0085). Whether an IANA name exists is the tz database's to say, not this
/// type's (it checks the form only).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Zone {
    kind: ZoneKind,
    id: String,
}

impl Zone {
    /// An IANA zone: not empty, no leading `/`, no space or control character.
    ///
    /// # Errors
    /// `BadZone`.
    pub fn iana(id: &str) -> Result<Self, TypeProblem> {
        if id.is_empty()
            || id.starts_with('/')
            || id.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(TypeProblem::BadZone);
        }
        Ok(Self {
            kind: ZoneKind::Iana,
            id: id.to_owned(),
        })
    }

    /// A custom zone: starts with `/` and has at least one more character, no space or control character.
    ///
    /// # Errors
    /// `BadZone`.
    pub fn custom(id: &str) -> Result<Self, TypeProblem> {
        if id.len() < 2
            || !id.starts_with('/')
            || id.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(TypeProblem::BadZone);
        }
        Ok(Self {
            kind: ZoneKind::Custom,
            id: id.to_owned(),
        })
    }

    /// The kind of zone.
    #[must_use]
    pub fn kind(&self) -> ZoneKind {
        self.kind
    }

    /// The id as written.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}
