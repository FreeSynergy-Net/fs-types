//! `Place`: a point as a `geo:` URI (RFC 5870).
//!
//! Pattern: Adapter over the free crate `geo-uri` (parse and print RFC 5870), Value object around it. WGS-84 only
//! (the only coordinate system RFC 5870 defines); the crate range-checks latitude and longitude. Shapes (`GeoJSON`
//! lines and areas) are `Unsupported` here (DESIGN.md).

use std::str::FromStr;

use fs_interfaces::types_capnp::BasicType;
use geo_uri::GeoUri;

use crate::{BasicValue, GeoPoint, TypeProblem};

/// A point on Earth: `geo:48.2010,16.3695` with optional altitude and uncertainty.
#[derive(Clone, Debug, PartialEq)]
pub struct Place(GeoUri);

impl Place {
    /// Latitude in degrees, -90 to 90.
    #[must_use]
    pub fn latitude(&self) -> f64 {
        self.0.latitude()
    }

    /// Longitude in degrees, -180 to 180.
    #[must_use]
    pub fn longitude(&self) -> f64 {
        self.0.longitude()
    }
}

impl Place {
    /// The coordinate of this place as a [`GeoPoint`] (height and uncertainty kept).
    ///
    /// # Errors
    /// `OutOfRange` if the stored numbers are not finite (a `geo:` text such as `geo:nan,0`).
    pub fn point(&self) -> Result<GeoPoint, TypeProblem> {
        GeoPoint::new(
            self.0.latitude(),
            self.0.longitude(),
            self.0.altitude(),
            self.0.uncertainty(),
        )
    }

    /// The place for a [`GeoPoint`]; no name, no address (the wrapper with those is planned, see DESIGN.md v0.0.24).
    #[must_use]
    pub fn from_point(point: &GeoPoint) -> Self {
        // A GeoPoint is valid, so its own text always reads back.
        Self::parse(&point.to_geo_uri()).unwrap_or_else(|_| unreachable!("valid GeoPoint"))
    }
}

impl BasicValue for Place {
    const KIND: BasicType = BasicType::Place;

    fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let t = typed.trim();
        if t.is_empty() {
            return Err(TypeProblem::Empty);
        }
        // The crate range-checks latitude and longitude itself; we only tell the person which kind of error it was.
        GeoUri::from_str(t).map(Self).map_err(|e| match e {
            geo_uri::Error::OutOfRangeLatitude | geo_uri::Error::OutOfRangeLongitude => {
                TypeProblem::PlaceRange
            }
            _ => TypeProblem::BadPlace,
        })
    }

    fn canonical(&self) -> String {
        self.0.to_string()
    }
}
