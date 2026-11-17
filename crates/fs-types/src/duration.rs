//! `Duration`: a span of time, RFC 8984 section 1.4.6 (the JSCalendar subset of ISO 8601 durations).
//!
//! Pattern: Value object, Template method (`BasicValue`). Own small parser: the grammar is a handful of rules and no
//! free crate keeps to this subset (no years, no months, a week stands alone). Units are kept as written, never
//! converted into each other (`PT60M` stays `PT60M`, as JSCalendar does); the canonical text is upper case, without
//! leading zeros. A negative duration is not a value here (JSCalendar's `Duration` has no sign). Time zones and
//! clocks are not involved: the span is plain arithmetic on seconds.
//!
//! ```text
//! P1W    P2D    P1DT2H30M    PT45S    PT0.5S
//! ```

use crate::types_capnp::BasicType;

use crate::{BasicValue, TypeProblem};

/// Seconds per week, day, hour, minute.
const WEEK: u64 = 604_800;
const DAY: u64 = 86_400;
const HOUR: u64 = 3_600;
const MINUTE: u64 = 60;

/// A duration: weeks alone, or days and a time part (hours, minutes, seconds with an optional fraction).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Duration {
    weeks: Option<u64>,
    days: Option<u64>,
    hours: Option<u64>,
    minutes: Option<u64>,
    seconds: Option<u64>,
    /// Digits after the decimal point of the seconds, no trailing zero; empty = none.
    fraction: String,
}

/// Take `<digits><unit>` from the front of `s` if the next unit is `unit`.
fn take(s: &mut &str, unit: char) -> Result<Option<(u64, String)>, TypeProblem> {
    let end = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    if end == 0 || !s[end..].starts_with(unit) {
        return Ok(None);
    }
    let number = &s[..end];
    *s = &s[end + unit.len_utf8()..];
    let (int, frac) = match number.split_once('.') {
        Some((i, f)) if unit == 'S' && !f.is_empty() => (i, f.trim_end_matches('0')),
        Some(_) => return Err(TypeProblem::BadDuration),
        None => (number, ""),
    };
    if int.is_empty() || frac.contains('.') || frac.len() > 9 {
        return Err(TypeProblem::BadDuration);
    }
    let n = int.parse::<u64>().map_err(|_| TypeProblem::OutOfRange)?;
    Ok(Some((n, frac.to_owned())))
}

impl Duration {
    /// Whole seconds (the fraction is cut off); `None` if the number does not fit `u64`.
    #[must_use]
    pub fn whole_seconds(&self) -> Option<u64> {
        let parts = [
            (self.weeks, WEEK),
            (self.days, DAY),
            (self.hours, HOUR),
            (self.minutes, MINUTE),
            (self.seconds, 1),
        ];
        parts.iter().try_fold(0u64, |sum, (n, unit)| {
            sum.checked_add(n.unwrap_or(0).checked_mul(*unit)?)
        })
    }
}

impl BasicValue for Duration {
    const KIND: BasicType = BasicType::Duration;

    fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let t = typed.trim();
        if t.is_empty() {
            return Err(TypeProblem::Empty);
        }
        let upper = t.to_ascii_uppercase();
        let mut rest = upper.strip_prefix('P').ok_or(TypeProblem::BadDuration)?;
        let mut d = Self {
            weeks: None,
            days: None,
            hours: None,
            minutes: None,
            seconds: None,
            fraction: String::new(),
        };
        if let Some((w, _)) = take(&mut rest, 'W')? {
            d.weeks = Some(w);
        } else {
            d.days = take(&mut rest, 'D')?.map(|(n, _)| n);
            if let Some(time) = rest.strip_prefix('T') {
                rest = time;
                if rest.is_empty() {
                    return Err(TypeProblem::BadDuration);
                }
                d.hours = take(&mut rest, 'H')?.map(|(n, _)| n);
                d.minutes = take(&mut rest, 'M')?.map(|(n, _)| n);
                if let Some((s, f)) = take(&mut rest, 'S')? {
                    d.seconds = Some(s);
                    d.fraction = f;
                    // RFC 8984: seconds follow minutes (`dur-minute = 1*DIGIT "M" [dur-second]`), so hours and
                    // seconds without minutes are read, and written with `0M` (read tolerantly, write strictly).
                    if d.hours.is_some() && d.minutes.is_none() {
                        d.minutes = Some(0);
                    }
                }
            }
        }
        let any = d.weeks.or(d.days).or(d.hours).or(d.minutes).or(d.seconds);
        if !rest.is_empty() || any.is_none() {
            return Err(TypeProblem::BadDuration);
        }
        Ok(d)
    }

    fn canonical(&self) -> String {
        let mut out = String::from("P");
        if let Some(w) = self.weeks {
            out.push_str(&w.to_string());
            out.push('W');
        }
        if let Some(n) = self.days {
            out.push_str(&n.to_string());
            out.push('D');
        }
        if self.hours.or(self.minutes).or(self.seconds).is_some() {
            out.push('T');
            for (n, unit) in [(self.hours, 'H'), (self.minutes, 'M')] {
                if let Some(n) = n {
                    out.push_str(&n.to_string());
                    out.push(unit);
                }
            }
            if let Some(s) = self.seconds {
                out.push_str(&s.to_string());
                if !self.fraction.is_empty() {
                    out.push('.');
                    out.push_str(&self.fraction);
                }
                out.push('S');
            }
        }
        out
    }
}
