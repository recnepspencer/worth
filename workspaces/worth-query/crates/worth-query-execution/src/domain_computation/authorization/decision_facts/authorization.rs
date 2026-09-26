use std::sync::Arc;

use worth_relational::facade::authorization::{
    RelationalAuthorizationObservationCounters, RelationalAuthorizationObservationEvidence,
    RelationalAuthorizationObservationFreshness,
};
use worth_runtime_bridge::facade::{
    BridgeAuthorizationDecisionEvidence, BridgeAuthorizationRuntime,
};

mod delegation;
mod durable_dependencies;
pub(in crate::domain_computation::authorization) use delegation::WorthQueryDelegationDecisionFact;
pub(in crate::domain_computation) use delegation::WorthQueryDurableCapabilityLineage;
pub(in crate::domain_computation) use durable_dependencies::WorthQueryDurableAuthorizationDependencies;
mod delegation_activation;
pub(in crate::domain_computation::authorization) use delegation_activation::WorthQueryDelegationActivationDecisionFact;

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryAuthorizationDecisionFact {
    session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: Arc<RelationalAuthorizationObservationEvidence>,
    bridge: Arc<BridgeAuthorizationDecisionEvidence>,
    native_dependencies: Option<String>,
    delegation: Option<Arc<WorthQueryDelegationDecisionFact>>,
    delegation_activation: Option<Arc<WorthQueryDelegationActivationDecisionFact>>,
    preparatory_relational_work: RelationalAuthorizationObservationCounters,
}

impl WorthQueryAuthorizationDecisionFact {
    pub(in crate::domain_computation) fn durable_lineage(
        &self,
    ) -> WorthQueryDurableCapabilityLineage {
        self.delegation
            .as_ref()
            .map_or(WorthQueryDurableCapabilityLineage::Unbound, |delegation| {
                delegation.durable_lineage()
            })
    }
    pub(in crate::domain_computation::authorization) fn from_capability_observation(
        _permit: crate::domain_computation::authorization::capability_observation::WorthQueryAuthorizationDecisionPermit,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        relational: RelationalAuthorizationObservationEvidence,
        bridge: BridgeAuthorizationDecisionEvidence,
    ) -> Result<Self, ()> {
        let dependencies = durable_dependencies::capture(runtime, &relational)?;
        let mut decision = Self::mint(session_identity, relational, bridge);
        decision.native_dependencies = Some(dependencies);
        Ok(decision)
    }

    pub(in crate::domain_computation::authorization) fn from_conventional_observation(
        _permit: crate::domain_computation::authorization::admission::WorthQueryConventionalAuthorizationDecisionPermit,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        relational: RelationalAuthorizationObservationEvidence,
        bridge: BridgeAuthorizationDecisionEvidence,
    ) -> Self {
        Self::mint(session_identity, relational, bridge)
    }

    fn mint(
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        relational: RelationalAuthorizationObservationEvidence,
        bridge: BridgeAuthorizationDecisionEvidence,
    ) -> Self {
        Self {
            session_identity,
            relational: Arc::new(relational),
            bridge: Arc::new(bridge),
            native_dependencies: None,
            delegation: None,
            delegation_activation: None,
            preparatory_relational_work: RelationalAuthorizationObservationCounters::default(),
        }
    }

    pub(in crate::domain_computation::authorization) fn with_preparatory_relational_work(
        mut self,
        work: RelationalAuthorizationObservationCounters,
    ) -> Self {
        add_counters(&mut self.preparatory_relational_work, work);
        self
    }

    pub(in crate::domain_computation::authorization) fn with_delegation(
        mut self,
        delegation: WorthQueryDelegationDecisionFact,
    ) -> Self {
        debug_assert!(delegation.belongs_to_session(self.session_identity));
        self.delegation = Some(Arc::new(delegation));
        self
    }

    pub(in crate::domain_computation::authorization) fn attach_delegation_activation(
        &mut self,
        activation: WorthQueryDelegationActivationDecisionFact,
    ) -> Result<(), ()> {
        if self.delegation_activation.is_some()
            || !activation.belongs_to_session(self.session_identity)
        {
            return Err(());
        }
        self.delegation_activation = Some(Arc::new(activation));
        Ok(())
    }

    pub(in crate::domain_computation::authorization) fn retain_delegation_activation_from(
        &mut self,
        prior: &Self,
    ) -> Result<(), ()> {
        if self.delegation_activation.is_some() {
            return Err(());
        }
        if let Some(activation) = &prior.delegation_activation {
            self.delegation_activation = Some(Arc::new(
                activation.retained_for_session(self.session_identity),
            ));
        }
        Ok(())
    }

    pub(in crate::domain_computation) const fn session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.session_identity
    }

    pub(in crate::domain_computation) fn primary_identity(&self) -> [u8; 32] {
        *self.relational.observation_identity().bytes()
    }

    pub(in crate::domain_computation) fn relational_counters(
        &self,
    ) -> RelationalAuthorizationObservationCounters {
        let mut counters = self.preparatory_relational_work;
        add_counters(&mut counters, self.relational.counters());
        if let Some(delegation) = &self.delegation {
            delegation.add_relational_counters(&mut counters);
        }
        if let Some(activation) = &self.delegation_activation {
            activation.add_relational_counters(&mut counters);
        }
        counters
    }

    pub(in crate::domain_computation) fn signal_dependency_count(&self) -> usize {
        bridge_dependency_count(&self.bridge)
            + self
                .delegation
                .as_ref()
                .map_or(0, |delegation| delegation.signal_dependency_count())
            + self
                .delegation_activation
                .as_ref()
                .map_or(0, |activation| activation.signal_dependency_count())
    }

    pub(in crate::domain_computation) fn bridge_is_retained(
        &self,
        bridge: &BridgeAuthorizationRuntime,
    ) -> bool {
        bridge.retains(&self.bridge)
            && bridge_matches(&self.relational, &self.bridge)
            && self
                .delegation
                .as_ref()
                .is_none_or(|delegation| delegation.bridge_is_retained(bridge))
            && self
                .delegation_activation
                .as_ref()
                .is_none_or(|activation| activation.bridge_is_retained(bridge))
    }

    pub(in crate::domain_computation) fn remains_current_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &BridgeAuthorizationRuntime,
    ) -> bool {
        self.bridge_is_retained(bridge)
            && observation_is_current(runtime, snapshot, &self.relational)
            && self
                .delegation
                .as_ref()
                .is_none_or(|delegation| delegation.remains_current_in(runtime, snapshot, bridge))
            && self
                .delegation_activation
                .as_ref()
                .is_none_or(|activation| activation.remains_current_in(runtime, snapshot, bridge))
    }

    pub(in crate::domain_computation) fn remains_equal_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> bool {
        bridge_matches(&self.relational, &self.bridge)
            && observation_is_current(runtime, snapshot, &self.relational)
            && self
                .delegation
                .as_ref()
                .is_none_or(|delegation| delegation.remains_equal_in(runtime, snapshot))
            && self
                .delegation_activation
                .as_ref()
                .is_none_or(|activation| activation.remains_equal_in(runtime, snapshot))
    }

    pub(in crate::domain_computation::authorization) fn has_same_lineage(
        &self,
        other: &Self,
    ) -> bool {
        match (&self.delegation, &other.delegation) {
            (Some(left), Some(right)) => left.has_same_lineage(right),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        }
    }
}

pub(super) fn observation_is_current(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    observation: &RelationalAuthorizationObservationEvidence,
) -> bool {
    runtime.compare_authorization_observation(observation, snapshot.clone())
        == RelationalAuthorizationObservationFreshness::Fresh
}

fn bridge_matches(
    relational: &RelationalAuthorizationObservationEvidence,
    bridge: &BridgeAuthorizationDecisionEvidence,
) -> bool {
    bridge.is_allowed() && bridge.dependency_identity() == relational.observation_identity().bytes()
}

fn bridge_dependency_count(evidence: &BridgeAuthorizationDecisionEvidence) -> usize {
    let counters = evidence.counters();
    counters.entities_depended_on
        + counters.relations_depended_on
        + counters.adjacency_lists_depended_on
        + counters.fields_depended_on
}

pub(in crate::domain_computation::authorization) fn add_counters(
    total: &mut RelationalAuthorizationObservationCounters,
    counters: RelationalAuthorizationObservationCounters,
) {
    total.paths_evaluated += counters.paths_evaluated;
    total.adjacency_lists_read += counters.adjacency_lists_read;
    total.adjacency_edges_inspected += counters.adjacency_edges_inspected;
    total.relation_records_inspected += counters.relation_records_inspected;
    total.entity_records_inspected += counters.entity_records_inspected;
    total.predicate_fields_inspected += counters.predicate_fields_inspected;
    total.relation_join_index_lookups += counters.relation_join_index_lookups;
    total.relation_join_candidates_inspected += counters.relation_join_candidates_inspected;
    total.maximum_frontier_width = total
        .maximum_frontier_width
        .max(counters.maximum_frontier_width);
    total.reconstructive_graph_scans += counters.reconstructive_graph_scans;
    total.reconstructive_relation_records_scanned +=
        counters.reconstructive_relation_records_scanned;
}
