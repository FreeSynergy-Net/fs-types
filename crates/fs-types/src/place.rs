//! `Place`: a point as a `geo:` URI (RFC 5870).
//!
//! Pattern: Value object over [`GeoPoint`] (the one core of a coordinate), Adapter to the wire form (`BasicType::Place`,
//! a `geo:` text). `parse` and `canonical` go through `GeoPoint`, so a text and its normal form are one value:
//! `geo:0,180` is `geo:0,-180`, `geo:-0,-0` is `geo:0,0`, a parameter other than `u=` is dropped. WGS-84 only (the only
//! coordinate system RFC 5870 defines). Shapes (`GeoJSON` lines and areas) are `Unsupported` here (DESIGN.md). The named
//! wrapper (name, address, links) is [`crate::Location`].

use fs_interfaces::types_capnp::BasicType;

use crate::{BasicValue, GeoPoint, TypeProblem};

/// A point on Earth: `geo:48.2010,16.3695` with optional altitude and uncertainty.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Place(GeoPoint);

impl Place {
    /// Latitude in degrees, -90 to 90.
    #[must_use]
    pub fn latitude(&self) -> f64 {
        self.0.lat()
    }

    /// Longitude in degrees, -180 (included) to 180 (excluded).
    #[must_use]
    pub fn longitude(&self) -> f64 {
        self.0.lon()
    }

    /// The coordinate of this place as a [`GeoPoint`] (height and uncertainty kept).
    ///
    /// # Errors
    /// None any more (a `Place` always holds a valid `GeoPoint`); the `Result` stays for the callers of v0.0.24.
    pub fn point(&self) -> Result<GeoPoint, TypeProblem> {
        Ok(self.0)
    }

    /// The place for a [`GeoPoint`]. Both ways lose nothing: `GeoPoint` to `Place` to `GeoPoint`, and `Place` to
    /// `GeoPoint` to `Place` (since v0.0.25 the text is normalised on the way in).
    #[must_use]
    pub fn from_point(point: &GeoPoint) -> Self {
        Self(*point)
    }
}

impl BasicValue for Place {
    const KIND: BasicType = BasicType::Place;

    /// # Errors
    /// As [`GeoPoint::from_geo_uri`]: `Empty`, `BadPlace`, `PlaceRange`, `OutOfRange` (also a negative `u=`).
    fn parse(typed: &str) -> Result<Self, TypeProblem> {
        GeoPoint::from_geo_uri(typed).map(Self)
    }

    fn canonical(&self) -> String {
        self.0.to_geo_uri()
    }
}
