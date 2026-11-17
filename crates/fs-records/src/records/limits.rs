//! Limits of size and time. Settable (user rule „einstellbar statt fest“, decision O7): the program gives only the default
//! ([`Limits::default`]); the CEILING is an instance setting too (the admin sets and may lock it, fs-settings), so every check
//! takes it as a parameter: [`Limits::admit`] refuses a value above it, [`Limits::clamped`] pulls a value under it.
//! [`CEILING_DEFAULT`] is only the ceiling a program proposes when the instance sets none.
//!
//! The provider refuses a query beyond ITS limits; the host checks the query before sending and every page after receiving
//! ([`Limits::check_page`]), and bounds every call in time with [`within`]. The timer is handed in (the host passes
//! `tokio::time::sleep(limits.call())`), so this crate needs no runtime.
//!
//! Pattern: Specification (pure checks) + a small combinator ([`within`]).

use std::future::{Future, poll_fn};
use std::pin::pin;
use std::task::Poll;
use std::time::Duration;

use super::codes::{RecordsCode, keys};
use super::query::RecordQuery;
use super::view::RecordPage;
use crate::coded::Coded;

/// The limits of one provider or one host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Records per page.
    pub page_max: u32,
    /// Conditions per query.
    pub conditions_max: usize,
    /// Sort keys per query.
    pub sort_max: usize,
    /// Projected fields per query.
    pub fields_max: usize,
    /// Bytes of one value text.
    pub value_bytes_max: usize,
    /// Milliseconds of one call.
    pub call_ms: u64,
}

impl Default for Limits {
    /// 200 per page, 16 conditions, 4 sort keys, 64 fields, 64 KiB per value, 5 s per call.
    fn default() -> Self {
        Self {
            page_max: 200,
            conditions_max: 16,
            sort_max: 4,
            fields_max: 64,
            value_bytes_max: 64 * 1024,
            call_ms: 5_000,
        }
    }
}

/// The ceiling a program proposes when the instance sets none: 1000 per page, 64 conditions, 8 sort keys, 256 fields,
/// 1 MiB per value, 60 s per call. The instance's own ceiling replaces it (fs-settings).
pub const CEILING_DEFAULT: Limits = Limits {
    page_max: 1_000,
    conditions_max: 64,
    sort_max: 8,
    fields_max: 256,
    value_bytes_max: 1024 * 1024,
    call_ms: 60_000,
};

impl Limits {
    /// Each limit between 1 and its value in `ceiling` (the instance's ceiling).
    #[must_use]
    pub fn clamped(self, ceiling: Self) -> Self {
        Self {
            page_max: self.page_max.clamp(1, ceiling.page_max.max(1)),
            conditions_max: self.conditions_max.clamp(1, ceiling.conditions_max.max(1)),
            sort_max: self.sort_max.clamp(1, ceiling.sort_max.max(1)),
            fields_max: self.fields_max.clamp(1, ceiling.fields_max.max(1)),
            value_bytes_max: self
                .value_bytes_max
                .clamp(1, ceiling.value_bytes_max.max(1)),
            call_ms: self.call_ms.clamp(1, ceiling.call_ms.max(1)),
        }
    }

    /// Takes these limits as a setting under `ceiling`, or refuses them: a value of 0 or above its ceiling is not clamped
    /// silently but returned as a code, so the setting dialog can say which one.
    ///
    /// # Errors
    /// `limit-refused` with `what` (the field name) and `max` (its ceiling).
    pub fn admit(self, ceiling: Self) -> Result<Self, Coded<RecordsCode>> {
        let pairs: [(&str, u64, u64); 6] = [
            (
                "page-max",
                u64::from(self.page_max),
                u64::from(ceiling.page_max),
            ),
            (
                "conditions-max",
                self.conditions_max as u64,
                ceiling.conditions_max as u64,
            ),
            ("sort-max", self.sort_max as u64, ceiling.sort_max as u64),
            (
                "fields-max",
                self.fields_max as u64,
                ceiling.fields_max as u64,
            ),
            (
                "value-bytes-max",
                self.value_bytes_max as u64,
                ceiling.value_bytes_max as u64,
            ),
            ("call-ms", self.call_ms, ceiling.call_ms),
        ];
        match pairs.into_iter().find(|(_, v, max)| *v == 0 || v > max) {
            Some((what, _, max)) => Err(Coded::new(RecordsCode::LimitRefused)
                .with_text(keys::WHAT, what)
                .with_number(keys::MAX, max)),
            None => Ok(self),
        }
    }

    /// The smaller of two (host and provider): the host asks no more than both allow.
    #[must_use]
    pub fn meet(self, other: Self) -> Self {
        Self {
            page_max: self.page_max.min(other.page_max),
            conditions_max: self.conditions_max.min(other.conditions_max),
            sort_max: self.sort_max.min(other.sort_max),
            fields_max: self.fields_max.min(other.fields_max),
            value_bytes_max: self.value_bytes_max.min(other.value_bytes_max),
            call_ms: self.call_ms.min(other.call_ms),
        }
    }

    /// The time of one call.
    #[must_use]
    pub fn call(&self) -> Duration {
        Duration::from_millis(self.call_ms)
    }

    /// Whether a query keeps the limits.
    ///
    /// # Errors
    /// `page-size` (0 or above `page_max`), `too-many` (`what` = `conditions`, `sort`, `fields`).
    pub fn check_query(&self, q: &RecordQuery) -> Result<(), Coded<RecordsCode>> {
        if q.size == 0 || q.size > self.page_max {
            return Err(Coded::new(RecordsCode::PageSize)
                .with_number(keys::SIZE, u64::from(q.size))
                .with_number(keys::MAX, u64::from(self.page_max)));
        }
        let counts = [
            ("conditions", q.conditions.len(), self.conditions_max),
            ("sort", q.sort.len(), self.sort_max),
            ("fields", q.fields.len(), self.fields_max),
        ];
        match counts.into_iter().find(|(_, n, max)| n > max) {
            Some((what, _, max)) => Err(Coded::new(RecordsCode::TooMany)
                .with_text(keys::WHAT, what)
                .with_number(keys::MAX, max as u64)),
            None => Ok(()),
        }
    }

    /// Whether a page answered to `q` keeps the rules: not longer than asked, no value above `value_bytes_max`, no hidden
    /// field with a value. The host drops a page that does not, and shows the code.
    ///
    /// # Errors
    /// `page-too-long`, `value-too-large`, `hidden-has-value`.
    pub fn check_page(&self, q: &RecordQuery, page: &RecordPage) -> Result<(), Coded<RecordsCode>> {
        if page.items.len() > q.size as usize {
            return Err(Coded::new(RecordsCode::PageTooLong)
                .with_number(keys::SIZE, u64::from(q.size))
                .with_number(keys::GOT, page.items.len() as u64));
        }
        for record in &page.items {
            record.check_hidden()?;
            for field in &record.values {
                if let Some(v) = field.values.iter().find(|v| v.len() > self.value_bytes_max) {
                    return Err(Coded::new(RecordsCode::ValueTooLarge)
                        .with_text(keys::PATH, field.path.join("/"))
                        .with_number(keys::BYTES, v.len() as u64)
                        .with_number(keys::MAX, self.value_bytes_max as u64));
                }
            }
        }
        Ok(())
    }
}

/// Runs `work` until it ends or `timer` ends first; then `timeout` with `ms`. `work` is polled first, so a call that is
/// ready when the timer is too still counts.
///
/// # Errors
/// `timeout` with parameter `ms`.
pub async fn within<T>(
    ms: u64,
    work: impl Future<Output = T>,
    timer: impl Future<Output = ()>,
) -> Result<T, Coded<RecordsCode>> {
    let mut work = pin!(work);
    let mut timer = pin!(timer);
    poll_fn(|cx| {
        if let Poll::Ready(v) = work.as_mut().poll(cx) {
            return Poll::Ready(Ok(v));
        }
        if timer.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err(
                Coded::new(RecordsCode::Timeout).with_number(keys::MS, ms)
            ));
        }
        Poll::Pending
    })
    .await
}
