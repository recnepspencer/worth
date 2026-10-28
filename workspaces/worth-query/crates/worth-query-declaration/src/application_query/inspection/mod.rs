//! Optional inspection metadata meaning for an exact ordinary query binding.
//! Declarations carry no observation, read permission, or retention authority.
mod definition;
mod metadata;
pub(in crate::application_query) mod validation;

pub use definition::{
    ApplicationQueryInspectionDefinition, ApplicationQueryInspectionDefinitionDenial,
    ErasedApplicationQueryInspectionDefinition, WorthQueryPortableApplicationQueryInspectionParts,
};
pub use metadata::{
    ApplicationQueryInspectionMetadataFacet, ApplicationQueryInspectionMetadataMeaning,
    ApplicationQueryInspectionMetadataRef,
};
