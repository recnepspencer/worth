use std::marker::PhantomData;

use super::super::{
    ApplicationQueryBinding, ApplicationQueryBindingReference, ApplicationQueryObservableInfluence,
    ApplicationQueryResultSlotKey,
};
use crate::application_schema::ApplicationSchema;

/// Metadata admission never grants access to the diagnostic fields themselves.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryInspectionMetadataFacet {
    ObservationIdentity,
    EvidenceLinks,
    EvidenceCounts,
    EvidenceAvailability,
    Explanation,
    DiagnosticFields,
}

impl ApplicationQueryInspectionMetadataFacet {
    pub const ALL: [Self; 6] = [
        Self::ObservationIdentity,
        Self::EvidenceLinks,
        Self::EvidenceCounts,
        Self::EvidenceAvailability,
        Self::Explanation,
        Self::DiagnosticFields,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::ObservationIdentity => "inspection-observation-identity",
            Self::EvidenceLinks => "inspection-evidence-links",
            Self::EvidenceCounts => "inspection-evidence-counts",
            Self::EvidenceAvailability => "inspection-evidence-availability",
            Self::Explanation => "inspection-explanation",
            Self::DiagnosticFields => "inspection-diagnostic-fields",
        }
    }

    pub const fn influence(self) -> ApplicationQueryObservableInfluence {
        match self {
            Self::EvidenceCounts => ApplicationQueryObservableInfluence::Count,
            _ => ApplicationQueryObservableInfluence::Explanation,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryInspectionMetadataMeaning {
    binding: ApplicationQueryBindingReference,
    facet: ApplicationQueryInspectionMetadataFacet,
}

impl ApplicationQueryInspectionMetadataMeaning {
    pub fn from_untrusted_fields(
        binding: ApplicationQueryBindingReference,
        facet: ApplicationQueryInspectionMetadataFacet,
    ) -> Self {
        Self { binding, facet }
    }

    pub fn binding(&self) -> &ApplicationQueryBindingReference {
        &self.binding
    }
    pub const fn facet(&self) -> ApplicationQueryInspectionMetadataFacet {
        self.facet
    }
    pub const fn slot_type(&self) -> &'static str {
        self.facet.name()
    }
    pub const fn query_type(&self) -> &str {
        self.binding.parts().query.as_str()
    }

    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey {
        ApplicationQueryResultSlotKey::inspection_metadata(self.binding.clone(), self.facet)
    }
}

pub struct ApplicationQueryInspectionMetadataRef<Schema, Binding> {
    facet: ApplicationQueryInspectionMetadataFacet,
    _binding: PhantomData<fn() -> (Schema, Binding)>,
}

impl<Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>
    ApplicationQueryInspectionMetadataRef<Schema, Binding>
{
    pub const fn new(facet: ApplicationQueryInspectionMetadataFacet) -> Self {
        Self {
            facet,
            _binding: PhantomData,
        }
    }

    pub fn meaning(&self) -> ApplicationQueryInspectionMetadataMeaning {
        ApplicationQueryInspectionMetadataMeaning::from_untrusted_fields(
            ApplicationQueryBindingReference::of::<Schema, Binding>(),
            self.facet,
        )
    }
}
