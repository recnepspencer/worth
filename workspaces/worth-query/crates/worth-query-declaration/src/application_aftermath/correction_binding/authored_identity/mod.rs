//! Family-declared logical identity for authored lifecycle inverses.
mod authoring;
mod key;
pub use authoring::ApplicationAuthoredCorrection;
pub use key::ApplicationAuthoredLogicalKey;

use crate::application_operation::ApplicationMutationOutputPosture;
use crate::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationSchema, ApplicationSchemaMember,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

/// One authored entity family declares the key preserved when deletion is corrected.
pub trait ApplicationAuthoredCorrectionFamily<Schema: ApplicationSchema>: 'static {
    const IDENTITY: &'static str;
    type Entity: ApplicationEntityMarkerIdentity<Schema>;
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Self::Entity>;
}

/// Descriptive family and role meaning; installation resolves every part.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredAuthoredCorrection {
    parts: DeclaredAuthoredCorrectionParts,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredAuthoredCorrectionParts {
    pub family: WorthQueryPortableTypeIdentity,
    pub logical_key: Box<ApplicationSchemaMember>,
    pub binding_unit: Option<String>,
    pub source_role: String,
    pub inverse_role: String,
    pub source_posture: ApplicationMutationOutputPosture,
}

impl DeclaredAuthoredCorrection {
    pub fn from_untrusted_parts(parts: DeclaredAuthoredCorrectionParts) -> Self {
        Self { parts }
    }
    pub fn parts(&self) -> &DeclaredAuthoredCorrectionParts {
        &self.parts
    }
    pub fn same_family_as(&self, other: &Self) -> bool {
        self.parts.family == other.parts.family
            && self.parts.logical_key == other.parts.logical_key
            && self.parts.binding_unit == other.parts.binding_unit
    }
}
