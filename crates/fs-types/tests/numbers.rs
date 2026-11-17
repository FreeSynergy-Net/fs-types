//! `Integer`, `Whole`, `Flag`, `Duration` and the day arithmetic of `Date` (0.0.3).

use fs_types::{
    BasicRules, BasicType, BasicValue, Date, Duration, Flag, Integer, TypeProblem, Whole,
};

fn canon<T: BasicValue>(typed: &str) -> String {
    T::parse(typed).unwrap().canonical()
}

fn bad<T: BasicValue>(typed: &str) -> TypeProblem {
    T::parse(typed).err().expect("should be refused")
}

fn date(t: &str) -> Date {
    Date::parse(t).unwrap()
}

#[test]
fn integer_is_signed_decimal_in_canonical_form() {
    assert_eq!(canon::<Integer>(" +007 "), "7");
    assert_eq!(canon::<Integer>("-0"), "0");
    assert_eq!(canon::<Integer>("-42"), "-42");
    assert_eq!(
        Integer::parse("-9223372036854775808").unwrap().get(),
        i64::MIN
    );
    assert_eq!(
        Integer::parse("9223372036854775807").unwrap().get(),
        i64::MAX
    );
    assert_eq!(
        bad::<Integer>("9223372036854775808"),
        TypeProblem::OutOfRange
    );
    assert_eq!(
        bad::<Integer>("-9223372036854775809"),
        TypeProblem::OutOfRange
    );
    assert_eq!(bad::<Integer>("1.5"), TypeProblem::BadNumber);
    assert_eq!(bad::<Integer>("-"), TypeProblem::BadNumber);
    assert_eq!(bad::<Integer>("1 2"), TypeProblem::BadNumber);
    assert_eq!(bad::<Integer>(" "), TypeProblem::Empty);
    assert_eq!(bad::<Integer>("0x10").code(), "number.bad");
}

#[test]
fn whole_refuses_negative_numbers() {
    assert_eq!(canon::<Whole>("0"), "0");
    assert_eq!(canon::<Whole>("-0"), "0");
    assert_eq!(canon::<Whole>("+12"), "12");
    assert_eq!(bad::<Whole>("-1"), TypeProblem::OutOfRange);
    assert_eq!(bad::<Whole>("-100"), TypeProblem::OutOfRange);
    assert_eq!(
        Whole::parse("18446744073709551615").unwrap().get(),
        u64::MAX
    );
    assert_eq!(
        bad::<Whole>("18446744073709551616"),
        TypeProblem::OutOfRange
    );
    assert_eq!(bad::<Whole>("abc"), TypeProblem::BadNumber);
    assert_eq!(bad::<Whole>(""), TypeProblem::Empty);
}

#[test]
fn flag_reads_yes_no_and_writes_true_false() {
    assert_eq!(canon::<Flag>(" TRUE "), "true");
    assert_eq!(canon::<Flag>("No"), "false");
    assert_eq!(canon::<Flag>("yes"), "true");
    assert!(Flag::parse("true").unwrap().get());
    assert!(!Flag::parse("false").unwrap().get());
    assert_ne!(Flag::parse("true").unwrap(), Flag::parse("false").unwrap());
    assert_eq!(bad::<Flag>("1"), TypeProblem::BadFlag);
    assert_eq!(bad::<Flag>("maybe").code(), "flag.bad");
    assert_eq!(bad::<Flag>(""), TypeProblem::Empty);
}

#[test]
fn duration_follows_the_jscalendar_grammar() {
    assert_eq!(canon::<Duration>("P1W"), "P1W");
    assert_eq!(canon::<Duration>(" p1dt2h30m "), "P1DT2H30M");
    assert_eq!(canon::<Duration>("PT0045S"), "PT45S");
    assert_eq!(canon::<Duration>("PT0.500S"), "PT0.5S");
    assert_eq!(canon::<Duration>("PT0.0S"), "PT0S");
    assert_eq!(canon::<Duration>("PT60M"), "PT60M");
    assert_eq!(canon::<Duration>("P2D"), "P2D");
    // Hours then seconds without minutes is not RFC 8984: read, written with `0M`.
    assert_eq!(canon::<Duration>("PT1H30S"), "PT1H0M30S");
    assert_eq!(canon::<Duration>("P1DT1H1S"), "P1DT1H0M1S");
    assert_eq!(canon::<Duration>("PT1H0.5S"), "PT1H0M0.5S");
    // Written once, read again, written again: the same text (idempotent).
    for t in [
        "PT1H30S",
        "P1DT1H1S",
        "PT1H",
        "PT30S",
        "P1W",
        "PT0S",
        "pt1h0.5s",
        "P1DT2H3M4.5S",
    ] {
        let once = canon::<Duration>(t);
        assert_eq!(canon::<Duration>(&once), once, "{t}");
    }
    for ok in [
        "PT1H",
        "PT1H5M",
        "PT1H5M30S",
        "PT5M30S",
        "PT30S",
        "P1DT30S",
        "PT0S",
    ] {
        assert_eq!(canon::<Duration>(ok), ok);
    }
    for t in [
        "1D", "P", "PT", "P1H", "P1DT", "PT1D", "P1Y", "P1M", "P1W2D", "-P1D", "P1.5D", "PT.5S",
        "PT1.S", "P1DT1S1M", "P1 D",
    ] {
        assert_eq!(bad::<Duration>(t), TypeProblem::BadDuration, "{t}");
    }
    assert_eq!(
        bad::<Duration>("PT1.12345678901S"),
        TypeProblem::BadDuration
    );
    assert_eq!(
        bad::<Duration>("P99999999999999999999D"),
        TypeProblem::OutOfRange
    );
    assert_eq!(bad::<Duration>(""), TypeProblem::Empty);
}

#[test]
fn duration_counts_whole_seconds() {
    let secs = |t: &str| Duration::parse(t).unwrap().whole_seconds();
    assert_eq!(secs("P1W"), Some(604_800));
    assert_eq!(secs("P1DT2H3M4S"), Some(86_400 + 7_200 + 180 + 4));
    assert_eq!(secs("PT0.9S"), Some(0));
    assert_eq!(secs("P40000000000000W"), None);
}

#[test]
fn registry_checks_the_new_kinds_by_kind() {
    let r = BasicRules::standard();
    assert_eq!(r.check(BasicType::Integer, "+5", &[]).unwrap(), "5");
    assert_eq!(r.check(BasicType::Whole, "5", &[]).unwrap(), "5");
    assert_eq!(r.check(BasicType::Flag, "YES", &[]).unwrap(), "true");
    assert_eq!(r.check(BasicType::Duration, "pt5m", &[]).unwrap(), "PT5M");
    assert_eq!(
        r.check(BasicType::Whole, "-1", &[]).err(),
        Some(TypeProblem::OutOfRange)
    );
}

#[test]
fn schema_ordinals_are_appended() {
    assert_eq!(BasicType::Reference as u16, 16);
    assert_eq!(BasicType::Integer as u16, 17);
    assert_eq!(BasicType::Duration as u16, 18);
    assert_eq!(BasicType::Flag as u16, 19);
    assert_eq!(BasicType::Whole as u16, 20);
}

#[test]
fn plus_days_crosses_month_and_year_ends() {
    assert_eq!(date("2026-11-17").plus_days(0), Some(date("2026-11-17")));
    assert_eq!(date("2026-11-17").plus_days(13), Some(date("2026-11-30")));
    assert_eq!(date("2026-11-17").plus_days(14), Some(date("2026-12-01")));
    assert_eq!(date("2026-12-31").plus_days(1), Some(date("2027-01-01")));
    assert_eq!(date("2027-01-01").plus_days(-1), Some(date("2026-12-31")));
    assert_eq!(date("2026-03-01").plus_days(-1), Some(date("2026-02-28")));
}

#[test]
fn plus_days_knows_leap_years() {
    assert_eq!(date("2024-02-28").plus_days(1), Some(date("2024-02-29")));
    assert_eq!(date("2024-02-28").plus_days(2), Some(date("2024-03-01")));
    assert_eq!(date("2023-02-28").plus_days(1), Some(date("2023-03-01")));
    assert_eq!(date("2000-02-28").plus_days(1), Some(date("2000-02-29")));
    assert_eq!(date("1900-02-28").plus_days(1), Some(date("1900-03-01")));
    assert_eq!(date("2024-01-01").plus_days(366), Some(date("2025-01-01")));
}

#[test]
fn plus_days_overflow_is_none_not_a_panic() {
    assert_eq!(date("2026-11-17").plus_days(i64::MAX), None);
    assert_eq!(date("2026-11-17").plus_days(i64::MIN), None);
    assert_eq!(date("2026-11-17").plus_days(1 << 40), None);
    // The days fit `TimeDelta`, the date does not (the `checked_add_signed` path).
    assert_eq!(date("2026-11-17").plus_days(100_000_000), None);
    assert_eq!(date("2026-11-17").plus_days(-100_000_000), None);
    assert_eq!(date("2026-11-17").plus_days(-(1 << 40)), None);
}

#[test]
fn plus_days_stays_within_the_years_parse_reads() {
    // Results outside 0000..=9999 would write a text `Date::parse` refuses: `None` instead.
    assert_eq!(date("2026-11-17").plus_days(2_929_000), None);
    assert_eq!(date("2026-11-17").plus_days(-800_000), None);
    assert_eq!(date("9999-12-31").plus_days(1), None);
    assert_eq!(date("0000-01-01").plus_days(-1), None);
    // The edge days themselves are valid.
    assert_eq!(date("9999-12-30").plus_days(1), Some(date("9999-12-31")));
    assert_eq!(date("0000-01-02").plus_days(-1), Some(date("0000-01-01")));
    // Every result written can be read again.
    for days in [-700_000, -1, 0, 1, 2_000_000, 2_900_000] {
        if let Some(d) = date("2026-11-17").plus_days(days) {
            assert_eq!(Date::parse(&d.canonical()).unwrap(), d, "{days}");
        }
    }
}

#[test]
fn days_until_counts_with_sign() {
    let a = date("2026-11-17");
    assert_eq!(a.days_until(&date("2026-11-30")), 13);
    assert_eq!(date("2026-11-30").days_until(&a), -13);
    assert_eq!(a.days_until(&a), 0);
    assert_eq!(date("2024-02-28").days_until(&date("2024-03-01")), 2);
    assert_eq!(date("2023-02-28").days_until(&date("2023-03-01")), 1);
    assert_eq!(a.days_until(&a.plus_days(1000).unwrap()), 1000);
}
