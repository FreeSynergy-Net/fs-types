//! `GeoPoint`, `PositionFix`, `AccuracyLevel`: range checks, `geo:` URI round trip, `GeoJSON` order, blur.

use std::collections::HashSet;

use fs_types::{
    AccuracyLevel, BasicValue, FixSource, GeoPoint, Place, PositionFix, TypeProblem, Utc,
};

const LEVELS: [AccuracyLevel; 4] = [
    AccuracyLevel::M100,
    AccuracyLevel::Km1,
    AccuracyLevel::Km10,
    AccuracyLevel::Region,
];

fn metres(a: &GeoPoint, b: &GeoPoint) -> f64 {
    let (p1, p2) = (a.lat().to_radians(), b.lat().to_radians());
    let dl = (b.lon() - a.lon()).to_radians();
    let h = ((p2 - p1) / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    2.0 * 6_371_000.0 * h.sqrt().asin()
}

#[test]
fn range_and_finiteness_are_checked() {
    assert!(GeoPoint::from_lat_lon(90.0, -180.0).is_ok());
    assert_eq!(
        GeoPoint::from_lat_lon(90.1, 0.0).err(),
        Some(TypeProblem::PlaceRange)
    );
    assert_eq!(
        GeoPoint::from_lat_lon(0.0, 180.1).err(),
        Some(TypeProblem::PlaceRange)
    );
    assert_eq!(
        GeoPoint::from_lat_lon(0.0, -180.1).err(),
        Some(TypeProblem::PlaceRange)
    );
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            GeoPoint::from_lat_lon(bad, 0.0).err(),
            Some(TypeProblem::OutOfRange)
        );
        assert_eq!(
            GeoPoint::from_lat_lon(0.0, bad).err(),
            Some(TypeProblem::OutOfRange)
        );
        assert_eq!(
            GeoPoint::new(0.0, 0.0, Some(bad), None).err(),
            Some(TypeProblem::OutOfRange)
        );
        assert_eq!(
            GeoPoint::new(0.0, 0.0, None, Some(bad)).err(),
            Some(TypeProblem::OutOfRange)
        );
    }
    assert_eq!(
        GeoPoint::new(0.0, 0.0, None, Some(-1.0)).err(),
        Some(TypeProblem::OutOfRange)
    );
}

#[test]
fn lon_180_is_minus_180_and_negative_zero_is_zero() {
    let p = GeoPoint::from_lat_lon(1.0, 180.0).unwrap();
    assert_eq!(p, GeoPoint::from_lat_lon(1.0, -180.0).unwrap());
    let z = GeoPoint::from_lat_lon(-0.0, -0.0).unwrap();
    assert_eq!(z, GeoPoint::from_lat_lon(0.0, 0.0).unwrap());
    let mut set = HashSet::new();
    set.insert(z);
    assert!(set.contains(&GeoPoint::from_lat_lon(0.0, 0.0).unwrap()));
    assert_ne!(
        GeoPoint::new(1.0, 1.0, None, None).unwrap(),
        GeoPoint::new(1.0, 1.0, Some(0.0), None).unwrap()
    );
}

#[test]
fn geo_uri_round_trip_with_height_and_uncertainty() {
    let p = GeoPoint::new(48.2010, 16.3695, Some(170.5), Some(20.0)).unwrap();
    let text = p.to_geo_uri();
    assert_eq!(text, "geo:48.201,16.3695,170.5;u=20");
    assert_eq!(GeoPoint::from_geo_uri(&text).unwrap(), p);
    let plain = GeoPoint::from_geo_uri(" geo:-33.8688,151.2093 ").unwrap();
    assert_eq!(plain.to_geo_uri(), "geo:-33.8688,151.2093");
    assert_eq!(plain.alt(), None);
    let u = GeoPoint::from_geo_uri("geo:1,2;crs=wgs84;u=5.5").unwrap();
    assert_eq!(u.uncertainty(), Some(5.5));
}

#[test]
fn geo_uri_refusals() {
    assert_eq!(GeoPoint::from_geo_uri(" ").err(), Some(TypeProblem::Empty));
    assert_eq!(
        GeoPoint::from_geo_uri("48,16").err(),
        Some(TypeProblem::BadPlace)
    );
    assert_eq!(
        GeoPoint::from_geo_uri("geo:1,2;crs=other").err(),
        Some(TypeProblem::BadPlace)
    );
    assert_eq!(
        GeoPoint::from_geo_uri("geo:91,0").err(),
        Some(TypeProblem::PlaceRange)
    );
    assert_eq!(
        GeoPoint::from_geo_uri("geo:nan,0").err(),
        Some(TypeProblem::PlaceRange)
    );
    assert_eq!(
        GeoPoint::from_geo_uri("geo:inf,0").err(),
        Some(TypeProblem::PlaceRange)
    );
    assert_eq!(
        GeoPoint::from_geo_uri("geo:1,2;u=-3").err(),
        Some(TypeProblem::OutOfRange)
    );
}

#[test]
fn geojson_order_is_lon_lat() {
    let p = GeoPoint::new(48.2, 16.37, Some(170.0), Some(9.0)).unwrap();
    assert_eq!(p.to_geojson_position(), vec![16.37, 48.2, 170.0]);
    let back = GeoPoint::from_geojson_position(&[16.37, 48.2]).unwrap();
    assert_eq!((back.lat(), back.lon()), (48.2, 16.37));
    assert_eq!(
        GeoPoint::from_geojson_position(&[1.0]).err(),
        Some(TypeProblem::BadPlace)
    );
    assert_eq!(
        GeoPoint::from_geojson_position(&[0.0, 95.0]).err(),
        Some(TypeProblem::PlaceRange)
    );
}

#[test]
fn blur_stays_in_the_level_and_is_idempotent() {
    let samples = [
        (48.2010, 16.3695),
        (-33.8688, 151.2093),
        (0.0, 0.0),
        (89.9999, 179.9999),
        (-90.0, -180.0),
        (90.0, 180.0),
        (64.1466, -21.9426),
    ];
    for (lat, lon) in samples {
        let p = GeoPoint::new(lat, lon, Some(300.0), Some(3.0)).unwrap();
        for level in LEVELS {
            let b = level.blur(p);
            assert!(metres(&p, &b) <= level.radius_m(), "{lat},{lon} {level:?}");
            assert_eq!(level.blur(b), b, "idempotent {lat},{lon} {level:?}");
            assert_eq!(b.alt(), None);
            assert!(b.uncertainty().unwrap() >= level.radius_m());
            assert_eq!(level.blur(p), b, "deterministic");
        }
        assert_eq!(AccuracyLevel::Exact.blur(p), p);
    }
}

#[test]
fn nearby_points_share_a_cell_and_coarser_is_coarser() {
    let a = GeoPoint::from_lat_lon(48.20101, 16.36951).unwrap();
    let b = GeoPoint::from_lat_lon(48.20102, 16.36952).unwrap();
    assert_eq!(AccuracyLevel::Km1.blur(a), AccuracyLevel::Km1.blur(b));
    assert!(AccuracyLevel::M100 < AccuracyLevel::Region);
}

#[test]
fn fix_records_the_level_and_never_gets_finer() {
    let p = GeoPoint::from_lat_lon(48.2010, 16.3695).unwrap();
    let fix = PositionFix {
        point: p,
        source: FixSource::Gnss,
        measured_at: Some(Utc::from_millis(1_700_000_000_000)),
        blur: None,
    };
    let km = fix.at_level(AccuracyLevel::Km1);
    assert_eq!(km.blur, Some(AccuracyLevel::Km1));
    assert_eq!(km.point, AccuracyLevel::Km1.blur(p));
    assert_eq!(km.source, FixSource::Gnss);
    assert_eq!(km.at_level(AccuracyLevel::M100), km);
    assert_eq!(km.at_level(AccuracyLevel::Km1), km);
    assert_eq!(
        km.at_level(AccuracyLevel::Km10).blur,
        Some(AccuracyLevel::Km10)
    );
}

#[test]
fn place_gives_and_takes_a_geo_point() {
    let p = GeoPoint::new(48.2, 16.37, Some(5.0), Some(10.0)).unwrap();
    let place = Place::from_point(&p);
    assert_eq!(place.canonical(), p.to_geo_uri());
    assert_eq!(place.point().unwrap(), p);
}
