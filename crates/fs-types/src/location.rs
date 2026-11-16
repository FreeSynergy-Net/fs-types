//! `Location`: a named place, the JSCalendar `Location` (RFC 8984 section 4.2.5) as one shared type.
//!
//! Pattern: Value object, valid on construction (`Location::new`, then the `with_*` steps), Adapter to the JSCalendar form
//! ([`LocationParts`], plain data: the JSON mapping itself is the job of `fs-jscalendar`). Beside the name, the
//! description, the location types, the coordinates and the links of JSCalendar it holds a postal [`Address`] (an
//! addition of ours: JSCalendar has none). The coordinates are a [`GeoPoint`]; on the wire they are a [`Place`] (a `geo:`
//! URI, JSCalendar `coordinates`), so there is no new wire kind: a `Location` is a structure over the kinds that exist.
//! `Place` stays the point and nothing else (ADR 0085: one meaning for each wire kind).

use std::collections::BTreeSet;

use crate::value::{bad_char, one_line};
use crate::{Address, BasicValue, GeoPoint, Link, Place, TypeProblem};

/// Longest name, in characters.
const NAME_MAX: usize = 1024;
/// Longest description, in characters.
const DESCRIPTION_MAX: usize = 8192;
/// Longest location type, in characters.
const TYPE_MAX: usize = 128;

/// A named place: a meeting room, a shop, a trail head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    name: String,
    description: Option<String>,
    types: BTreeSet<String>,
    coordinates: Option<GeoPoint>,
    address: Option<Address>,
    links: Vec<Link>,
}

/// The parts of a [`Location`] in the form JSCalendar names them, as plain texts and values (the mapping to JSON is
/// `fs-jscalendar`'s). `coordinates` is a `geo:` URI (RFC 5870), each link an absolute URI (`href`), each location
/// type a value of the IANA registry or a vendor value (`domain:value`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationParts {
    /// `name`.
    pub name: String,
    /// `description`.
    pub description: Option<String>,
    /// `locationTypes` (the keys that are `true`), sorted.
    pub location_types: Vec<String>,
    /// `coordinates`, a `geo:` URI.
    pub coordinates: Option<String>,
    /// Our addition: the postal address.
    pub address: Option<Address>,
    /// `links`, the `href` of each.
    pub links: Vec<String>,
}

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
    /// A location with a name and nothing else.
    ///
    /// # Errors
    /// `Empty` (no name), `Character`, `TooLong` as for a one-line text.
    pub fn new(name: &str) -> Result<Self, TypeProblem> {
        Ok(Self {
            name: one_line(name, NAME_MAX)?,
            description: None,
            types: BTreeSet::new(),
            coordinates: None,
            address: None,
            links: Vec::new(),
        })
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

    /// The name, never empty.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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
        LocationParts {
            name: self.name.clone(),
            description: self.description.clone(),
            location_types: self.types.iter().cloned().collect(),
            coordinates: self.coordinates.map(|c| c.to_geo_uri()),
            address: self.address.clone(),
            links: self.links.iter().map(BasicValue::canonical).collect(),
        }
    }

    /// Read the JSCalendar form: every part is checked again, so a `Location` is valid whatever the JSON said.
    ///
    /// # Errors
    /// The first part that breaks its rule: `Empty` (name), `Character`, `TooLong`, a `geo:` problem
    /// (`BadPlace`, `PlaceRange`, `OutOfRange`), a link problem (`BadUri`, `NotAbsolute`, `UnsafeScheme`).
    pub fn from_parts(parts: &LocationParts) -> Result<Self, TypeProblem> {
        let types: Vec<&str> = parts.location_types.iter().map(String::as_str).collect();
        let links = parts
            .links
            .iter()
            .map(|l| Link::parse(l))
            .collect::<Result<Vec<_>, _>>()?;
        let coordinates = parts
            .coordinates
            .as_deref()
            .map(GeoPoint::from_geo_uri)
            .transpose()?;
        Self::new(&parts.name)?
            .with_description(parts.description.as_deref().unwrap_or(""))?
            .with_location_types(&types)
            .map(|l| {
                l.with_coordinates(coordinates)
                    .with_address(parts.address.clone())
                    .with_links(&links)
            })
    }
}
