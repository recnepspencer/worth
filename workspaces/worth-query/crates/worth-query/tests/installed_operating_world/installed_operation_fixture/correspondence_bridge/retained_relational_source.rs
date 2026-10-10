use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_runtime_bridge::facade::{
    BridgeAuthoritativeSourceProfile, BridgeCommittedPatchEnvelope, CommittedPatchSource,
    RelationalBridgeObservationLease, RelationalBridgeSnapshotIdentityParts,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridgeRelationalSource,
    SnapshotReadSource, TruthSnapshotIdentity, TruthSnapshotReader,
};

pub(super) struct RetainedRelationalSource {
    source: RuntimeBridgeRelationalSource,
    _observations: Vec<RelationalBridgeObservationLease>,
}

impl RetainedRelationalSource {
    pub(super) fn new(
        source: RuntimeBridgeRelationalSource,
        retained_bases: Vec<AdmittedRelationalBranchBasis>,
    ) -> (Self, Vec<RelationalBridgeSnapshotIdentityParts>) {
        let observations: Vec<_> = retained_bases
            .iter()
            .map(|basis| {
                source
                    .retain_branch_basis_for_bridge(basis)
                    .expect("fixture Bridge observation should retain")
            })
            .collect();
        let snapshots = observations
            .iter()
            .map(|observation| {
                observation
                    .snapshot_identity()
                    .relational_snapshot_parts()
                    .expect("Relational Bridge observation identity")
            })
            .collect();
        (
            Self {
                source,
                _observations: observations,
            },
            snapshots,
        )
    }
}

impl CommittedPatchSource for RetainedRelationalSource {
    fn authoritative_source_profile(&self) -> Option<BridgeAuthoritativeSourceProfile> {
        CommittedPatchSource::authoritative_source_profile(&self.source)
    }

    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        self.source.load_committed_patch(request, _execution)
    }
}

impl SnapshotReadSource for RetainedRelationalSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        self.source.open_snapshot(identity, _execution)
    }
}
