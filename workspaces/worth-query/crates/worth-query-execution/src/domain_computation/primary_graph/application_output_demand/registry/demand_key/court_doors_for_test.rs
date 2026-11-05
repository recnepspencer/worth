//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_family_identity::OutputFamilyIdentity;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerApplicability;

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn applicability_for_test(&self) -> &WorthQueryProducerApplicability {
        &self.applicability
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn producer_for_test(&self) -> &String {
        &self.producer
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &SourceEpoch {
        &self.source
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn applicability_mut_for_test(&mut self) -> &mut WorthQueryProducerApplicability {
        &mut self.applicability
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn new_for_test(
        family: &'static str,
        producer: String,
        applicability: WorthQueryProducerApplicability,
        source: SourceEpoch,
    ) -> Self {
        Self::new(family, producer, applicability, source)
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn source_into_for_test(self) -> SourceEpoch {
        self.source
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn family_mut_for_test(&mut self) -> &mut OutputFamilyIdentity {
        &mut self.family
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn producer_mut_for_test(&mut self) -> &mut String {
        &mut self.producer
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn producer_identity_for_test(&self) -> &str {
        Self::producer_identity(self)
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn replacement_order_for_test(&self, other: &Self) -> Option<Ordering> {
        Self::replacement_order(self, other)
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn same_occurrence_for_test(&self, other: &Self) -> bool {
        Self::same_occurrence(self, other)
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn same_semantic_source_for_test(&self, other: &Self) -> bool {
        Self::same_semantic_source(self, other)
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn source_epoch_for_test(&self) -> &SourceEpoch {
        Self::source_epoch(self)
    }
}
