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

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn implementation_for_test(
        &self,
    ) -> &worth_relational::facade::mvcc::RequiredOutputProducingImplementation {
        &self.implementation
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &[u8; 32] {
        &self.source
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn posture_for_test(&self) -> &WorthQueryAcceptedOutputCheckpointPosture {
        &self.posture
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn producer_fact_wire_version_for_test(&self) -> &u16 {
        &self.producer_fact_wire_version
    }
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    #[doc(hidden)]
    pub fn new_for_test(
        implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
        producer: std::sync::Arc<String>,
        posture: WorthQueryAcceptedOutputCheckpointPosture,
        source: [u8; 32],
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        source_partition: [u8; 32],
        producer_dependency: Option<[u8; 32]>,
        idempotency_key: [u8; 32],
        roles: std::sync::Arc<Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole>>,
        producer_facts: Option<std::sync::Arc<Vec<u8>>>,
        producer_fact_wire_version: u16,
        metadata: Option<std::sync::Arc<
        crate::domain_computation::primary_graph::application_checkpoint::CheckpointMetadataFunding>>,
    ) -> Self {
        Self {
            implementation,
            producer,
            posture,
            source,
            scope,
            source_partition,
            producer_dependency,
            idempotency_key,
            roles,
            producer_facts,
            producer_fact_wire_version,
            metadata,
        }
    }
}
