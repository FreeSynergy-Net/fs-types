//! FreeSynergy basic types and type descriptions (ADR 0066, schema `schema/types.capnp` in this crate).
//!
//! * Basic types: [`BasicValue`] with one value object per kind ([`Text`], [`EmailAddress`], [`Name`], [`Address`],
//!   [`Place`], [`Date`], [`Language`], [`Link`], [`Image`], [`Key`], [`Choice`], [`Keywords`], [`Reference`] …): they
//!   check, normalise and suggest. [`BasicRules`] reaches them by [`BasicType`] alone.
//! * Type description: [`TypeDescription`] (fields, commands), the trait [`DescribesType`] a program implements for each
//!   data type it owns, and [`TypeRegistry`] that collects them and answers the `TypeCatalog` contract.
//!
//! * Since 0.0.5: [`Time`] (time of day, RFC 3339 `partial-time`) and [`Color`] (CSS Color Level 4 hex, sRGB with alpha).
//!
//! * Shared types of the topic crates (ADR 0085): [`Utc`], [`LocalDateTime`], [`Zone`], [`PersonRef`], [`PersonArt`], [`Attendee`],
//!   [`ContactAddress`], [`ObjectId`], [`Revision`], [`MediaType`], [`FileRef`], [`Page`], [`PageCursor`],
//!   [`Capabilities`], [`Problem`] (DESIGN.md, "Shared types").
//! * Positions (v0.0.24): [`GeoPoint`] (the coordinate), [`PositionFix`] with [`FixSource`] and [`AccuracyLevel`] (origin, time, blur).
//! * Named place (v0.0.25): [`Location`] (JSCalendar `Location`: optional name, description, types, coordinates, relativeTo, timeZone, address, links) with [`LocationParts`].
//!
//! * Contact values: [`values`] ([`EmailAddress`], [`PhoneNumber`], [`ValueProblem`], trait [`ContactValue`]), valid on construction.
//! * Failures as codes (first change after `v0.0.1`): [`coded`] ([`coded::ErrorCode`], [`coded::Coded`], [`coded::Code`], [`coded::Param`]); a code with typed parameters, never a sentence (ADR 0057).
//! * Wire form: [`types_capnp`] (generated, committed; ADR 0040); [`SCHEMA_DIR`] is where the schema file lives, for crates that import it.
//!
//! Pattern: Registry (rules by kind, descriptions by name), Value object, Adapter (to the free crates and to the wire
//! form). Where it lives and why: `DESIGN.md`.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]

/// The directory that holds `types.capnp` in this crate's source tree (a dependent crate that reads this crate from a
/// git checkout gets the path of that checkout). Use it as the import path of a schema compiler, with
/// `import "/types.capnp"` in the schema that needs the basic types. See the README, "Using the schema".
pub const SCHEMA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/schema");

#[rustfmt::skip]
#[allow(clippy::all, clippy::pedantic, unused_parens, dead_code, missing_docs)]
pub mod types_capnp;
pub mod coded;
pub mod values;

mod address;
mod choice;
mod color;
mod contact;
mod datetime;
pub mod describe;
mod duration;
mod fix;
mod geo;
mod ids;
mod image;
mod iri;
mod key;
mod keywords;
mod language;
mod location;
mod location_parts;
mod moment;
mod name;
mod number;
mod paging;
mod person;
mod place;
mod problem;
mod rule;
mod secret;
mod text;
mod time_of_day;
mod value;

pub use address::Address;
pub use choice::Choice;
pub use color::Color;
pub use datetime::{Date, DateTime};
pub use describe::{
    CommandDescription, CommandKind, DescribesType, DescriptionProblem, Edit, FieldDescription,
    FieldEdit, RecordIssue, RecordProblem, TypeDescription, TypeRegistry,
};
pub use duration::Duration;
pub use fix::{AccuracyLevel, FixSource, PositionFix};
pub use geo::GeoPoint;
pub use ids::{FileRef, MediaType, ObjectId, Revision};
pub use image::Image;
pub use iri::{Link, Reference};
pub use key::Key;
pub use keywords::Keywords;
pub use language::Language;
pub use location::{Location, RelativeTo};
pub use location_parts::LocationParts;
pub use moment::{LocalDateTime, Utc, Zone, ZoneKind};
pub use name::Name;
pub use number::{Flag, Integer, Whole};
pub use paging::{Capabilities, Page, PageCursor, Problem};
pub use person::{Attendee, ContactAddress, PersonArt, PersonRef};
pub use place::Place;
pub use problem::TypeProblem;
pub use rule::{BasicRules, Rule};
pub use secret::Secret;
pub use text::{MultilineText, Text};
pub use time_of_day::Time;
pub use types_capnp::{AddressKind, BasicType, NameKind};
pub use value::BasicValue;
pub use values::{ContactValue, EmailAddress, PhoneNumber, ValueProblem};
