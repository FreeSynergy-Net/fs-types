//! `NameKind`, `AddressKind`: the `kind` words of JSContact name and address components (RFC 9553 sections 2.2.1.2 and
//! 2.5.1.2), as enums that keep a word they do not know.
//!
//! Pattern: Value enum with an escape variant. The registry is open (RFC 9553 section 1.7.5: IANA values and vendor
//! values `domain:name`), so a reader meets words that are not in the list. The wire form is `Text` (`types.capnp`);
//! the rule is "read everything, write only what is known or ours": [`NameKind::from_text`] never refuses a word, it
//! keeps an unknown one as `Other` and [`NameKind::as_str`] gives it back unchanged. The words are case-sensitive
//! (RFC 9553 section 1.7.5), so `"Given"` is `Other("Given")`, not `Given`. The same pattern serves
//! [`crate::RelativeTo`].
//!
//! One value, one form: `Other` never holds a word that has a known variant. `from_text` guarantees it, and the value
//! objects that take a kind from the caller ([`crate::Name::new`], [`crate::Address::new`]) put it through
//! `checked`, which parses the word again, so `Other("given")` made by hand is `Given` in the value.

use crate::TypeProblem;
use crate::value::bad_char;
use strum::{EnumString, IntoStaticStr};

/// Longest registry word we accept, in characters (abuse limit, not a rule of the registry).
pub(crate) const WORD_MAX: usize = 128;

/// The word checks shared by every registry enum: not empty, no control character, not over-long.
pub(crate) fn word_ok(word: &str) -> Result<(), TypeProblem> {
    if word.is_empty() {
        return Err(TypeProblem::Empty);
    }
    if word.chars().any(bad_char) {
        return Err(TypeProblem::Character);
    }
    if word.chars().count() > WORD_MAX {
        return Err(TypeProblem::TooLong);
    }
    Ok(())
}

/// The methods every registry enum has: `as_str`, `from_text`, `Display`, `checked`. `Other` is the escape variant.
macro_rules! registry_word {
    ($name:ident) => {
        impl $name {
            /// The registry word, exactly as it is written on the wire (an unknown word as it was read).
            #[must_use]
            pub fn as_str(&self) -> &str {
                match self {
                    Self::Other(word) => word.as_str(),
                    known => <&'static str>::from(known),
                }
            }

            /// Read a word. Never refuses: an unknown word is kept as `Other`, a known one is always its variant.
            #[must_use]
            pub fn from_text(word: &str) -> Self {
                <Self as ::std::str::FromStr>::from_str(word)
                    .unwrap_or_else(|_| Self::Other(word.to_owned()))
            }

            /// The value in its one form (`Other` of a known word becomes the variant) after the word checks.
            ///
            /// # Errors
            /// `Empty`, `Character` (control character), `TooLong` (over 128 characters) for the word.
            pub fn checked(self) -> Result<Self, $crate::TypeProblem> {
                $crate::kind::word_ok(self.as_str())?;
                Ok(Self::from_text(self.as_str()))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}
pub(crate) use registry_word;

/// The kind of a name component (RFC 9553 section 2.2.1.2), or a word of the registry we do not know.
#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "camelCase")]
#[non_exhaustive]
pub enum NameKind {
    /// `title`.
    Title,
    /// `given`.
    Given,
    /// `given2`.
    Given2,
    /// `surname`.
    Surname,
    /// `surname2`.
    Surname2,
    /// `credential`.
    Credential,
    /// `generation`.
    Generation,
    /// `separator`: literal text between the other components.
    Separator,
    /// A word that is not in the list: a newer registry value or a vendor value (`example.com:baz`), kept as read.
    #[strum(default)]
    Other(String),
}
registry_word!(NameKind);

/// The kind of an address component (RFC 9553 section 2.5.1.2), or a word of the registry we do not know.
#[derive(Clone, Debug, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "camelCase")]
#[non_exhaustive]
pub enum AddressKind {
    /// `room`.
    Room,
    /// `apartment`.
    Apartment,
    /// `floor`.
    Floor,
    /// `building`.
    Building,
    /// `number`.
    Number,
    /// `name`.
    Name,
    /// `block`.
    Block,
    /// `subdistrict`.
    Subdistrict,
    /// `district`.
    District,
    /// `locality`.
    Locality,
    /// `region`.
    Region,
    /// `postcode`.
    Postcode,
    /// `country`.
    Country,
    /// `direction`.
    Direction,
    /// `landmark`.
    Landmark,
    /// `postOfficeBox`.
    PostOfficeBox,
    /// `separator`: literal text between the other components.
    Separator,
    /// A word that is not in the list: a newer registry value or a vendor value (`example.com:baz`), kept as read.
    #[strum(default)]
    Other(String),
}
registry_word!(AddressKind);
