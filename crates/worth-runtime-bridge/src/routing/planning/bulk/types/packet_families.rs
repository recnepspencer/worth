#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuityRemapPacket {
    packet_identity: ContinuityPacketIdentity,
    workload_identity: BridgeWorkloadIdentity,
    originating_route_identity: BridgeRouteIdentity,
    continuity_member_identity: BulkContinuityMemberIdentity,
    branch_identity: TruthBranchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    prior_slice_count: usize,
    packet_index: usize,
    digest: Arc<str>,
}

impl ContinuityRemapPacket {
    pub(crate) fn new(
        workload_identity: BridgeWorkloadIdentity,
        originating_route_identity: BridgeRouteIdentity,
        continuity_member_identity: BulkContinuityMemberIdentity,
        branch_identity: TruthBranchIdentity,
        snapshot_identity: TruthSnapshotIdentity,
        prior_slice_count: usize,
        packet_index: usize,
    ) -> Self {
        let basis = format!(
            "continuity-remap-packet|workload={}|route={}|continuity-member={}|branch={}|snapshot={}|prior-slice-count={}|packet-index={}",
            workload_identity.as_str(),
            originating_route_identity.as_str(),
            continuity_member_identity.as_str(),
            branch_identity.canonical_label(),
            snapshot_identity.canonical_label(),
            prior_slice_count,
            packet_index,
        );
        Self {
            packet_identity: ContinuityPacketIdentity::admit_bridge_owned(digest_string(
                "continuity-packet",
                &basis,
            )),
            workload_identity,
            originating_route_identity,
            continuity_member_identity,
            branch_identity,
            snapshot_identity,
            prior_slice_count,
            packet_index,
            digest: digest_string("continuity-remap-packet", &basis),
        }
    }

    pub fn packet_identity(&self) -> &ContinuityPacketIdentity {
        &self.packet_identity
    }

    pub fn workload_identity(&self) -> &BridgeWorkloadIdentity {
        &self.workload_identity
    }

    pub fn originating_route_identity(&self) -> &BridgeRouteIdentity {
        &self.originating_route_identity
    }

    pub fn continuity_member_identity(&self) -> &BulkContinuityMemberIdentity {
        &self.continuity_member_identity
    }

    pub fn branch_identity(&self) -> String {
        self.branch_identity.canonical_label()
    }

    pub(crate) fn typed_branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn snapshot_identity(&self) -> String {
        self.snapshot_identity.canonical_label()
    }

    pub(crate) fn typed_snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn prior_slice_count(&self) -> usize {
        self.prior_slice_count
    }

    pub fn packet_index(&self) -> usize {
        self.packet_index
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthViewMaterializationPacket {
    packet_identity: TruthViewPacketIdentity,
    workload_identity: BridgeWorkloadIdentity,
    truth_view_member_identity: BulkTruthViewMemberIdentity,
    source_branch: TruthBranchIdentity,
    source_commit: TruthCommitIdentity,
    source_snapshot: TruthSnapshotIdentity,
    planned_route_count: usize,
    snapshot_read_count: usize,
    packet_index: usize,
    digest: Arc<str>,
}

impl TruthViewMaterializationPacket {
    pub(crate) fn new(
        workload_identity: BridgeWorkloadIdentity,
        truth_view_member_identity: BulkTruthViewMemberIdentity,
        source_branch: TruthBranchIdentity,
        source_commit: TruthCommitIdentity,
        source_snapshot: TruthSnapshotIdentity,
        planned_route_count: usize,
        snapshot_read_count: usize,
        packet_index: usize,
    ) -> Self {
        let basis = format!(
            "truth-view-materialization-packet|workload={}|truth-view-member={}|branch={}|commit={}|snapshot={}|planned-route-count={}|snapshot-read-count={}|packet-index={}",
            workload_identity.as_str(),
            truth_view_member_identity.as_str(),
            source_branch.canonical_label(),
            source_commit.canonical_label(),
            source_snapshot.canonical_label(),
            planned_route_count,
            snapshot_read_count,
            packet_index,
        );
        Self {
            packet_identity: TruthViewPacketIdentity::admit_bridge_owned(digest_string(
                "truth-view-packet",
                &basis,
            )),
            workload_identity,
            truth_view_member_identity,
            source_branch,
            source_commit,
            source_snapshot,
            planned_route_count,
            snapshot_read_count,
            packet_index,
            digest: digest_string("truth-view-materialization-packet", &basis),
        }
    }

    pub fn packet_identity(&self) -> &TruthViewPacketIdentity {
        &self.packet_identity
    }

    pub fn workload_identity(&self) -> &BridgeWorkloadIdentity {
        &self.workload_identity
    }

    pub fn truth_view_member_identity(&self) -> &BulkTruthViewMemberIdentity {
        &self.truth_view_member_identity
    }

    pub fn source_branch(&self) -> String {
        self.source_branch.canonical_label()
    }

    pub(crate) fn typed_source_branch(&self) -> &TruthBranchIdentity {
        &self.source_branch
    }

    pub fn source_commit(&self) -> String {
        self.source_commit.canonical_label()
    }

    pub(crate) fn typed_source_commit(&self) -> &TruthCommitIdentity {
        &self.source_commit
    }

    pub fn source_snapshot(&self) -> String {
        self.source_snapshot.canonical_label()
    }

    pub(crate) fn typed_source_snapshot(&self) -> &TruthSnapshotIdentity {
        &self.source_snapshot
    }

    pub fn planned_route_count(&self) -> usize {
        self.planned_route_count
    }

    pub fn snapshot_read_count(&self) -> usize {
        self.snapshot_read_count
    }

    pub fn packet_index(&self) -> usize {
        self.packet_index
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WideningAggregationPacket {
    packet_identity: WideningPacketIdentity,
    workload_identity: BridgeWorkloadIdentity,
    originating_route_identity: BridgeRouteIdentity,
    widening_class: BridgeMappingWideningClass,
    bounded_scope_identity: TruthDeltaSurfaceIdentity,
    packet_index: usize,
    digest: Arc<str>,
}

impl WideningAggregationPacket {
    pub(crate) fn new(
        workload_identity: BridgeWorkloadIdentity,
        originating_route_identity: BridgeRouteIdentity,
        widening_class: BridgeMappingWideningClass,
        bounded_scope_identity: TruthDeltaSurfaceIdentity,
        packet_index: usize,
    ) -> Self {
        let basis = format!(
            "widening-aggregation-packet|workload={}|route={}|widening-class={}|bounded-scope={}|packet-index={}",
            workload_identity.as_str(),
            originating_route_identity.as_str(),
            mapping_widening_class_basis(widening_class),
            bounded_scope_identity.as_str(),
            packet_index,
        );
        Self {
            packet_identity: WideningPacketIdentity::admit_bridge_owned(digest_string(
                "widening-packet",
                &basis,
            )),
            workload_identity,
            originating_route_identity,
            widening_class,
            bounded_scope_identity,
            packet_index,
            digest: digest_string("widening-aggregation-packet", &basis),
        }
    }

    pub fn packet_identity(&self) -> &WideningPacketIdentity {
        &self.packet_identity
    }

    pub fn workload_identity(&self) -> &BridgeWorkloadIdentity {
        &self.workload_identity
    }

    pub fn originating_route_identity(&self) -> &BridgeRouteIdentity {
        &self.originating_route_identity
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

    pub(crate) fn bounded_truth_delta_surface_identity(&self) -> &TruthDeltaSurfaceIdentity {
        &self.bounded_scope_identity
    }

    pub fn packet_index(&self) -> usize {
        self.packet_index
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

use super::*;
