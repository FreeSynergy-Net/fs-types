//! `PositionFix`, `FixSource`, `AccuracyLevel`: a position with where it came from, and how coarse it may be shown.
//!
//! Pattern: Value object (plain data, valid because its `GeoPoint` is). Strategy by level: [`AccuracyLevel::blur`]
//! is a fixed rule per level. The blur is deterministic on purpose, with no randomness: the same point and level
//! always give the same result, so two people who share a place at the same level see the same cell, and blurring a
//! blurred point changes nothing. The plan calls the type `PositionFix` (fs-maps 2.0.4); the levels are the release
//! steps of a location share (fs-maps question 2.0.5: exact, about 100 m, 1 km, 10 km, region).

use crate::{GeoPoint, Utc};

/// Where a position came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FixSource {
    /// Typed or set on a map by a person.
    Manual,
    /// Satellite navigation (GPS, Galileo, GLONASS).
    Gnss,
    /// Derived from network, Wi-Fi or cell data.
    Network,
    /// Read from a file or another program.
    Import,
    /// Found by a search service from an address.
    Geocoder,
    /// Computed from other positions (middle of a route, centre of an area).
    Derived,
}

/// How exact a position may be shown: the release step of a share.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum AccuracyLevel {
    /// As measured.
    Exact,
    /// About 100 m.
    M100,
    /// About 1 km.
    Km1,
    /// About 10 km (a town).
    Km10,
    /// About 100 km (a region).
    Region,
}

impl AccuracyLevel {
    /// The most the blur moves a point, in metres (nominal: the cell is smaller than this on every latitude and the
    /// point goes to the middle of its cell). `0.0` for `Exact`. The cell is a grid in degrees: its east-west width
    /// shrinks with cos(latitude) (at 60 degrees north a `Km1` cell is 1.1 km by 0.56 km). The levels are upper
    /// bounds of the shift; the limit for AI is the city (`Km10`, about 10 km; user decision 16.11.) and is not enforced by this type, it belongs in
    /// `fs-access`. Routing and directions without an AI may be exact.
    #[must_use]
    pub fn radius_m(self) -> f64 {
        match self {
            Self::Exact => 0.0,
            Self::M100 => 100.0,
            Self::Km1 => 1_000.0,
            Self::Km10 => 10_000.0,
            Self::Region => 100_000.0,
        }
    }

    /// Grid step in degrees (0.001 degree of latitude is about 111 m).
    fn step_deg(self) -> Option<f64> {
        match self {
            Self::Exact => None,
            Self::M100 => Some(0.001),
            Self::Km1 => Some(0.01),
            Self::Km10 => Some(0.1),
            Self::Region => Some(1.0),
        }
    }

    /// Blur `point` to this level: latitude and longitude go to the middle of their grid cell, height is dropped,
    /// and the uncertainty becomes at least [`AccuracyLevel::radius_m`]. `Exact` returns the point unchanged.
    /// No randomness; blurring twice equals blurring once.
    #[must_use]
    pub fn blur(self, point: GeoPoint) -> GeoPoint {
        let Some(step) = self.step_deg() else {
            return point;
        };
        let lat = cell_middle(point.lat(), -90.0, 180.0, step);
        let lon = cell_middle(point.lon(), -180.0, 360.0, step);
        let u = point.uncertainty().unwrap_or(0.0).max(self.radius_m());
        // The middle of a cell is always in range and finite, so this never fails. It must not fall back to the
        // input: that would let the unblurred point through.
        GeoPoint::new(lat, lon, None, Some(u))
            .unwrap_or_else(|_| unreachable!("a cell middle is a valid point"))
    }
}

/// Middle of the cell (width `step`) that holds `x`, on the range `origin` to `origin + span`; the upper edge belongs
/// to the last cell.
fn cell_middle(x: f64, origin: f64, span: f64, step: f64) -> f64 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cells = (span / step).round() as u64;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = (((x - origin) / step).floor().max(0.0) as u64).min(cells - 1);
    #[allow(clippy::cast_precision_loss)]
    let middle = (index as f64 + 0.5).mul_add(step, origin);
    // Round to the digits of the cell (one more than the step has), so the text shows 123.005, not 123.00500000000001.
    let scale = 10.0 / step;
    (middle * scale).round() / scale
}

/// A position with its origin, its measuring time and the level it was blurred to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PositionFix {
    /// The coordinate.
    pub point: GeoPoint,
    /// Where it came from.
    pub source: FixSource,
    /// When it was measured (UTC), if known.
    pub measured_at: Option<Utc>,
    /// The level it was blurred to on purpose; `None` = not blurred.
    pub blur: Option<AccuracyLevel>,
}

impl PositionFix {
    /// This fix shown at `level`: the point is blurred, `blur` records the level. A finer request than the fix
    /// already has changes nothing (blur never makes a position more exact).
    #[must_use]
    pub fn at_level(self, level: AccuracyLevel) -> Self {
        // `blur` is a public field and may claim a level the point never had: always blur (it is idempotent), at the
        // coarser of the claimed and the requested level.
        let lvl = self.blur.map_or(level, |b| b.max(level));
        Self {
            point: lvl.blur(self.point),
            blur: Some(lvl),
            ..self
        }
    }
}
