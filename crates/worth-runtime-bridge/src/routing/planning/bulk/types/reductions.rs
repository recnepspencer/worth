#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducedContinuityRemap {
    continuity_identity: ReducedContinuityIdentity,
    continuity_member_identity: BulkContinuityMemberIdentity,
    branch_identity: TruthBranchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    reduced_route_count: usize,
    prior_slice_count: usize,
    digest: Arc<str>,
}

impl ReducedContinuityRemap {
    pub(crate) fn new(
        continuity_identity: ReducedContinuityIdentity,
        continuity_member_identity: BulkContinuityMemberIdentity,
        branch_identity: TruthBranchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
        reduced_route_count: usize,
        prior_slice_count: usize,
    ) -> Self {
        let basis = format!(
            "reduced-continuity-remap|identity={}|continuity-member={}|branch={}|snapshot={}|reduced-route-count={}|prior-slice-count={}",
            continuity_identity.as_str(),
            continuity_member_identity.as_str(),
            branch_identity.canonical_label(),
            snapshot_identity.canonical_label(),
            reduced_route_count,
            prior_slice_count,
        );
        Self {
            continuity_identity,
            continuity_member_identity,
            branch_identity,
            snapshot_identity,
            reduced_route_count,
            prior_slice_count,
            digest: digest_string("reduced-continuity-remap", &basis),
        }
    }

    pub fn continuity_identity(&self) -> &ReducedContinuityIdentity {
        &self.continuity_identity
    }
    pub fn continuity_member_identity(&self) -> &BulkContinuityMemberIdentity {
        &self.continuity_member_identity
    }
    pub fn branch_identity(&self) -> String {
        self.branch_identity.canonical_label()
    }
    pub fn snapshot_identity(&self) -> String {
        self.snapshot_identity.canonical_label()
    }
    pub fn reduced_route_count(&self) -> usize {
        self.reduced_route_count
    }
    pub fn prior_slice_count(&self) -> usize {
        self.prior_slice_count
    }
    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducedTruthViewMaterialization {
    truth_view_identity: ReducedTruthViewIdentity,
    truth_view_member_identity: BulkTruthViewMemberIdentity,
    source_branch: TruthBranchIdentity,
    source_commit: TruthCommitIdentity,
    source_snapshot: TruthSnapshotIdentity,
    planned_route_count: usize,
    snapshot_read_count: usize,
    digest: Arc<str>,
}

impl ReducedTruthViewMaterialization {
    pub(crate) fn new(
        truth_view_identity: ReducedTruthViewIdentity,
        truth_view_member_identity: BulkTruthViewMemberIdentity,
        source_branch: TruthBranchIdentity,
        source_commit: TruthCommitIdentity,
        source_snapshot: TruthSnapshotIdentity,
        planned_route_count: usize,
        snapshot_read_count: usize,
    ) -> Self {
        let basis = format!(
            "reduced-truth-view-materialization|identity={}|truth-view-member={}|branch={}|commit={}|snapshot={}|planned-route-count={}|snapshot-read-count={}",
            truth_view_identity.as_str(),
            truth_view_member_identity.as_str(),
            source_branch.canonical_label(),
            source_commit.canonical_label(),
            source_snapshot.canonical_label(),
            planned_route_count,
            snapshot_read_count,
        );
        Self {
            truth_view_identity,
            truth_view_member_identity,
            source_branch,
            source_commit,
            source_snapshot,
            planned_route_count,
            snapshot_read_count,
            digest: digest_string("reduced-truth-view-materialization", &basis),
        }
    }

    pub fn truth_view_identity(&self) -> &ReducedTruthViewIdentity {
        &self.truth_view_identity
    }
    pub fn truth_view_member_identity(&self) -> &BulkTruthViewMemberIdentity {
        &self.truth_view_member_identity
    }
    pub fn source_branch(&self) -> String {
        self.source_branch.canonical_label()
    }
    pub fn source_commit(&self) -> String {
        self.source_commit.canonical_label()
    }
    pub fn source_snapshot(&self) -> String {
        self.source_snapshot.canonical_label()
    }
    pub fn planned_route_count(&self) -> usize {
        self.planned_route_count
    }
    pub fn snapshot_read_count(&self) -> usize {
        self.snapshot_read_count
    }
    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducedBridgePublication {
    routing_target_identity: ReducedRoutingTargetIdentity,
    publication_identity: ReducedPublicationIdentity,
    subscription_slice_identity: BridgeSubscriptionSliceIdentity,
    reduced_route_identities: Arc<[BridgeRouteIdentity]>,
    invalidation_target_count: usize,
    digest: Arc<str>,
}

impl ReducedBridgePublication {
    pub(crate) fn new(
        routing_target_identity: ReducedRoutingTargetIdentity,
        publication_identity: ReducedPublicationIdentity,
        subscription_slice_identity: BridgeSubscriptionSliceIdentity,
        reduced_route_identities: Vec<BridgeRouteIdentity>,
        invalidation_target_count: usize,
    ) -> Self {
        let mut basis = format!(
            "reduced-bridge-publication|routing-target={}|publication={}|subscription-slice={}|route-count={}|invalidation-target-count={}",
            routing_target_identity.as_str(),
            publication_identity.as_str(),
            subscription_slice_identity.as_str(),
            reduced_route_identities.len(),
            invalidation_target_count,
        );
        for route_identity in &reduced_route_identities {
            basis.push_str("|route=");
            basis.push_str(route_identity.as_str());
        }
        Self {
            routing_target_identity,
            publication_identity,
            subscription_slice_identity,
            reduced_route_identities: reduced_route_identities.into(),
            invalidation_target_count,
            digest: digest_string("reduced-bridge-publication", &basis),
        }
    }

    pub fn routing_target_identity(&self) -> &ReducedRoutingTargetIdentity {
        &self.routing_target_identity
    }

    pub fn publication_identity(&self) -> &ReducedPublicationIdentity {
        &self.publication_identity
    }

    pub fn subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        &self.subscription_slice_identity
    }

    pub fn reduced_route_identities(&self) -> &[BridgeRouteIdentity] {
        &self.reduced_route_identities
    }

    pub fn invalidation_target_count(&self) -> usize {
        self.invalidation_target_count
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducedWideningAggregation {
    widening_identity: ReducedWideningIdentity,
    widening_class: BridgeMappingWideningClass,
    bounded_scope_identity: TruthDeltaSurfaceIdentity,
    reduced_route_identities: Arc<[BridgeRouteIdentity]>,
    digest: Arc<str>,
}

impl ReducedWideningAggregation {
    pub(crate) fn new(
        widening_identity: ReducedWideningIdentity,
        widening_class: BridgeMappingWideningClass,
        bounded_scope_identity: TruthDeltaSurfaceIdentity,
        reduced_route_identities: Vec<BridgeRouteIdentity>,
    ) -> Self {
        let mut basis = format!(
            "reduced-widening-aggregation|identity={}|widening-class={}|bounded-scope={}|route-count={}",
            widening_identity.as_str(),
            mapping_widening_class_basis(widening_class),
            bounded_scope_identity.as_str(),
            reduced_route_identities.len(),
        );
        for route_identity in &reduced_route_identities {
            basis.push_str("|route=");
            basis.push_str(route_identity.as_str());
        }
        Self {
            widening_identity,
            widening_class,
            bounded_scope_identity,
            reduced_route_identities: reduced_route_identities.into(),
            digest: digest_string("reduced-widening-aggregation", &basis),
        }
    }

    pub fn widening_identity(&self) -> &ReducedWideningIdentity {
        &self.widening_identity
    }

    pub fn widening_class(&self) -> BridgeMappingWideningClass {
        self.widening_class
    }

    pub fn widening_class_label(&self) -> &'static str {
        mapping_widening_class_basis(self.widening_class)
    }

    pub fn bounded_scope_identity(&self) -> &str {
        self.bounded_scope_identity.as_str()
    }

    #[cfg(test)]
    pub(crate) fn bounded_truth_delta_surface_identity(&self) -> &TruthDeltaSurfaceIdentity {
        &self.bounded_scope_identity
    }

    pub fn reduced_route_identities(&self) -> &[BridgeRouteIdentity] {
        &self.reduced_route_identities
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducedBridgeWorkloadArtifact {
    workload_identity: BridgeWorkloadIdentity,
    reduced_continuity_remaps: Arc<[ReducedContinuityRemap]>,
    reduced_truth_views: Arc<[ReducedTruthViewMaterialization]>,
    reduced_widenings: Arc<[ReducedWideningAggregation]>,
    reduced_publications: Arc<[ReducedBridgePublication]>,
    counters: BridgeBulkPlanningCounters,
    digest: Arc<str>,
}

impl ReducedBridgeWorkloadArtifact {
    pub(crate) fn new(
        workload_identity: BridgeWorkloadIdentity,
        reduced_continuity_remaps: Vec<ReducedContinuityRemap>,
        reduced_truth_views: Vec<ReducedTruthViewMaterialization>,
        reduced_widenings: Vec<ReducedWideningAggregation>,
        reduced_publications: Vec<ReducedBridgePublication>,
        counters: BridgeBulkPlanningCounters,
    ) -> Self {
        let reduction_output_count = reduced_continuity_remaps.len()
            + reduced_truth_views.len()
            + reduced_widenings.len()
            + reduced_publications.len();
        let mut basis = format!(
            "reduced-bridge-workload-artifact|workload={}|reduction-input-count={}|reduction-output-count={}",
            workload_identity.as_str(),
            counters.bulk_reduction_input_count(),
            reduction_output_count,
        );
        for continuity in &reduced_continuity_remaps {
            basis.push_str("|continuity=");
            basis.push_str(continuity.digest());
        }
        for truth_view in &reduced_truth_views {
            basis.push_str("|truth-view=");
            basis.push_str(truth_view.digest());
        }
        for widening in &reduced_widenings {
            basis.push_str("|widening=");
            basis.push_str(widening.digest());
        }
        for publication in &reduced_publications {
            basis.push_str("|publication=");
            basis.push_str(publication.digest());
        }
        Self {
            workload_identity,
            reduced_continuity_remaps: reduced_continuity_remaps.into(),
            reduced_truth_views: reduced_truth_views.into(),
            reduced_widenings: reduced_widenings.into(),
            reduced_publications: reduced_publications.into(),
            counters,
            digest: digest_string("reduced-bridge-workload-artifact", &basis),
        }
    }

    pub fn workload_identity(&self) -> &BridgeWorkloadIdentity {
        &self.workload_identity
    }

    pub fn reduced_continuity_remaps(&self) -> &[ReducedContinuityRemap] {
        &self.reduced_continuity_remaps
    }

    pub fn reduced_truth_views(&self) -> &[ReducedTruthViewMaterialization] {
        &self.reduced_truth_views
    }

    pub fn reduced_widenings(&self) -> &[ReducedWideningAggregation] {
        &self.reduced_widenings
    }

    pub fn reduced_publications(&self) -> &[ReducedBridgePublication] {
        &self.reduced_publications
    }

    pub fn reduction_input_count(&self) -> usize {
        self.counters.bulk_reduction_input_count()
    }

    pub fn reduction_output_count(&self) -> usize {
        self.counters.bulk_reduction_output_count()
    }

    pub fn counters(&self) -> &BridgeBulkPlanningCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

use super::*;
