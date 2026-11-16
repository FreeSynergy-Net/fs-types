//! `Location` (JSCalendar shape) and the `Place` / `GeoPoint` normalisation in both directions.

use fs_types::{
    Address, AddressKind, BasicValue, GeoPoint, Link, Location, LocationParts, Place, TypeProblem,
};

fn full_location() -> Location {
    let addr = Address::new(
        &[(AddressKind::Name, "Ring"), (AddressKind::Number, "1")],
        "at",
        "",
    )
    .unwrap();
    Location::new("  Room 4  ")
        .unwrap()
        .with_description("Second floor\r\nbehind the lift")
        .unwrap()
        .with_location_types(&["office", "meeting-room", "office"])
        .unwrap()
        .with_coordinates(Some(
            GeoPoint::new(48.2, 16.37, Some(171.0), Some(5.0)).unwrap(),
        ))
        .with_address(Some(addr))
        .with_links(&[
            Link::parse("https://Example.org/a/../room").unwrap(),
            Link::parse("https://example.org/room").unwrap(),
        ])
}

#[test]
fn name_is_required_and_checked() {
    assert_eq!(Location::new("  ").err(), Some(TypeProblem::Empty));
    assert_eq!(Location::new("a\u{7}b").err(), Some(TypeProblem::Character));
    assert_eq!(
        Location::new(&"x".repeat(1025)).err(),
        Some(TypeProblem::TooLong)
    );
    let l = Location::new(" Café ").unwrap();
    assert_eq!(l.name(), "Café");
    assert!(l.description().is_none() && l.coordinates().is_none() && l.address().is_none());
    assert!(l.links().is_empty() && l.location_types().count() == 0 && l.place().is_none());
}

#[test]
fn parts_are_normalised_and_checked() {
    let l = full_location();
    assert_eq!(l.name(), "Room 4");
    assert_eq!(l.description(), Some("Second floor\nbehind the lift"));
    assert_eq!(
        l.location_types().collect::<Vec<_>>(),
        ["meeting-room", "office"]
    );
    assert_eq!(l.links().len(), 1, "same link twice once");
    assert_eq!(
        Location::new("x")
            .unwrap()
            .with_location_types(&["a b"])
            .err(),
        Some(TypeProblem::Character)
    );
    assert_eq!(
        Location::new("x")
            .unwrap()
            .with_location_types(&[" "])
            .err(),
        Some(TypeProblem::Empty)
    );
    assert_eq!(
        Location::new("x")
            .unwrap()
            .with_description("a\u{202E}b")
            .err(),
        Some(TypeProblem::Character)
    );
    // An empty description removes it.
    assert!(l.with_description("  ").unwrap().description().is_none());
}

#[test]
fn coordinates_travel_as_place() {
    let l = full_location();
    assert_eq!(l.place().unwrap().canonical(), "geo:48.2,16.37,171;u=5");
    assert_eq!(
        l.place().unwrap().point().unwrap(),
        *l.coordinates().unwrap()
    );
}

#[test]
fn parts_round_trip_both_ways() {
    let l = full_location();
    let parts = l.to_parts();
    assert_eq!(parts.coordinates.as_deref(), Some("geo:48.2,16.37,171;u=5"));
    assert_eq!(parts.links, ["https://example.org/room"]);
    assert_eq!(Location::from_parts(&parts).unwrap(), l);
    assert_eq!(Location::from_parts(&parts).unwrap().to_parts(), parts);
    let minimal = Location::new("Home").unwrap();
    assert_eq!(Location::from_parts(&minimal.to_parts()).unwrap(), minimal);
}

#[test]
fn from_parts_checks_every_part() {
    let base = Location::new("Home").unwrap().to_parts();
    let with = |f: fn(&mut LocationParts)| {
        let mut p = base.clone();
        f(&mut p);
        Location::from_parts(&p).err()
    };
    assert_eq!(with(|p| p.name = " ".into()), Some(TypeProblem::Empty));
    assert_eq!(
        with(|p| p.coordinates = Some("geo:91,0".into())),
        Some(TypeProblem::PlaceRange)
    );
    assert_eq!(
        with(|p| p.coordinates = Some("52,13".into())),
        Some(TypeProblem::BadPlace)
    );
    assert_eq!(
        with(|p| p.links = vec!["javascript:x".into()]),
        Some(TypeProblem::UnsafeScheme)
    );
    assert_eq!(
        with(|p| p.links = vec!["/rel".into()]),
        Some(TypeProblem::NotAbsolute)
    );
    assert_eq!(
        with(|p| p.location_types = vec!["a b".into()]),
        Some(TypeProblem::Character)
    );
}

#[test]
fn place_text_is_normalised_through_the_point() {
    assert_eq!(Place::parse("geo:0,180").unwrap().canonical(), "geo:0,-180");
    assert_eq!(Place::parse("geo:-0,-0").unwrap().canonical(), "geo:0,0");
    assert_eq!(
        Place::parse("geo:1,2;crs=wgs84;foo=bar")
            .unwrap()
            .canonical(),
        "geo:1,2"
    );
    assert_eq!(
        Place::parse("geo:0,180").unwrap(),
        Place::parse("geo:0,-180").unwrap()
    );
}

#[test]
fn place_and_point_round_trip_in_both_directions() {
    for text in [
        "geo:0,180",
        "geo:-0,-0",
        "geo:48.2,16.37,171;u=5",
        "geo:-90,-180",
        "geo:90,179.5;u=0",
    ] {
        let place = Place::parse(text).unwrap();
        // Place -> GeoPoint -> Place
        assert_eq!(Place::from_point(&place.point().unwrap()), place, "{text}");
        // and the text is a fixed point of parse/canonical
        assert_eq!(Place::parse(&place.canonical()).unwrap(), place, "{text}");
        assert_eq!(
            Place::parse(&place.canonical()).unwrap().canonical(),
            place.canonical()
        );
    }
    for (lat, lon, alt, u) in [
        (0.0, 180.0, None, None),
        (-0.0, -0.0, Some(-0.0), Some(0.0)),
        (52.5, 13.4, Some(34.0), Some(12.5)),
    ] {
        let point = GeoPoint::new(lat, lon, alt, u).unwrap();
        // GeoPoint -> Place -> GeoPoint
        let place = Place::from_point(&point);
        assert_eq!(place.point().unwrap(), point);
        assert_eq!(GeoPoint::from_geo_uri(&place.canonical()).unwrap(), point);
    }
}

#[test]
fn negative_uncertainty_is_the_same_problem_for_point_and_place() {
    let g = GeoPoint::from_geo_uri("geo:1,2;u=-1").err();
    let p = Place::parse("geo:1,2;u=-1").err();
    assert_eq!(g, p);
    assert_eq!(GeoPoint::new(1.0, 2.0, None, Some(-1.0)).err(), g);
    assert_eq!(g, Some(TypeProblem::OutOfRange));
    assert_eq!(
        Place::parse("geo:nan,0").err(),
        GeoPoint::from_geo_uri("geo:nan,0").err()
    );
}
