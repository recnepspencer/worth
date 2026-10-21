#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryAcceptedOutputCheckpointPosture {
    Performed,
    StableReused,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryAcceptedOutputCheckpointIdentity {
    pub(in crate::domain_computation::primary_graph) implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
    pub(in crate::domain_computation::primary_graph) producer: std::sync::Arc<String>,
    pub(in crate::domain_computation::primary_graph) posture:
        WorthQueryAcceptedOutputCheckpointPosture,
    pub(in crate::domain_computation::primary_graph) source: [u8; 32],
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) source_partition: [u8; 32],
    pub(in crate::domain_computation::primary_graph) producer_dependency: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) idempotency_key: [u8; 32],
    pub(in crate::domain_computation::primary_graph) roles:
        std::sync::Arc<Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole>>,
    /// Authenticated encoding of the complete rebased producer fact set.
    /// Its source wire version travels with recovered bytes across later captures.
    pub(in crate::domain_computation::primary_graph) producer_facts: Option<std::sync::Arc<Vec<u8>>>,
    pub(in crate::domain_computation::primary_graph) producer_fact_wire_version: u16,
    // Last to drop; clones share the exact immutable heaps AND their original payer.
    pub(in crate::domain_computation::primary_graph) metadata: Option<std::sync::Arc<
        crate::domain_computation::primary_graph::application_checkpoint::CheckpointMetadataFunding>>,
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    pub(in crate::domain_computation::primary_graph) fn output_slot(
        &self,
    ) -> CheckpointOutputSlot<'_> {
        CheckpointOutputSlot::new(&self.producer, self.scope, self.source_partition)
    }

    pub(in crate::domain_computation::primary_graph) fn canonical_cmp(
        &self,
        other: &Self,
    ) -> std::cmp::Ordering {
        self.producer
            .cmp(&other.producer)
            .then_with(|| self.implementation.cmp(&other.implementation))
            .then_with(|| self.posture.cmp(&other.posture))
            .then_with(|| self.source.cmp(&other.source))
            .then_with(|| self.scope.cmp(&other.scope))
            .then_with(|| self.source_partition.cmp(&other.source_partition))
            .then_with(|| self.producer_dependency.cmp(&other.producer_dependency))
            .then_with(|| self.idempotency_key.cmp(&other.idempotency_key))
            .then_with(|| self.roles.cmp(&other.roles))
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn same_output_slot(
        &self,
        other: &Self,
    ) -> bool {
        self.output_slot() == other.output_slot()
    }
}

/// Borrowed checkpoint slot identity; it carries no publication authority.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct CheckpointOutputSlot<'a> {
    producer: &'a str,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    partition: [u8; 32],
}

impl<'a> CheckpointOutputSlot<'a> {
    pub(in crate::domain_computation::primary_graph) fn new(
        producer: &'a str,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
    ) -> Self {
        Self {
            producer,
            scope,
            partition,
        }
    }
}

// Resource custody is not semantic checkpoint identity.
impl PartialEq for WorthQueryAcceptedOutputCheckpointIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for WorthQueryAcceptedOutputCheckpointIdentity {}
impl PartialOrd for WorthQueryAcceptedOutputCheckpointIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for WorthQueryAcceptedOutputCheckpointIdentity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.canonical_cmp(other)
            .then_with(|| self.producer_facts.cmp(&other.producer_facts))
            .then_with(|| {
                self.producer_fact_wire_version
                    .cmp(&other.producer_fact_wire_version)
            })
    }
}
