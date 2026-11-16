//! `ObjectId`, `Revision`, `MediaType`, `FileRef`: how programs name an object, its state, a media type, a file.
//!
//! Pattern: Value object (plain data compared by value; no behaviour to hide). Same shape the first topic crates
//! used as stand-ins, so their code moves to these types unchanged.

use crate::Reference;

/// The id of an object: a `UUIDv7` (ADR 0027), 16 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(pub [u8; 16]);

/// The state of a row for compare-and-set; rises with every write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(pub u64);

/// A media type (IANA), as written.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MediaType(pub String);

/// A file in `fs-files` with its checksum (ADR 0086, addendum).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileRef {
    /// Where the file is.
    pub iri: Reference,
    /// SHA-256 of its bytes.
    pub sha256: [u8; 32],
    /// What it is.
    pub media_type: MediaType,
}
