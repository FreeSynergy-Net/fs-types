//! What a rule needs from a provider under test: the provider, its types, a way to seed one situation, a way to write.
//!
//! Pattern: Strategy (the program supplies the fixture over its real store; the suite supplies [`super::MemoryBench`]).

use fs_types::TypeDescription;

use crate::records::Records;

/// One situation a rule asks for. The fixture clears what it seeded before and seeds this, as the asking person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrange {
    /// Five readable records in one scope; `probe` is a value of `text` held by exactly `ids[0]`.
    Five,
    /// Three readable records with the same value in `text`; `ids` in ascending id order.
    Ties,
    /// `ids[0]` has the field `hidden` she may not read, and the value `probe` in it (Nachzug: `probe` is that value).
    Hidden,
    /// `ids[0]` is locked (claimed by someone else).
    Locked,
    /// `ids[0]` exists but she may not read it; `ids[1]` she may.
    Unreadable,
}

/// Where the rule looks after [`RecordsFixture::arrange`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    /// The data type seeded.
    pub data_type: String,
    /// The scope the records lie in.
    pub scope: Option<String>,
    /// The ids seeded, as the situation describes.
    pub ids: Vec<String>,
    /// A readable top-level text field.
    pub text: String,
    /// A value of `text` held by exactly `ids[0]` (situation `Five`).
    pub probe: String,
    /// The hidden field (situation `Hidden`).
    pub hidden: String,
    /// A grouping of the shape.
    pub grouping: String,
    /// Nachzug 17.11.: the value of `text` of each id, index-aligned with `ids` (situation `Five`: five different values).
    pub values: Vec<String>,
}

/// The provider under test.
pub trait RecordsFixture {
    /// The provider, as the asking person.
    fn records(&self) -> &dyn Records;

    /// Its type catalog (what `TypeCatalog.describe` answers).
    fn types(&self) -> Vec<TypeDescription>;

    /// Seeds a situation; `None` = this fixture cannot (the need `fixture.arrange`).
    fn arrange(&mut self, situation: Arrange) -> Option<Facts>;

    /// Another writer changes one seeded record; `false` = this fixture cannot.
    fn touch(&mut self) -> bool;
}
