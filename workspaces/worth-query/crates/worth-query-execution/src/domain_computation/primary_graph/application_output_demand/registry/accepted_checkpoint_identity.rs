#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryAcceptedOutputCheckpointPosture {
    Performed,
    StableReused,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryAcceptedOutputCheckpointIdentity {
    pub(in crate::domain_computation::primary_graph) producer: String,
    pub(in crate::domain_computation::primary_graph) posture:
        WorthQueryAcceptedOutputCheckpointPosture,
    pub(in crate::domain_computation::primary_graph) source: [u8; 32],
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) source_partition: [u8; 32],
    pub(in crate::domain_computation::primary_graph) producer_dependency: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) idempotency_key: [u8; 32],
    pub(in crate::domain_computation::primary_graph) resources:
        Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources>,
    pub(in crate::domain_computation::primary_graph) roles:
        Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole>,
    /// Authenticated encoding of the complete rebased producer fact set.
    /// Its source wire version travels with recovered bytes across later captures.
    pub(in crate::domain_computation::primary_graph) producer_facts: Option<Vec<u8>>,
    pub(in crate::domain_computation::primary_graph) producer_fact_wire_version: u16,
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    pub(in crate::domain_computation::primary_graph) fn canonical_cmp(
        &self,
        other: &Self,
    ) -> std::cmp::Ordering {
        self.producer
            .cmp(&other.producer)
            .then_with(|| self.posture.cmp(&other.posture))
            .then_with(|| self.source.cmp(&other.source))
            .then_with(|| self.scope.cmp(&other.scope))
            .then_with(|| self.source_partition.cmp(&other.source_partition))
            .then_with(|| self.producer_dependency.cmp(&other.producer_dependency))
            .then_with(|| self.idempotency_key.cmp(&other.idempotency_key))
            .then_with(|| self.resources.cmp(&other.resources))
            .then_with(|| self.roles.cmp(&other.roles))
    }

    pub(in crate::domain_computation::primary_graph) fn same_output_slot(
        &self,
        other: &Self,
    ) -> bool {
        self.producer == other.producer
            && self.scope == other.scope
            && self.source_partition == other.source_partition
    }
}
