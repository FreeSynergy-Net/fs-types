//! `PersonArt`, `PersonRef`, `Attendee`, `ContactAddress`: how a human is named in data (ADR 0085 section 3).
//!
//! Pattern: Value object. A person is always a reference to the person, never a free name; an address (mail, phone)
//! is how to reach someone and is not an identity.

use crate::{BasicValue, ObjectId, Reference, TypeProblem};

/// The kind of what a person reference points at, AS2 names. A cache: when it differs from the target, the target
/// counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PersonArt {
    /// A profile of a person.
    Profile,
    /// A person.
    Person,
    /// A group.
    Group,
    /// An organisation.
    Organization,
}

/// A reference to a person. `abzug` only for issued documents (a booking), otherwise `None`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersonRef {
    /// The IRI of the person (a `Reference`: `urn:uuid:…` locally, `https://…` for a foreign actor).
    pub id: Reference,
    /// What the target is.
    pub art: PersonArt,
    /// The frozen copy of the person's data at issue time, where a document demands one.
    pub abzug: Option<ObjectId>,
}

/// Who or what takes part in an appointment: a person, or a resource or a place that can be booked.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Attendee {
    /// A human actor.
    Actor(PersonRef),
    /// A resource (a projector, a car).
    Resource(Reference),
    /// A place (a room).
    Location(Reference),
}

/// How to reach a human: an absolute IRI with the scheme `mailto`, `tel`, `xmpp`, `sip`, `matrix` or `https`
/// (RFC 6068, 3966, 5122, 3261, Matrix URI, RFC 9110), kept as written. Not a person id (ADR 0085 section 3).
/// Search compares scheme and domain without case (the provider's rule), the value is never rewritten. Called
/// `Address` by the first topic crates; here that name is the postal address of `address.rs`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContactAddress(String);

impl ContactAddress {
    /// The scheme in lower case (`mailto`, `tel` …).
    #[must_use]
    pub fn scheme(&self) -> String {
        self.0
            .split_once(':')
            .map(|(s, _)| s.to_ascii_lowercase())
            .unwrap_or_default()
    }

    /// Read an address.
    ///
    /// # Errors
    /// The problem of [`Reference`] for a text that is no absolute IRI; `NotInList` for a scheme not named above.
    pub fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let text = Reference::parse(typed)?.canonical();
        let scheme = text
            .split_once(':')
            .map(|(s, _)| s.to_ascii_lowercase())
            .unwrap_or_default();
        if ["mailto", "tel", "xmpp", "sip", "matrix", "https"].contains(&scheme.as_str()) {
            Ok(Self(text))
        } else {
            Err(TypeProblem::NotInList)
        }
    }

    /// The address as written.
    #[must_use]
    pub fn canonical(&self) -> String {
        self.0.clone()
    }
}
