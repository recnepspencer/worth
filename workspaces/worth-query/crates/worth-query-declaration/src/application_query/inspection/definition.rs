use std::marker::PhantomData;

use super::super::{
    ApplicationQueryBinding, ApplicationQueryBindingReference, ApplicationQueryCanonicalArtifact,
    ApplicationQueryDisclosureContract,
};
use crate::application_schema::ApplicationSchema;

/// Portable meaning only; the binding includes input, result, scope and principal.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryPortableApplicationQueryInspectionParts {
    pub semantic_version: u16,
    pub binding: ApplicationQueryBindingReference,
    pub metadata_disclosure: ApplicationQueryDisclosureContract,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ErasedApplicationQueryInspectionDefinition {
    parts: WorthQueryPortableApplicationQueryInspectionParts,
    canonical: ApplicationQueryCanonicalArtifact,
}

impl ErasedApplicationQueryInspectionDefinition {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryInspectionParts) -> Self {
        let canonical = super::super::canonical_basis::prepare_inspection_basis(&parts);
        Self { parts, canonical }
    }

    pub fn parts(&self) -> &WorthQueryPortableApplicationQueryInspectionParts {
        &self.parts
    }

    pub fn canonical_basis(&self) -> &ApplicationQueryCanonicalArtifact {
        &self.canonical
    }

    pub fn validate(&self) -> Result<(), ApplicationQueryInspectionDefinitionDenial> {
        super::validation::validate(&self.parts)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationQueryInspectionDefinitionDenial {
    UnsupportedSemanticVersion,
    InvalidBindingReference,
    InvalidMetadataDisclosure,
}

pub struct ApplicationQueryInspectionDefinition<Schema, Binding> {
    erased: ErasedApplicationQueryInspectionDefinition,
    _binding: PhantomData<fn() -> (Schema, Binding)>,
}

impl<Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>
    ApplicationQueryInspectionDefinition<Schema, Binding>
{
    /// The explicit metadata contract is independent of payload disclosure.
    pub fn new(
        metadata_disclosure: ApplicationQueryDisclosureContract,
    ) -> Result<Self, ApplicationQueryInspectionDefinitionDenial> {
        let erased = ErasedApplicationQueryInspectionDefinition::from_untrusted_parts(
            WorthQueryPortableApplicationQueryInspectionParts {
                semantic_version: 1,
                binding: ApplicationQueryBindingReference::of::<Schema, Binding>(),
                metadata_disclosure,
            },
        );
        erased.validate()?;
        Ok(Self {
            erased,
            _binding: PhantomData,
        })
    }

    pub fn into_erased(self) -> ErasedApplicationQueryInspectionDefinition {
        self.erased
    }
}
