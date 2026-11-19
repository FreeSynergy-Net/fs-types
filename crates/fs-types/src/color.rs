//! `Color`: an sRGB colour with alpha, CSS Color Level 4 hex notation.
//!
//! Pattern: Value object, Template method (`BasicValue`). The four hex forms of CSS Color Level 4 are read:
//! `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, in any case, with the `#`. Named colours, `rgb()` and the other
//! functions are not read (one notation, one standard). The canonical text is lower case, always the long form
//! (`#rrggbb`), with `aa` only when the colour is not fully opaque, so equal colours have equal text.

use crate::types_capnp::BasicType;

use crate::{BasicValue, TypeProblem};

/// An sRGB colour, 8 bit per channel, alpha 255 = opaque.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {
    /// A colour from its four channels.
    #[must_use]
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Red channel.
    #[must_use]
    pub fn red(&self) -> u8 {
        self.red
    }

    /// Green channel.
    #[must_use]
    pub fn green(&self) -> u8 {
        self.green
    }

    /// Blue channel.
    #[must_use]
    pub fn blue(&self) -> u8 {
        self.blue
    }

    /// Alpha channel, 255 is opaque.
    #[must_use]
    pub fn alpha(&self) -> u8 {
        self.alpha
    }
}

fn nibble(c: u8) -> Option<u8> {
    char::from(c)
        .to_digit(16)
        .and_then(|d| u8::try_from(d).ok())
}

impl BasicValue for Color {
    const KIND: BasicType = BasicType::Color;

    fn parse(typed: &str) -> Result<Self, TypeProblem> {
        let t = typed.trim();
        if t.is_empty() {
            return Err(TypeProblem::Empty);
        }
        let digits = t.strip_prefix('#').ok_or(TypeProblem::BadColor)?;
        let nibbles: Option<Vec<u8>> = digits.bytes().map(nibble).collect();
        let n = nibbles.ok_or(TypeProblem::BadColor)?;
        let channels: Vec<u8> = match n.len() {
            // One digit stands for the same digit twice (`#f` + `f` = 0xff).
            3 | 4 => n.iter().map(|d| d * 17).collect(),
            6 | 8 => n.chunks(2).map(|p| p[0] * 16 + p[1]).collect(),
            _ => return Err(TypeProblem::BadColor),
        };
        let alpha = channels.get(3).copied().unwrap_or(255);
        Ok(Self::new(channels[0], channels[1], channels[2], alpha))
    }

    fn canonical(&self) -> String {
        if self.alpha == 255 {
            format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
        } else {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                self.red, self.green, self.blue, self.alpha
            )
        }
    }
}
