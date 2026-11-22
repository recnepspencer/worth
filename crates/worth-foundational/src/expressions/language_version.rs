//! Language revisions.
//!
//! V2 adds the `Date` type and ASCII case-insensitive text matching. Nothing
//! V1 means changed, so a draft or program that uses only V1 vocabulary keeps
//! its V1 draft encoding and its V1 canonical identity byte for byte; only
//! meaning that uses V2 vocabulary records V2.

/// The language revision a draft or program requires: the oldest one whose
/// vocabulary covers everything it uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LanguageVersion {
    V1,
    V2,
}

impl LanguageVersion {
    /// The language field of the draft encoding header.
    pub(crate) const fn wire_code(self) -> u16 {
        match self {
            Self::V1 => 1,
            Self::V2 => 2,
        }
    }

    pub(crate) const fn from_wire_code(code: u16) -> Option<Self> {
        match code {
            1 => Some(Self::V1),
            2 => Some(Self::V2),
            _ => None,
        }
    }

    /// The semantic version recorded in canonical program identity.
    pub(crate) const fn identity_label(self) -> &'static str {
        match self {
            Self::V1 => "worth-expression-v1",
            Self::V2 => "worth-expression-v2",
        }
    }

    /// The revision that introduced an authored type name.
    pub(crate) fn of_type_name(name: &str) -> Self {
        if name == "Date" {
            Self::V2
        } else {
            Self::V1
        }
    }
}
