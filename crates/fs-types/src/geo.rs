//! `GeoPoint`: a coordinate on Earth (WGS 84), nothing else.
//!
//! Pattern: Value object (valid on construction, equal values are equal bit for bit) and Adapter over the free crate
//! `geo-uri` for reading RFC 5870. A `GeoPoint` is only the coordinate: latitude, longitude, optional height and
//! optional uncertainty. It has no name and no address; that is the job of [`crate::Location`] (the wrapper; `Place` is the point as wire form,
//! plan fs-maps E0). Inside a program latitude comes first (ISO 6709, RFC 5870); the `GeoJSON` order longitude, latitude
//! (RFC 7946 section 3.1.1) exists only as the helpers [`GeoPoint::to_geojson_position`] and
//! [`GeoPoint::from_geojson_position`] for the border.

use std::hash::{Hash, Hasher};
use std::str::FromStr;

use geo_uri::GeoUri;

use crate::TypeProblem;

/// A point on Earth in WGS 84: `lat` -90..90, `lon` -180..180 (180 is stored as -180), optional height above the
/// ellipsoid in metres and optional uncertainty in metres (RFC 5870 `u=`, zero or more). Always valid: NaN and
/// infinity never get in, `-0.0` is stored as `0.0`, so equality and hash by the bits of the numbers are exact.
#[derive(Clone, Copy, Debug)]
pub struct GeoPoint {
    lat: f64,
    lon: f64,
    alt: Option<f64>,
    uncertainty: Option<f64>,
}

/// `-0.0 + 0.0` is `0.0` (IEEE 754): one zero, so the bits of equal numbers are equal.
fn zero_fix(x: f64) -> f64 {
    x + 0.0
}

impl GeoPoint {
    /// Make a point, checking every number.
    ///
    /// # Errors
    /// `PlaceRange` (latitude outside -90..90 or longitude outside -180..180), `OutOfRange` (NaN or infinity anywhere,
    /// or a negative uncertainty).
    pub fn new(
        lat: f64,
        lon: f64,
        alt: Option<f64>,
        uncertainty: Option<f64>,
    ) -> Result<Self, TypeProblem> {
        let all_finite = lat.is_finite()
            && lon.is_finite()
            && alt.is_none_or(f64::is_finite)
            && uncertainty.is_none_or(f64::is_finite);
        if !all_finite || uncertainty.is_some_and(|u| u < 0.0) {
            return Err(TypeProblem::OutOfRange);
        }
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
            return Err(TypeProblem::PlaceRange);
        }
        let lon = if lon.to_bits() == 180.0_f64.to_bits() {
            -180.0
        } else {
            lon
        };
        Ok(Self {
            lat: zero_fix(lat),
            lon: zero_fix(lon),
            alt: alt.map(zero_fix),
            uncertainty: uncertainty.map(zero_fix),
        })
    }

    /// A point of latitude and longitude only.
    ///
    /// # Errors
    /// As [`GeoPoint::new`].
    pub fn from_lat_lon(lat: f64, lon: f64) -> Result<Self, TypeProblem> {
        Self::new(lat, lon, None, None)
    }

    /// Latitude in degrees, -90 to 90.
    #[must_use]
    pub fn lat(&self) -> f64 {
        self.lat
    }

    /// Longitude in degrees, -180 (included) to 180 (excluded).
    #[must_use]
    pub fn lon(&self) -> f64 {
        self.lon
    }

    /// Height in metres above the WGS 84 ellipsoid, if known.
    #[must_use]
    pub fn alt(&self) -> Option<f64> {
        self.alt
    }

    /// Uncertainty in metres, if known.
    #[must_use]
    pub fn uncertainty(&self) -> Option<f64> {
        self.uncertainty
    }

    /// The same point with another uncertainty (`None` removes it).
    ///
    /// # Errors
    /// `OutOfRange` (NaN, infinity or negative).
    pub fn with_uncertainty(self, uncertainty: Option<f64>) -> Result<Self, TypeProblem> {
        Self::new(self.lat, self.lon, self.alt, uncertainty)
    }

    /// The same point without height.
    #[must_use]
    pub fn without_alt(self) -> Self {
        Self { alt: None, ..self }
    }

    /// Read a `geo:` URI (RFC 5870): `geo:48.2,16.37`, with height `geo:48.2,16.37,170` and uncertainty `;u=20`.
    /// Only WGS 84 (`crs=wgs84` or none); other parameters are dropped.
    ///
    /// # Errors
    /// `Empty`, `BadPlace` (not a `geo:` URI or another coordinate system), `PlaceRange`, `OutOfRange`.
    pub fn from_geo_uri(text: &str) -> Result<Self, TypeProblem> {
        let t = text.trim();
        if t.is_empty() {
            return Err(TypeProblem::Empty);
        }
        let uri = GeoUri::from_str(t).map_err(|e| match e {
            geo_uri::Error::OutOfRangeLatitude | geo_uri::Error::OutOfRangeLongitude => {
                TypeProblem::PlaceRange
            }
            geo_uri::Error::OutOfRangeUncertainty => TypeProblem::OutOfRange,
            _ => TypeProblem::BadPlace,
        })?;
        Self::new(
            uri.latitude(),
            uri.longitude(),
            uri.altitude(),
            uri.uncertainty(),
        )
    }

    /// The `geo:` URI (RFC 5870): `geo:<lat>,<lon>[,<alt>][;u=<uncertainty>]`. Reading it back gives the same point.
    #[must_use]
    pub fn to_geo_uri(&self) -> String {
        let alt = self.alt.map_or_else(String::new, |a| format!(",{a}"));
        let unc = self
            .uncertainty
            .map_or_else(String::new, |u| format!(";u={u}"));
        format!("geo:{},{}{alt}{unc}", self.lat, self.lon)
    }

    /// The `GeoJSON` position (RFC 7946 section 3.1.1): longitude, latitude, then height if known. Only for the
    /// border to `GeoJSON`; uncertainty has no place in a position and is dropped.
    #[must_use]
    pub fn to_geojson_position(&self) -> Vec<f64> {
        let mut v = vec![self.lon, self.lat];
        v.extend(self.alt);
        v
    }

    /// Read a `GeoJSON` position: `[lon, lat]` or `[lon, lat, alt]` (RFC 7946 section 3.1.1).
    ///
    /// # Errors
    /// `BadPlace` (not two or three numbers), then as [`GeoPoint::new`].
    pub fn from_geojson_position(position: &[f64]) -> Result<Self, TypeProblem> {
        match *position {
            [lon, lat] => Self::new(lat, lon, None, None),
            [lon, lat, alt] => Self::new(lat, lon, Some(alt), None),
            _ => Err(TypeProblem::BadPlace),
        }
    }

    fn bits(&self) -> [u64; 4] {
        let o = |x: Option<f64>| x.map_or(u64::MAX, f64::to_bits);
        [
            self.lat.to_bits(),
            self.lon.to_bits(),
            o(self.alt),
            o(self.uncertainty),
        ]
    }
}

impl PartialEq for GeoPoint {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits()
    }
}

impl Eq for GeoPoint {}

impl Hash for GeoPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bits().hash(state);
    }
}
