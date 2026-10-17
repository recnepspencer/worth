//! Portable, descriptive value meaning. These records convey no predecessor,
//! installation, recovered-source, or compiled-edition authority.
mod aspect;
mod field;
mod predecessor;

pub use aspect::ApplicationAspectValueMeaning;
pub use field::ApplicationFieldValueMeaning;
pub use predecessor::{ApplicationAspectMeaningChange, ApplicationAspectMeaningPredecessorDenial};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationValueMeaningDenial {
    Canonical(worth_foundational::facade::CanonicalDigestDerivationDenial),
    DuplicateAspect,
    DuplicateAspectIdentity,
    RevisionZero,
    FieldWithoutAspect,
    DuplicateField,
}

#[cfg(test)]
mod tests;
