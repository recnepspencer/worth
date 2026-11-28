#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeBulkWorkloadSegment {
    request: BridgeRouteRequest,
    mapping_context: BridgeMappingContext,
}

impl BridgeBulkWorkloadSegment {
    pub fn new(request: BridgeRouteRequest) -> Self {
        Self {
            request,
            mapping_context: BridgeMappingContext::default(),
        }
    }

    pub fn with_mapping_context(mut self, mapping_context: BridgeMappingContext) -> Self {
        self.mapping_context = mapping_context;
        self
    }

    pub fn request(&self) -> &BridgeRouteRequest {
        &self.request
    }

    pub fn mapping_context(&self) -> &BridgeMappingContext {
        &self.mapping_context
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeBulkWorkloadRequest {
    segments: Vec<BridgeBulkWorkloadSegment>,
}

impl BridgeBulkWorkloadRequest {
    pub fn new(segments: Vec<BridgeBulkWorkloadSegment>) -> Self {
        Self { segments }
    }

    pub fn from_requests(requests: Vec<BridgeRouteRequest>) -> Self {
        Self {
            segments: requests
                .into_iter()
                .map(BridgeBulkWorkloadSegment::new)
                .collect(),
        }
    }

    pub fn segments(&self) -> &[BridgeBulkWorkloadSegment] {
        &self.segments
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalBridgeWorkloadRequest {
    workload_identity: BridgeWorkloadIdentity,
    route_members: Arc<[BridgeRouteIdentity]>,
    subscription_slice_members: Arc<[BridgeSubscriptionSliceIdentity]>,
    continuity_members: Arc<[BulkContinuityMemberIdentity]>,
    truth_view_members: Arc<[BulkTruthViewMemberIdentity]>,
    commit_members: Arc<[TruthCommitIdentity]>,
    snapshot_members: Arc<[TruthSnapshotIdentity]>,
    branch_members: Arc<[TruthBranchIdentity]>,
    workload_segment_identities: Arc<[BulkWorkloadSegmentIdentity]>,
    digest: Arc<str>,
}

impl CanonicalBridgeWorkloadRequest {
    pub(crate) fn new(
        workload_identity: BridgeWorkloadIdentity,
        route_members: Vec<BridgeRouteIdentity>,
        subscription_slice_members: Vec<BridgeSubscriptionSliceIdentity>,
        continuity_members: Vec<BulkContinuityMemberIdentity>,
        truth_view_members: Vec<BulkTruthViewMemberIdentity>,
        commit_members: Vec<TruthCommitIdentity>,
        snapshot_members: Vec<TruthSnapshotIdentity>,
        branch_members: Vec<TruthBranchIdentity>,
        workload_segment_identities: Vec<BulkWorkloadSegmentIdentity>,
    ) -> Self {
        let mut basis = format!(
            "canonical-bridge-workload-request|workload={}|route-count={}|slice-count={}|continuity-count={}|truth-view-count={}|commit-count={}|snapshot-count={}|branch-count={}|segment-count={}",
            workload_identity.as_str(),
            route_members.len(),
            subscription_slice_members.len(),
            continuity_members.len(),
            truth_view_members.len(),
            commit_members.len(),
            snapshot_members.len(),
            branch_members.len(),
            workload_segment_identities.len(),
        );
        for member in &route_members {
            basis.push_str("|route=");
            basis.push_str(member.as_str());
        }
        for member in &subscription_slice_members {
            basis.push_str("|slice=");
            basis.push_str(member.as_str());
        }
        for member in &continuity_members {
            basis.push_str("|continuity=");
            basis.push_str(member.as_str());
        }
        for member in &truth_view_members {
            basis.push_str("|truth-view=");
            basis.push_str(member.as_str());
        }
        for member in &commit_members {
            basis.push_str("|commit=");
            basis.push_str(&member.canonical_label());
        }
        for member in &snapshot_members {
            basis.push_str("|snapshot=");
            basis.push_str(&member.canonical_label());
        }
        for member in &branch_members {
            basis.push_str("|branch=");
            basis.push_str(&member.canonical_label());
        }
        for segment in &workload_segment_identities {
            basis.push_str("|segment=");
            basis.push_str(segment.as_str());
        }
        Self {
            workload_identity,
            route_members: route_members.into(),
            subscription_slice_members: subscription_slice_members.into(),
            continuity_members: continuity_members.into(),
            truth_view_members: truth_view_members.into(),
            commit_members: commit_members.into(),
            snapshot_members: snapshot_members.into(),
            branch_members: branch_members.into(),
            workload_segment_identities: workload_segment_identities.into(),
            digest: digest_string("canonical-bridge-workload-request", &basis),
        }
    }

    pub fn workload_identity(&self) -> &BridgeWorkloadIdentity {
        &self.workload_identity
    }
    pub fn route_members(&self) -> &[BridgeRouteIdentity] {
        &self.route_members
    }
    pub fn subscription_slice_members(&self) -> &[BridgeSubscriptionSliceIdentity] {
        &self.subscription_slice_members
    }
    pub fn continuity_members(&self) -> &[BulkContinuityMemberIdentity] {
        &self.continuity_members
    }
    pub fn truth_view_members(&self) -> &[BulkTruthViewMemberIdentity] {
        &self.truth_view_members
    }
    pub fn commit_members(&self) -> &[TruthCommitIdentity] {
        &self.commit_members
    }
    pub fn snapshot_members(&self) -> &[TruthSnapshotIdentity] {
        &self.snapshot_members
    }
    pub fn branch_members(&self) -> &[TruthBranchIdentity] {
        &self.branch_members
    }
    pub fn workload_segment_identities(&self) -> &[BulkWorkloadSegmentIdentity] {
        &self.workload_segment_identities
    }
    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
use super::*;
