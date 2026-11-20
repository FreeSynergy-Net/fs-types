//! `Time` (RFC 3339 partial-time) and `Color` (CSS hex) (0.0.5).

use fs_types::{BasicRules, BasicType, BasicValue, Color, Time, TypeProblem};

fn canon<T: BasicValue>(typed: &str) -> String {
    T::parse(typed).unwrap().canonical()
}

fn bad<T: BasicValue>(typed: &str) -> TypeProblem {
    T::parse(typed).err().expect("should be refused")
}

#[test]
fn time_reads_the_strict_form_and_keeps_it() {
    assert_eq!(canon::<Time>("00:00:00"), "00:00:00");
    assert_eq!(canon::<Time>("  23:59:59 "), "23:59:59");
    assert_eq!(canon::<Time>("09:05:07"), "09:05:07");
    let t = Time::parse("13:45:10").unwrap();
    assert_eq!(
        (t.hour(), t.minute(), t.second(), t.nanosecond()),
        (13, 45, 10, 0)
    );
}

#[test]
fn time_fraction_is_canonical_without_trailing_zeros() {
    assert_eq!(canon::<Time>("12:00:00.5"), "12:00:00.5");
    assert_eq!(canon::<Time>("12:00:00.500"), "12:00:00.5");
    assert_eq!(canon::<Time>("12:00:00.000"), "12:00:00");
    assert_eq!(canon::<Time>("12:00:00.000000001"), "12:00:00.000000001");
    assert_eq!(canon::<Time>("12:00:00.010"), "12:00:00.01");
    assert_eq!(
        Time::parse("12:00:00.25").unwrap().nanosecond(),
        250_000_000
    );
    assert_eq!(
        Time::parse("12:00:00.999999999").unwrap().nanosecond(),
        999_999_999
    );
}

#[test]
fn time_refuses_what_is_not_a_partial_time() {
    for t in [
        "24:00:00",
        "12:60:00",
        "12:00:60",
        "9:05:07",
        "09:5:07",
        "09:05",
        "09:05:07:01",
        "09-05-07",
        "T09:05:07",
        "09:05:07Z",
        "09:05:07+01:00",
        "12:00:00.",
        "12:00:00.1234567891",
        "12:00:00.1a",
        "12:00:00,5",
        "١٢:٠٠:٠٠",
        "+1:00:00",
        "12:00:-1",
        "noon",
    ] {
        assert_eq!(bad::<Time>(t), TypeProblem::BadTimeOfDay, "{t}");
    }
    assert_eq!(bad::<Time>(""), TypeProblem::Empty);
    assert_eq!(bad::<Time>("   "), TypeProblem::Empty);
}

#[test]
fn time_roundtrips_and_orders() {
    for t in ["00:00:00", "23:59:59.999999999", "01:02:03.04"] {
        let v = Time::parse(t).unwrap();
        assert_eq!(Time::parse(&v.canonical()).unwrap(), v);
        assert_eq!(v.canonical(), t);
    }
    assert!(Time::parse("09:00:00").unwrap() < Time::parse("09:00:00.1").unwrap());
    assert!(Time::new(24, 0, 0, 0).is_err());
    assert!(Time::new(0, 0, 0, 1_000_000_000).is_err());
    assert_eq!(Time::new(1, 2, 3, 0).unwrap().canonical(), "01:02:03");
}

#[test]
fn color_normalises_to_lower_case_long_form() {
    assert_eq!(canon::<Color>("#FFF"), "#ffffff");
    assert_eq!(canon::<Color>("#abc"), "#aabbcc");
    assert_eq!(canon::<Color>("#AbCdEf"), "#abcdef");
    assert_eq!(canon::<Color>(" #123456 "), "#123456");
    // Alpha only when not opaque; the short form expands digit by digit.
    assert_eq!(canon::<Color>("#abcf"), "#aabbcc");
    assert_eq!(canon::<Color>("#abc8"), "#aabbcc88");
    assert_eq!(canon::<Color>("#11223344"), "#11223344");
    assert_eq!(canon::<Color>("#112233FF"), "#112233");
    assert_eq!(canon::<Color>("#00000000"), "#00000000");
}

#[test]
fn color_channels_are_read_right() {
    let c = Color::parse("#12345678").unwrap();
    assert_eq!(
        (c.red(), c.green(), c.blue(), c.alpha()),
        (0x12, 0x34, 0x56, 0x78)
    );
    let c = Color::parse("#f08").unwrap();
    assert_eq!(
        (c.red(), c.green(), c.blue(), c.alpha()),
        (0xff, 0x00, 0x88, 255)
    );
    assert_eq!(Color::new(1, 2, 3, 255).canonical(), "#010203");
    assert_eq!(Color::parse("#0f08").unwrap(), Color::new(0, 255, 0, 0x88));
}

#[test]
fn color_refuses_other_notations_and_lengths() {
    for t in [
        "fff",
        "#ff",
        "#ffff f",
        "#fffff",
        "#fffffff",
        "#fffffffff",
        "#ggg",
        "#ffg",
        "#",
        "##fff",
        "red",
        "rgb(0,0,0)",
        "#-fff",
        "#+fff",
        "# fff",
        "#ffé",
        "0xfff",
        "#ｆｆｆ",
    ] {
        assert_eq!(bad::<Color>(t), TypeProblem::BadColor, "{t}");
    }
    assert_eq!(bad::<Color>(""), TypeProblem::Empty);
}

#[test]
fn color_roundtrips_every_channel_value() {
    for v in [0u8, 1, 15, 16, 127, 128, 254, 255] {
        let c = Color::new(v, 255 - v, v, v);
        assert_eq!(Color::parse(&c.canonical()).unwrap(), c);
    }
}

#[test]
fn the_registry_and_the_wire_numbers_know_both() {
    let r = BasicRules::standard();
    assert_eq!(r.rule(BasicType::Time).kind(), BasicType::Time);
    assert_eq!(r.rule(BasicType::Color).kind(), BasicType::Color);
    assert_eq!(
        r.check(BasicType::Time, "07:00:00.50", &[]).unwrap(),
        "07:00:00.5"
    );
    assert_eq!(r.check(BasicType::Color, "#ABC", &[]).unwrap(), "#aabbcc");
    assert_eq!(
        r.check(BasicType::Color, "blue", &[]).err(),
        Some(TypeProblem::BadColor)
    );
    assert_eq!(BasicType::Time as u16, 21);
    assert_eq!(BasicType::Color as u16, 22);
    assert_eq!(BasicType::try_from(21).unwrap(), BasicType::Time);
    assert_eq!(BasicType::try_from(22).unwrap(), BasicType::Color);
    assert!(BasicType::try_from(23).is_err());
    assert_eq!(TypeProblem::BadTimeOfDay.code(), "time-of-day.bad");
    assert_eq!(TypeProblem::BadColor.code(), "color.bad");
}
