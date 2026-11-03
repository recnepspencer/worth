//! Sealed test doors over this module's private members.
use super::*;

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
    pub fn new_for_test( family: &'static str, producer: String, applicability: WorthQueryProducerApplicability, source: SourceEpoch) -> Self {
        Self::new(family, producer, applicability, source)
    }
}

impl WorthQueryOutputDemandKey {
    #[doc(hidden)]
    pub fn source_into_for_test(self) -> SourceEpoch {
        self.source
    }
}
