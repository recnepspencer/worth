//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandSettlement {
    #[doc(hidden)]
    pub fn observation_for_test(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.observation
    }
}

impl WorthQueryOutputDemandSettlement {
    #[doc(hidden)]
    pub fn output_family_identity_for_test(&self) -> &String {
        &self.output_family_identity
    }
}

impl WorthQueryOutputDemandSettlement {
    #[doc(hidden)]
    pub fn posture_for_test(&self) -> &WorthQueryOutputSettlementPosture {
        &self.posture
    }
}

impl WorthQueryOutputDemandSettlement {
    #[doc(hidden)]
    pub fn producer_contacts_in_this_demand_for_test(&self) -> &usize {
        &self.producer_contacts_in_this_demand
    }
}

impl WorthQueryOutputDemandSettlement {
    #[doc(hidden)]
    pub fn producer_identity_for_test(&self) -> &SettlementProducerIdentity {
        &self.producer_identity
    }
}

impl WorthQueryOutputDemandSettlement {
    #[doc(hidden)]
    pub fn readiness_delivery_for_test(&self) -> &Option<WorthQueryOutputReadinessDeliveryEvidence> {
        &self.readiness_delivery
    }
}

impl WorthQueryOutputDemandSettlement {
    #[doc(hidden)]
    pub fn runtime_authority_for_test(&self) -> &u64 {
        &self.runtime_authority
    }
}

impl WorthQueryOutputDemandSettlement {
    #[doc(hidden)]
    pub fn schema_binding_for_test(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }
}
