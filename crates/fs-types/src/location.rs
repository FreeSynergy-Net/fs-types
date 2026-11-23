//! `Location`: a named place, the JSCalendar `Location` (RFC 8984 section 4.2.5) as one shared type.
//!
//! Pattern: Value object, valid on construction (`Location::new`, then the `with_*` steps; the name is optional as in RFC 8984), Adapter to the JSCalendar form
//! ([`LocationParts`], plain data: the JSON mapping itself is the job of `fs-jscalendar`). Beside the name, the
//! description, the location types, the coordinates and the links of JSCalendar it holds a postal [`Address`] (an
//! addition of ours: JSCalendar has none), and `relativeTo` ([`RelativeTo`]) and `timeZone` ([`Zone`]). The coordinates are a [`GeoPoint`]; on the wire they are a [`Place`] (a `geo:`
//! URI, JSCalendar `coordinates`), so there is no new wire kind: a `Location` is a structure over the kinds that exist.
//! `Place` stays the point and nothing else (ADR 0085: one meaning for each wire kind).

use std::collections::BTreeSet;

use strum::{AsRefStr, EnumString};

use crate::kind::registry_word;
use crate::value::{bad_char, one_line};
use crate::{Address, GeoPoint, Link, LocationParts, Place, TypeProblem, Zone};

/// Longest name, in characters.
const NAME_MAX: usize = 1024;
/// Longest description, in characters.
const DESCRIPTION_MAX: usize = 8192;
/// Longest location type, in characters.
const TYPE_MAX: usize = 128;

/// A named place: a meeting room, a shop, a trail head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    name: Option<String>,
    description: Option<String>,
    relative_to: Option<RelativeTo>,
    time_zone: Option<Zone>,
    types: BTreeSet<String>,
    coordinates: Option<GeoPoint>,
    address: Option<Address>,
    links: Vec<Link>,
}

/// Where a location is relative to its event (RFC 8984 section 4.2.5 `relativeTo`): its `start` or its `end`, or a word of
/// the registry we do not know (a newer IANA value or a vendor value). RFC 8984 says a value the reader does not
/// understand counts as if the property were omitted, so `Other` means "not said" for a calendar; it is kept and written
/// back unchanged (read everything). The alert trigger of `fs-jscalendar` is a different, closed set (section 4.5.2) and
/// is not this type. The pattern is the one of [`crate::NameKind`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
#[non_exhaustive]
pub enum RelativeTo {
    /// `start`.
    Start,
    /// `end`.
    End,
    /// A word that is not in the list, kept as read.
    #[strum(default, transparent)]
    Other(String),
}
registry_word!(RelativeTo);

fn location_type(typed: &str) -> Result<String, TypeProblem> {
    let t = typed.trim();
    if t.is_empty() {
        return Err(TypeProblem::Empty);
    }
    if t.chars().any(|c| bad_char(c) || c.is_whitespace()) {
        return Err(TypeProblem::Character);
    }
    if t.chars().count() > TYPE_MAX {
        return Err(TypeProblem::TooLong);
    }
    Ok(t.to_owned())
}

impl Location {
    /// A location with nothing in it (the `name` is optional in RFC 8984 section 4.2.5).
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            relative_to: None,
            time_zone: None,
            types: BTreeSet::new(),
            coordinates: None,
            address: None,
            links: Vec::new(),
        }
    }

    /// With a name (one line; empty text removes it).
    ///
    /// # Errors
    /// `Character`, `TooLong` as for a one-line text.
    pub fn with_name(self, name: &str) -> Result<Self, TypeProblem> {
        let name = if name.trim().is_empty() {
            None
        } else {
            Some(one_line(name, NAME_MAX)?)
        };
        Ok(Self { name, ..self })
    }

    /// With the position relative to the event (`None` removes it). An `Other` word is kept as it is (a known word in
    /// `Other` becomes its variant).
    ///
    /// # Errors
    /// `Empty`, `Character`, `TooLong` for the word of an `Other`.
    pub fn with_relative_to(self, relative_to: Option<RelativeTo>) -> Result<Self, TypeProblem> {
        let relative_to = relative_to.map(RelativeTo::checked).transpose()?;
        Ok(Self {
            relative_to,
            ..self
        })
    }

    /// With a time zone (`None` removes it): the zone the location's own times are in (RFC 8984 `timeZone`).
    #[must_use]
    pub fn with_time_zone(self, time_zone: Option<Zone>) -> Self {
        Self { time_zone, ..self }
    }

    /// With a description (lines allowed; empty text removes it).
    ///
    /// # Errors
    /// `Character`, `TooLong`.
    pub fn with_description(self, description: &str) -> Result<Self, TypeProblem> {
        let t = description.replace("\r\n", "\n").replace('\r', "\n");
        let t = t.trim();
        if t.chars().any(|c| bad_char(c) && c != '\n' && c != '\t') {
            return Err(TypeProblem::Character);
        }
        if t.chars().count() > DESCRIPTION_MAX {
            return Err(TypeProblem::TooLong);
        }
        let description = (!t.is_empty()).then(|| t.to_owned());
        Ok(Self {
            description,
            ..self
        })
    }

    /// With location types (duplicates collapse; the old set is replaced).
    ///
    /// # Errors
    /// `Empty`, `Character` (white space or a control character), `TooLong` for any of them.
    pub fn with_location_types(self, types: &[&str]) -> Result<Self, TypeProblem> {
        let types = types
            .iter()
            .map(|t| location_type(t))
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(Self { types, ..self })
    }

    /// With coordinates (`None` removes them).
    #[must_use]
    pub fn with_coordinates(self, coordinates: Option<GeoPoint>) -> Self {
        Self {
            coordinates,
            ..self
        }
    }

    /// With a postal address (`None` removes it).
    #[must_use]
    pub fn with_address(self, address: Option<Address>) -> Self {
        Self { address, ..self }
    }

    /// With links (the old list is replaced; the same link twice is kept once, in order of first appearance).
    #[must_use]
    pub fn with_links(self, links: &[Link]) -> Self {
        let mut seen: Vec<Link> = Vec::new();
        for l in links {
            if !seen.contains(l) {
                seen.push(l.clone());
            }
        }
        Self {
            links: seen,
            ..self
        }
    }

    /// The name, if any, never empty.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Relative to the start or the end of the event (or a word we do not know), if said.
    #[must_use]
    pub fn relative_to(&self) -> Option<&RelativeTo> {
        self.relative_to.as_ref()
    }

    /// The time zone, if any.
    #[must_use]
    pub fn time_zone(&self) -> Option<&Zone> {
        self.time_zone.as_ref()
    }

    /// The description, if any.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// The location types, sorted.
    pub fn location_types(&self) -> impl Iterator<Item = &str> {
        self.types.iter().map(String::as_str)
    }

    /// The coordinates, if known.
    #[must_use]
    pub fn coordinates(&self) -> Option<&GeoPoint> {
        self.coordinates.as_ref()
    }

    /// The coordinates as the wire kind [`Place`], if known.
    #[must_use]
    pub fn place(&self) -> Option<Place> {
        self.coordinates.as_ref().map(Place::from_point)
    }

    /// The postal address, if any.
    #[must_use]
    pub fn address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    /// The links.
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// The JSCalendar form of this location, ready for the JSON mapping.
    #[must_use]
    pub fn to_parts(&self) -> LocationParts {
        LocationParts::from_location(self)
    }

    /// Read the JSCalendar form: every part is checked again, so a `Location` is valid whatever the JSON said.
    ///
    /// # Errors
    /// The first part that breaks its rule: `Character`, `TooLong` (name, description, types, and the word of a
    /// `relativeTo`; an empty `relativeTo` counts as not set), `BadZone`, a `geo:` problem (`BadPlace`, `PlaceRange`,
    /// `OutOfRange`), a link problem (`BadUri`, `NotAbsolute`, `UnsafeScheme`).
    pub fn from_parts(parts: &LocationParts) -> Result<Self, TypeProblem> {
        parts.to_location()
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}
