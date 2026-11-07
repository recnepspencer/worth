use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::subscription::{
    QueryDeliveryBatch, QueryPatchGroupKind, QuerySubscriptionDeliveryCauseKind,
    SubscriptionConsumerAttachment,
};

use super::{
    WorthQueryAuthorityLane, WorthQueryLiveGraphReadMaintenanceReceipt,
    WorthQueryRuntimeAsyncResultState, WorthQueryRuntimeLiveSubscriptionInstallation,
    WorthQueryRuntimeMixedCauseDelivery, WorthQueryRuntimeRemaskPosture,
};

pub(super) struct WorthQueryRuntimeLiveSubscriptionActivation {
    pub(super) installation: WorthQueryRuntimeLiveSubscriptionInstallation,
    pub(super) active_lane_handle: crate::subscription::ActiveSubscriptionLaneHandle,
    pub(super) consumer_attachment: SubscriptionConsumerAttachment,
    pub(super) request: DeclarativeLiveQueryRequest,
    pub(super) remask_posture: Option<WorthQueryRuntimeRemaskPosture>,
    pub(super) read_authority_binding:
        Option<crate::intent_admission::WorthQueryReadExecutionBinding>,
}

pub(super) struct WorthQueryRuntimeLiveSubscriptionState {
    pub(super) installation: WorthQueryRuntimeLiveSubscriptionInstallation,
    pub(super) active_lane_handle: crate::subscription::ActiveSubscriptionLaneHandle,
    pub(super) consumer_attachment: SubscriptionConsumerAttachment,
    pub(super) request: DeclarativeLiveQueryRequest,
    pub(super) delivery_batches: Vec<WorthQueryRuntimeDeliveryBatch>,
    pub(super) last_delivery: Option<WorthQueryRuntimeRetainedDelivery>,
    pub(super) async_result_state: Option<WorthQueryRuntimeAsyncResultState>,
    pub(super) async_source_binding:
        Option<super::async_source_binding::WorthQueryRuntimeAsyncSourceBinding>,
    pub(super) remask_posture: Option<WorthQueryRuntimeRemaskPosture>,
    pub(super) read_authority_binding:
        Option<crate::intent_admission::WorthQueryReadExecutionBinding>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorthQueryLiveMutationRoutingWork {
    pub(crate) capability_index_lookups: usize,
    pub(crate) live_collection_index_probes: usize,
    pub(crate) live_relevance_index_probes: usize,
    pub(crate) installed_collection_index_probes: usize,
    pub(crate) installed_relevance_index_probes: usize,
    pub(crate) live_target_candidates_visited: usize,
    pub(crate) installed_target_candidates_selected: usize,
    pub(crate) installed_candidates_skipped: usize,
    pub(crate) target_overlap_deduplications: usize,
    pub(crate) installed_route_index_probes: usize,
}

impl WorthQueryLiveMutationRoutingWork {
    pub(crate) fn add(&mut self, other: Self) {
        self.capability_index_lookups += other.capability_index_lookups;
        self.live_collection_index_probes += other.live_collection_index_probes;
        self.live_relevance_index_probes += other.live_relevance_index_probes;
        self.installed_collection_index_probes += other.installed_collection_index_probes;
        self.installed_relevance_index_probes += other.installed_relevance_index_probes;
        self.live_target_candidates_visited += other.live_target_candidates_visited;
        self.installed_target_candidates_selected += other.installed_target_candidates_selected;
        self.installed_candidates_skipped += other.installed_candidates_skipped;
        self.target_overlap_deduplications += other.target_overlap_deduplications;
        self.installed_route_index_probes += other.installed_route_index_probes;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeDeliveryBatch {
    pub(super) view_name: String,
    pub(super) authority_lane: WorthQueryAuthorityLane,
    pub(super) delivery_batch_identity: WorthQueryEvidenceIdentity,
    pub(super) delivery_window_identity: WorthQueryEvidenceIdentity,
    pub(super) consumer_attachment_identity: WorthQueryEvidenceIdentity,
    pub(super) sequence: u64,
    pub(super) delivery_cause_kind: QuerySubscriptionDeliveryCauseKind,
    pub(super) delivery_cause_identity: WorthQueryEvidenceIdentity,
    pub(super) has_relational_patch: bool,
    pub(super) patch_group_kind: QueryPatchGroupKind,
    pub(super) patch_group_identity: WorthQueryEvidenceIdentity,
    pub(super) patch_group_width: u64,
    pub(super) relational_commit_identity:
        Option<worth_runtime_bridge::facade::TruthCommitIdentity>,
    pub(super) mutation_delta: Option<crate::memory_workspace::WorthQueryMutationDelta>,
    pub(super) live_graph_read_maintenance: Option<WorthQueryLiveGraphReadMaintenanceReceipt>,
    pub(super) preclassified_installed_impact:
        Option<crate::domain_installation::WorthQueryPreclassifiedInstalledLiveImpact>,
    pub(super) routing_work: WorthQueryLiveMutationRoutingWork,
    pub(super) mixed_cause_delivery: WorthQueryRuntimeMixedCauseDelivery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryRuntimeRetainedDelivery {
    delivery_batch_identity: WorthQueryEvidenceIdentity,
    delivery_cause_kind: QuerySubscriptionDeliveryCauseKind,
    delivery_cause_identity: WorthQueryEvidenceIdentity,
    has_relational_patch: bool,
    sequence: u64,
    mixed_cause_delivery: WorthQueryRuntimeMixedCauseDelivery,
}

impl WorthQueryRuntimeDeliveryBatch {
    pub(super) fn from_query_delivery(
        view_name: &str,
        batch: &QueryDeliveryBatch,
        relational_commit_identity: Option<worth_runtime_bridge::facade::TruthCommitIdentity>,
        mutation_delta: Option<crate::memory_workspace::WorthQueryMutationDelta>,
        live_graph_read_maintenance: Option<WorthQueryLiveGraphReadMaintenanceReceipt>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self {
            view_name: view_name.to_string(),
            authority_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
            delivery_batch_identity: batch.evidence_identity().clone(),
            delivery_window_identity: batch.delivery_window_identity().clone(),
            consumer_attachment_identity: batch.attachment_digest().evidence_identity().clone(),
            sequence: batch.sequence().get(),
            delivery_cause_kind: batch.delivery_cause_kind(),
            delivery_cause_identity: batch.delivery_cause().delivery_cause_identity().clone(),
            has_relational_patch: batch.has_relational_patch(),
            patch_group_kind: batch.patch_group().kind(),
            patch_group_identity: batch.patch_group().patch_group_identity().clone(),
            patch_group_width: batch.patch_group().width(),
            relational_commit_identity,
            mutation_delta,
            live_graph_read_maintenance,
            preclassified_installed_impact: None,
            routing_work: WorthQueryLiveMutationRoutingWork::default(),
            mixed_cause_delivery: if batch.has_relational_patch() {
                WorthQueryRuntimeMixedCauseDelivery::atomic_relational_patch(
                    batch.delivery_cause().delivery_cause_identity(),
                )?
            } else {
                WorthQueryRuntimeMixedCauseDelivery::atomic_time_only(
                    batch.delivery_cause_kind(),
                    batch.delivery_cause().delivery_cause_identity(),
                )?
            },
        })
    }

    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn delivery_batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_batch_identity
    }

    pub fn delivery_batch_for_reporting(&self) -> &str {
        self.delivery_batch_identity.as_str()
    }

    pub fn delivery_window_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_window_identity
    }

    pub fn delivery_window_for_reporting(&self) -> &str {
        self.delivery_window_identity.as_str()
    }

    pub fn consumer_attachment_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.consumer_attachment_identity
    }

    pub fn consumer_attachment_for_reporting(&self) -> &str {
        self.consumer_attachment_identity.as_str()
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn delivery_cause_kind(&self) -> QuerySubscriptionDeliveryCauseKind {
        self.delivery_cause_kind
    }

    pub fn delivery_cause_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_cause_identity
    }

    pub fn delivery_cause_for_reporting(&self) -> &str {
        self.delivery_cause_identity.as_str()
    }

    pub fn has_relational_patch(&self) -> bool {
        self.has_relational_patch
    }

    pub fn patch_group_kind(&self) -> QueryPatchGroupKind {
        self.patch_group_kind
    }

    pub fn patch_group_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.patch_group_identity
    }

    pub fn patch_group_for_reporting(&self) -> &str {
        self.patch_group_identity.as_str()
    }

    pub fn patch_group_width(&self) -> u64 {
        self.patch_group_width
    }

    pub fn relational_commit_identity(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::TruthCommitIdentity> {
        self.relational_commit_identity.as_ref()
    }

    pub(crate) fn mutation_delta(
        &self,
    ) -> Option<&crate::memory_workspace::WorthQueryMutationDelta> {
        self.mutation_delta.as_ref()
    }

    pub fn live_graph_read_maintenance(
        &self,
    ) -> Option<&WorthQueryLiveGraphReadMaintenanceReceipt> {
        self.live_graph_read_maintenance.as_ref()
    }

    pub(crate) fn preclassified_installed_impact(
        &self,
    ) -> Option<&crate::domain_installation::WorthQueryPreclassifiedInstalledLiveImpact> {
        self.preclassified_installed_impact.as_ref()
    }

    pub(super) fn with_preclassified_installed_impact(
        mut self,
        impact: crate::domain_installation::WorthQueryPreclassifiedInstalledLiveImpact,
    ) -> Self {
        self.preclassified_installed_impact = Some(impact);
        self
    }

    pub(super) fn with_routing_work(mut self, work: WorthQueryLiveMutationRoutingWork) -> Self {
        self.routing_work = work;
        self
    }

    pub(crate) const fn routing_work(&self) -> WorthQueryLiveMutationRoutingWork {
        self.routing_work
    }

    pub fn mixed_cause_delivery(&self) -> &WorthQueryRuntimeMixedCauseDelivery {
        &self.mixed_cause_delivery
    }
}

impl WorthQueryRuntimeRetainedDelivery {
    pub(super) fn from_batch(batch: &WorthQueryRuntimeDeliveryBatch) -> Self {
        Self {
            delivery_batch_identity: batch.delivery_batch_identity().clone(),
            delivery_cause_kind: batch.delivery_cause_kind(),
            delivery_cause_identity: batch.delivery_cause_identity().clone(),
            has_relational_patch: batch.has_relational_patch(),
            sequence: batch.sequence(),
            mixed_cause_delivery: batch.mixed_cause_delivery().clone(),
        }
    }

    pub(super) fn delivery_batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_batch_identity
    }

    pub(super) fn delivery_cause_kind(&self) -> QuerySubscriptionDeliveryCauseKind {
        self.delivery_cause_kind
    }

    pub(super) fn delivery_cause_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_cause_identity
    }

    pub(super) fn delivery_cause_for_reporting(&self) -> &str {
        self.delivery_cause_identity.as_str()
    }

    pub(super) fn has_relational_patch(&self) -> bool {
        self.has_relational_patch
    }

    pub(super) fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(super) fn mixed_cause_delivery(&self) -> &WorthQueryRuntimeMixedCauseDelivery {
        &self.mixed_cause_delivery
    }
}
