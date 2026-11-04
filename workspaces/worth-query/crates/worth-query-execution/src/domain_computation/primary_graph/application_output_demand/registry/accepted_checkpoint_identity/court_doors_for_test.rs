//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn idempotency_key_for_test(&self) -> &[u8; 32] {
        &self.idempotency_key
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn idempotency_key_mut_for_test(&mut self) -> &mut [u8; 32] {
        &mut self.idempotency_key
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn scope_for_test(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        &self.scope
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn source_partition_for_test(&self) -> &[u8; 32] {
        &self.source_partition
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn producer_for_test(&self) -> &std::sync::Arc<String> {
        &self.producer
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn producer_facts_for_test(&self) -> &Option<std::sync::Arc<Vec<u8>>> {
        &self.producer_facts
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn roles_for_test(&self) -> &std::sync::Arc<Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole>>{
        &self.roles
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn implementation_mut_for_test(
        &mut self,
    ) -> &mut worth_relational::facade::mvcc::RequiredOutputProducingImplementation {
        &mut self.implementation
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn producer_mut_for_test(&mut self) -> &mut std::sync::Arc<String> {
        &mut self.producer
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn producer_fact_wire_version_mut_for_test(&mut self) -> &mut u16 {
        &mut self.producer_fact_wire_version
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn producer_facts_mut_for_test(&mut self) -> &mut Option<std::sync::Arc<Vec<u8>>> {
        &mut self.producer_facts
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn roles_mut_for_test(&mut self) -> &mut std::sync::Arc<Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole>>{
        &mut self.roles
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn source_mut_for_test(&mut self) -> &mut [u8; 32] {
        &mut self.source
    }
}
