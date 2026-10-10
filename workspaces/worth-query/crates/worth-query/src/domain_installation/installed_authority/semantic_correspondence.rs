use std::marker::PhantomData;
use std::sync::Arc;

use worth_proof::TransitionOutcome;

use super::WorthQueryInstalledDomainOperation;
use crate::domain_installation::{
    WorthQueryInstalledDomainAuthority, WorthQueryInstalledGraphParticipation,
    WorthQueryInstalledGraphParticipationRecord,
};

type CorrespondenceMarker<D, O, F, G> = fn() -> (D, O, F, G);

pub type WorthQueryInstalledSemanticCorrespondenceOutcome<D, O, F, G> = TransitionOutcome<
    WorthQueryInstalledSemanticCorrespondence<D, O, F, G>,
    worth_runtime_bridge::facade::BridgeCorrespondenceDenial,
    worth_runtime_bridge::facade::BridgeCorrespondenceDeferred,
    worth_runtime_bridge::facade::BridgeCorrespondenceStale,
    worth_runtime_bridge::facade::BridgeCorrespondenceRebindRequired,
    worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionFailure,
>;

/// Query-owned proof that a Bridge/Signal correspondence was installed from
/// one exact installed operation and graph-participation authority. This is
/// intentionally not a bound operation capability; Phase 10 must still join it
/// to the matching bound graph-read authority.
pub struct WorthQueryInstalledSemanticCorrespondence<D, O, F, G> {
    domain_authority: Arc<WorthQueryInstalledDomainAuthority>,
    operation_authority:
        Arc<worth_query_installation::facade::WorthQueryInstalledDomainOperationAuthority>,
    graph_authority: Arc<WorthQueryInstalledGraphParticipationRecord>,
    location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
    dependency_ordinal: usize,
    bridge: worth_runtime_bridge::facade::BridgeInstalledSemanticCorrespondence,
    _marker: PhantomData<CorrespondenceMarker<D, O, F, G>>,
}

impl<D, O, F, G> WorthQueryInstalledSemanticCorrespondence<D, O, F, G> {
    pub fn target_count(&self) -> usize {
        self.bridge.target_count()
    }

    pub fn admission_counters(
        &self,
    ) -> worth_runtime_bridge::facade::CorrespondenceAdmissionCounters {
        self.bridge.admission_counters()
    }

    pub fn deliver_authoritative_change(
        &self,
        graph: &mut worth_runtime_bridge::facade::BridgeSignalGraphBinding<'_, '_>,
        request: worth_runtime_bridge::facade::RelationalCommittedPatchRequest,
        resource_request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> worth_runtime_bridge::facade::CorrespondenceDeliveryOutcome {
        graph.deliver_installed_correspondence(&self.bridge, request, resource_request)
    }

    pub fn graph_participation_identity(&self) -> &str {
        &self.graph_authority.authority_identity
    }

    pub fn installation_generation(&self) -> u64 {
        self.domain_authority.installation_generation().ordinal()
    }

    pub fn conditional_node_location(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryConditionalNodeLocation {
        &self.location
    }

    pub fn dependency_ordinal(&self) -> usize {
        self.dependency_ordinal
    }

    pub fn operation_slot(&self) -> String {
        self.operation_authority.operation_slot()
    }
}

impl<D: 'static, O: 'static, F: 'static> WorthQueryInstalledDomainOperation<D, O, F> {
    pub fn install_semantic_correspondence<G: 'static>(
        &self,
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        dependency_ordinal: usize,
        graph_participation: &WorthQueryInstalledGraphParticipation<G>,
        source_record_identity: Option<
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        >,
        graph: &mut worth_runtime_bridge::facade::BridgeSignalGraphBinding<'_, '_>,
    ) -> WorthQueryInstalledSemanticCorrespondenceOutcome<D, O, F, G> {
        let candidate = match self.semantic_correspondence_candidate(
            location.clone(),
            dependency_ordinal,
            graph_participation,
            source_record_identity,
        ) {
            Ok(candidate) => candidate,
            Err(denial) => return TransitionOutcome::Denied(denial),
        };
        let domain_authority = Arc::clone(self.domain_authority());
        let operation_authority = Arc::clone(self.operation_authority());
        let graph_authority = Arc::clone(&graph_participation.record);
        graph
            .install_semantic_correspondence(candidate)
            .map_success(|installed| WorthQueryInstalledSemanticCorrespondence {
                domain_authority,
                operation_authority,
                graph_authority,
                location,
                dependency_ordinal,
                bridge: installed,
                _marker: PhantomData,
            })
    }
}
