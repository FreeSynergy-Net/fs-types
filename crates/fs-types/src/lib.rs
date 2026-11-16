//! FreeSynergy basic types and type descriptions (ADR 0066, schema `types.capnp` in `fs-interfaces`).
//!
//! * Basic types: [`BasicValue`] with one value object per kind ([`Text`], [`EmailAddress`], [`Name`], [`Address`],
//!   [`Place`], [`Date`], [`Language`], [`Link`], [`Image`], [`Key`], [`Choice`], [`Keywords`], [`Reference`] …): they
//!   check, normalise and suggest. [`BasicRules`] reaches them by [`BasicType`] alone.
//! * Type description: [`TypeDescription`] (fields, commands), the trait [`DescribesType`] a program implements for each
//!   data type it owns, and [`TypeRegistry`] that collects them and answers the `TypeCatalog` contract.
//!
//! * Shared types of the topic crates (ADR 0085): [`Utc`], [`LocalDateTime`], [`Zone`], [`PersonRef`], [`PersonArt`], [`Attendee`],
//!   [`ContactAddress`], [`ObjectId`], [`Revision`], [`MediaType`], [`FileRef`], [`Page`], [`PageCursor`],
//!   [`Capabilities`], [`Problem`] (DESIGN.md, "Shared types").
//! * Positions (v0.0.24): [`GeoPoint`] (the coordinate), [`PositionFix`] with [`FixSource`] and [`AccuracyLevel`] (origin, time, blur).
//! * Named place (v0.0.25): [`Location`] (JSCalendar `Location`: name, description, types, coordinates, address, links) with [`LocationParts`].
//!
//! Pattern: Registry (rules by kind, descriptions by name), Value object, Adapter (to the free crates and to the wire
//! form). Where it lives and why: `DESIGN.md`.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]

mod address;
mod choice;
mod contact;
mod datetime;
pub mod describe;
mod fix;
mod geo;
mod ids;
mod image;
mod iri;
mod key;
mod keywords;
mod language;
mod location;
mod moment;
mod name;
mod paging;
mod person;
mod place;
mod problem;
mod rule;
mod text;
mod value;

pub use address::Address;
pub use choice::Choice;
pub use datetime::{Date, DateTime};
pub use describe::{
    CommandDescription, CommandKind, DescribesType, DescriptionProblem, Edit, FieldDescription,
    FieldEdit, RecordIssue, RecordProblem, TypeDescription, TypeRegistry,
};
pub use fix::{AccuracyLevel, FixSource, PositionFix};
pub use fs_interfaces::types_capnp::{AddressKind, BasicType, NameKind};
pub use fs_interfaces::values::{EmailAddress, PhoneNumber};
pub use geo::GeoPoint;
pub use ids::{FileRef, MediaType, ObjectId, Revision};
pub use image::Image;
pub use iri::{Link, Reference};
pub use key::Key;
pub use keywords::Keywords;
pub use language::Language;
pub use location::{Location, LocationParts};
pub use moment::{LocalDateTime, Utc, Zone, ZoneKind};
pub use name::Name;
pub use paging::{Capabilities, Page, PageCursor, Problem};
pub use person::{Attendee, ContactAddress, PersonArt, PersonRef};
pub use place::Place;
pub use problem::TypeProblem;
pub use rule::{BasicRules, Rule};
pub use text::{MultilineText, Text};
pub use value::BasicValue;
