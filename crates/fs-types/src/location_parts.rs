//! `LocationParts`: the JSCalendar form of a [`Location`] as plain texts (the JSON mapping is `fs-jscalendar`'s).
//!
//! Pattern: Adapter data with a builder. The fields are private on purpose: a new JSCalendar field later is a new
//! `with_*`/getter pair and breaks no consumer (the project puts `non_exhaustive` on enums only, not on structs).
//! Mapping to RFC 8984 section 4.2.5: `name` (optional), `description`, `locationTypes`, `coordinates` (`geo:` URI),
//! `relativeTo` (`start`, `end` or any other registry word, kept), `timeZone` (IANA name or custom `/id`), `links` (`href` only), plus our `address`.

use crate::{Address, BasicValue, Link, Location, RelativeTo, TypeProblem, Zone};

/// The parts of a [`Location`] in the form JSCalendar names them. All parts start empty (`Default`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocationParts {
    name: Option<String>,
    description: Option<String>,
    location_types: Vec<String>,
    coordinates: Option<String>,
    relative_to: Option<String>,
    time_zone: Option<String>,
    address: Option<Address>,
    links: Vec<String>,
}

impl LocationParts {
    /// Empty parts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// With `name`.
    #[must_use]
    pub fn with_name(self, name: &str) -> Self {
        Self {
            name: Some(name.to_owned()),
            ..self
        }
    }

    /// With `description`.
    #[must_use]
    pub fn with_description(self, text: &str) -> Self {
        Self {
            description: Some(text.to_owned()),
            ..self
        }
    }

    /// With `locationTypes` (the keys that are `true`).
    #[must_use]
    pub fn with_location_types(self, types: &[&str]) -> Self {
        Self {
            location_types: types.iter().map(|t| (*t).to_owned()).collect(),
            ..self
        }
    }

    /// With `coordinates`, a `geo:` URI (RFC 5870).
    #[must_use]
    pub fn with_coordinates(self, uri: &str) -> Self {
        Self {
            coordinates: Some(uri.to_owned()),
            ..self
        }
    }

    /// With `relativeTo` (`start`, `end` or any other word of the registry: it is kept as it is).
    #[must_use]
    pub fn with_relative_to(self, word: &str) -> Self {
        Self {
            relative_to: Some(word.to_owned()),
            ..self
        }
    }

    /// With `timeZone`.
    #[must_use]
    pub fn with_time_zone(self, zone: &str) -> Self {
        Self {
            time_zone: Some(zone.to_owned()),
            ..self
        }
    }

    /// With our postal address.
    #[must_use]
    pub fn with_address(self, address: Address) -> Self {
        Self {
            address: Some(address),
            ..self
        }
    }

    /// With `links`, the absolute `href` of each.
    #[must_use]
    pub fn with_links(self, hrefs: &[&str]) -> Self {
        Self {
            links: hrefs.iter().map(|h| (*h).to_owned()).collect(),
            ..self
        }
    }

    /// `name`.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// `description`.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// `locationTypes`, sorted when they come from a [`Location`].
    #[must_use]
    pub fn location_types(&self) -> &[String] {
        &self.location_types
    }

    /// `coordinates`, a `geo:` URI.
    #[must_use]
    pub fn coordinates(&self) -> Option<&str> {
        self.coordinates.as_deref()
    }

    /// `relativeTo`.
    #[must_use]
    pub fn relative_to(&self) -> Option<&str> {
        self.relative_to.as_deref()
    }

    /// `timeZone`.
    #[must_use]
    pub fn time_zone(&self) -> Option<&str> {
        self.time_zone.as_deref()
    }

    /// The postal address (ours).
    #[must_use]
    pub fn address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    /// `links`, the `href` of each.
    #[must_use]
    pub fn links(&self) -> &[String] {
        &self.links
    }

    pub(crate) fn from_location(l: &Location) -> Self {
        Self {
            name: l.name().map(str::to_owned),
            description: l.description().map(str::to_owned),
            location_types: l.location_types().map(str::to_owned).collect(),
            coordinates: l.coordinates().map(crate::GeoPoint::to_geo_uri),
            relative_to: l.relative_to().map(|r| r.as_str().to_owned()),
            time_zone: l.time_zone().map(|z| z.id().to_owned()),
            address: l.address().cloned(),
            links: l.links().iter().map(BasicValue::canonical).collect(),
        }
    }

    pub(crate) fn to_location(&self) -> Result<Location, TypeProblem> {
        let types: Vec<&str> = self.location_types.iter().map(String::as_str).collect();
        let links = self
            .links
            .iter()
            .map(|l| Link::parse(l))
            .collect::<Result<Vec<_>, _>>()?;
        let coordinates = self
            .coordinates
            .as_deref()
            .map(crate::GeoPoint::from_geo_uri)
            .transpose()?;
        let relative_to = self.relative_to.as_deref().map(RelativeTo::from_text);
        let time_zone = self
            .time_zone
            .as_deref()
            .map(|z| {
                if z.starts_with('/') {
                    Zone::custom(z)
                } else {
                    Zone::iana(z)
                }
            })
            .transpose()?;
        Ok(Location::new()
            .with_name(self.name.as_deref().unwrap_or(""))?
            .with_description(self.description.as_deref().unwrap_or(""))?
            .with_location_types(&types)?
            .with_coordinates(coordinates)
            .with_relative_to(relative_to)?
            .with_time_zone(time_zone)
            .with_address(self.address.clone())
            .with_links(&links))
    }
}
