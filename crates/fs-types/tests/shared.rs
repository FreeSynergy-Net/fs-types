//! The shared types of the topic crates: construction, refusals, the time forms of ADR 0085.

use fs_types::{
    Attendee, BasicValue, ContactAddress, Date, LocalDateTime, ObjectId, PageCursor, PersonArt,
    PersonRef, Problem, Reference, Revision, TypeProblem, Utc, Zone, ZoneKind,
};

#[test]
fn reference_is_an_absolute_iri_kept_as_written() {
    let r = Reference::parse("  urn:uuid:0192f5a0-7b1c-7cde-8f00-123456789abc ").unwrap();
    assert_eq!(
        r.canonical(),
        "urn:uuid:0192f5a0-7b1c-7cde-8f00-123456789abc"
    );
    let r = Reference::parse("https://Example.org/a/../b").unwrap();
    assert_eq!(r.canonical(), "https://Example.org/a/../b");
    assert_eq!(Reference::parse(" ").err(), Some(TypeProblem::Empty));
    assert_eq!(
        Reference::parse("no scheme").err(),
        Some(TypeProblem::BadUri)
    );
    assert_eq!(
        Reference::parse("/relative/path").err(),
        Some(TypeProblem::NotAbsolute)
    );
}

#[test]
fn date_exists_and_has_parts() {
    let d = Date::parse("2028-02-29").unwrap();
    assert_eq!((d.year(), d.month(), d.day()), (2028, 2, 29));
    assert_eq!(d.canonical(), "2028-02-29");
    assert_eq!(Date::parse("2027-02-29").err(), Some(TypeProblem::BadDate));
    assert_eq!(Date::parse("2027-2-9").err(), Some(TypeProblem::BadDate));
    assert_eq!(Date::parse("").err(), Some(TypeProblem::Empty));
    assert!(Date::parse("2026-11-16").unwrap() < Date::parse("2026-12-01").unwrap());
}

#[test]
fn local_date_time_has_no_zone_and_no_fraction() {
    let t = LocalDateTime::parse(" 2026-11-16T09:30:05 ").unwrap();
    assert_eq!((t.hour(), t.minute(), t.second()), (9, 30, 5));
    assert_eq!(t.date().canonical(), "2026-11-16");
    assert_eq!(t.canonical(), "2026-11-16T09:30:05");
    for bad in [
        "2026-11-16T09:30:05Z",
        "2026-11-16T09:30:05.5",
        "2026-11-16T09:30",
        "2026-11-16 09:30:05",
        "2026-11-16T+9:30:05",
        "2026-11-16T24:00:00",
        "2026-11-16T09:60:00",
        "2026-11-16T09:00:60",
        "2026-11-16T0\u{e4}:30:05",
    ] {
        assert_eq!(
            LocalDateTime::parse(bad).err(),
            Some(TypeProblem::BadTime),
            "{bad}"
        );
    }
    assert_eq!(
        LocalDateTime::parse("2026-13-16T09:30:05").err(),
        Some(TypeProblem::BadDate)
    );
    let d = Date::parse("2026-11-16").unwrap();
    assert!(LocalDateTime::new(d, 23, 59, 59).is_ok());
    assert_eq!(
        LocalDateTime::new(d, 24, 0, 0).err(),
        Some(TypeProblem::BadTime)
    );
}

#[test]
fn utc_counts_milliseconds_from_the_epoch() {
    let u = Utc::from_millis(1_794_821_405_123);
    assert_eq!(u.millis(), 1_794_821_405_123);
    assert_eq!(u.to_rfc3339().as_deref(), Some("2026-11-16T09:30:05.123Z"));
    assert_eq!(Utc::parse("2026-11-16T09:30:05.123Z").unwrap(), u);
    // The same instant written with an offset is the same value; a finer fraction is cut off.
    assert_eq!(Utc::parse("2026-11-16T10:30:05.123999+01:00").unwrap(), u);
    assert_eq!(Utc::parse("1969-12-31T23:59:59.999Z").unwrap().millis(), -1);
    assert_eq!(
        Utc::from_millis(0).to_rfc3339().as_deref(),
        Some("1970-01-01T00:00:00.000Z")
    );
    assert!(Utc::from_millis(1) > Utc::from_millis(-1));
    assert_eq!(
        Utc::parse("2026-11-16T09:30:05").err(),
        Some(TypeProblem::BadDateTime)
    );
    assert_eq!(Utc::parse("").err(), Some(TypeProblem::Empty));
    assert_eq!(Utc::from_millis(i64::MAX).to_rfc3339(), None);
}

#[test]
fn zone_is_iana_or_custom() {
    let z = Zone::iana("Europe/Vienna").unwrap();
    assert_eq!((z.kind(), z.id()), (ZoneKind::Iana, "Europe/Vienna"));
    let c = Zone::custom("/tz/ours").unwrap();
    assert_eq!((c.kind(), c.id()), (ZoneKind::Custom, "/tz/ours"));
    assert_ne!(Zone::iana("UTC").unwrap(), Zone::custom("/UTC").unwrap());
    for bad in ["", "/Europe/Vienna", "Europe/Vienna ", "Eu rope"] {
        assert_eq!(Zone::iana(bad).err(), Some(TypeProblem::BadZone), "{bad:?}");
    }
    for bad in ["", "/", "Europe/Vienna", "/a b"] {
        assert_eq!(
            Zone::custom(bad).err(),
            Some(TypeProblem::BadZone),
            "{bad:?}"
        );
    }
}

#[test]
fn contact_address_takes_any_absolute_uri_and_is_no_identity() {
    let a = ContactAddress::parse("MAILTO:ada@example.org").unwrap();
    assert_eq!(a.scheme(), "mailto");
    assert_eq!(a.canonical(), "MAILTO:ada@example.org");
    for ok in [
        "tel:+4312345",
        "xmpp:ada@example.org",
        "sip:ada@example.org",
        "matrix:u/ada:example.org",
        "https://example.org/ada",
    ] {
        assert!(ContactAddress::parse(ok).is_ok(), "{ok}");
    }
    assert!(ContactAddress::parse("urn:uuid:0192f5a0-7b1c-7cde-8f00-123456789abc").is_ok());
    assert_eq!(
        ContactAddress::parse("ada@example.org").err(),
        Some(TypeProblem::NotAbsolute)
    );
    assert_eq!(ContactAddress::parse("").err(), Some(TypeProblem::Empty));
}

#[test]
fn problem_carries_a_code_and_the_new_codes_are_stable() {
    let p = Problem::from_type("start", TypeProblem::BadTime);
    assert_eq!((p.field.as_str(), p.reason.as_str()), ("start", "time.bad"));
    let codes = [
        (TypeProblem::BadTime, "time.bad"),
        (TypeProblem::BadZone, "zone.bad"),
        (TypeProblem::OutOfRange, "out-of-range"),
        (TypeProblem::BadId, "id.bad"),
        (TypeProblem::BadVersion, "version.bad"),
    ];
    for (problem, code) in codes {
        assert_eq!(problem.code(), code);
        assert_eq!(problem.to_string(), format!("problem.{code}"));
    }
}

#[test]
fn a_reference_needs_something_after_the_scheme() {
    for bad in ["x:", "mailto:", "urn:", " https: "] {
        assert_eq!(
            Reference::parse(bad).err(),
            Some(TypeProblem::BadUri),
            "{bad:?}"
        );
    }
    for bad in ["mailto:", "tel:", "https:"] {
        assert_eq!(
            ContactAddress::parse(bad).err(),
            Some(TypeProblem::BadUri),
            "{bad:?}"
        );
    }
    assert!(Reference::parse("a:b").is_ok());
}

#[test]
fn utc_takes_what_it_can_show_and_says_why_not() {
    // Limits: year 0 and year 9999 in UTC are in, one millisecond beyond is out (also through an offset).
    let first = Utc::parse("0000-01-01T00:00:00.000Z").unwrap();
    assert_eq!(first.millis(), -62_167_219_200_000);
    assert_eq!(
        first.to_rfc3339().as_deref(),
        Some("0000-01-01T00:00:00.000Z")
    );
    let last = Utc::parse("9999-12-31T23:59:59.999Z").unwrap();
    assert_eq!(last.millis(), 253_402_300_799_999);
    assert_eq!(
        last.to_rfc3339().as_deref(),
        Some("9999-12-31T23:59:59.999Z")
    );
    assert_eq!(Utc::from_millis(first.millis() - 1).to_rfc3339(), None);
    assert_eq!(Utc::from_millis(last.millis() + 1).to_rfc3339(), None);
    assert_eq!(
        Utc::parse("0000-01-01T00:00:00+01:00").err(),
        Some(TypeProblem::OutOfRange)
    );
    assert_eq!(
        Utc::parse("9999-12-31T23:59:59.999-01:00").err(),
        Some(TypeProblem::OutOfRange)
    );
    // Documented choices: leap second refused; `-00:00`, lower case `t`/`z` and spaces around it accepted.
    assert_eq!(
        Utc::parse("2016-12-31T23:59:60Z").err(),
        Some(TypeProblem::BadDateTime)
    );
    let same = Utc::parse("2026-11-16T09:30:05.000Z").unwrap();
    for ok in [
        "2026-11-16T09:30:05-00:00",
        "2026-11-16t09:30:05z",
        "  2026-11-16T09:30:05Z  ",
        "2026-11-16T10:30:05+01:00",
    ] {
        assert_eq!(Utc::parse(ok).unwrap(), same, "{ok:?}");
    }
    // What `to_rfc3339` writes is read back to the same instant.
    for ms in [
        -62_167_219_200_000,
        -1,
        0,
        1,
        1_794_821_405_123,
        253_402_300_799_999,
    ] {
        let u = Utc::from_millis(ms);
        assert_eq!(Utc::parse(&u.to_rfc3339().unwrap()).unwrap(), u);
    }
}

#[test]
fn values_differ_where_they_should() {
    let id = Reference::parse("urn:uuid:0192f5a0-7b1c-7cde-8f00-123456789abc").unwrap();
    let p = PersonRef {
        id: id.clone(),
        art: PersonArt::Person,
        abzug: Some(ObjectId([7; 16])),
    };
    assert_ne!(
        p,
        PersonRef {
            abzug: None,
            ..p.clone()
        }
    );
    assert_ne!(
        p,
        PersonRef {
            art: PersonArt::Group,
            ..p.clone()
        }
    );
    assert_ne!(Attendee::Resource(id.clone()), Attendee::Location(id));
    assert!(Revision(2) > Revision(1) && ObjectId([1; 16]) < ObjectId([2; 16]));
    assert_ne!(PageCursor("a".into()), PageCursor("b".into()));
}
