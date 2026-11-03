//! Sealed test doors over this module's private members.
use super::*;

impl OriginalSettledProducerObservation {
    #[doc(hidden)]
    pub fn correspondence_for_test(&self) -> &Arc<WorthQueryApplicationOutputCorrespondence> {
        &self.correspondence
    }
}

impl OriginalSettledProducerObservation {
    #[doc(hidden)]
    pub fn read_for_test(&self) -> &WorthQueryProductBranchReadIdentity {
        &self.read
    }
}

impl OriginalSettledProducerObservation {
    #[doc(hidden)]
    pub fn runtime_authority_for_test(&self) -> &u64 {
        &self.runtime_authority
    }
}

impl OriginalSettledProducerObservation {
    #[doc(hidden)]
    pub fn schema_for_test(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
}

impl OriginalSettledProducerObservation {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &OriginalSettledProducerSource {
        &self.source
    }
}
