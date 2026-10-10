use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use worth_runtime_bridge::facade::{
    BridgeCommittedPatchEnvelope, BridgeDeliveryReceipt, BridgeHistoricalLineageAuthority,
    BridgeHistoricalLineageRequest, BridgeHistoricalResolvedLineageIdentity,
    BridgeHistoricalResolvedRecordIdentity, BridgeLineageSourceError,
    BridgeSignalInvalidationDelivery, CommittedPatchSource, ContinuityLineageSource,
    InvalidationSink, RelationalBridgeRecordIdentityParts, RelationalBridgeSourceError,
    SignalBridgeSinkError, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    SnapshotReadSource, TruthBranchHeadSource, TruthBranchIdentity, TruthCommitIdentity,
    TruthSnapshotIdentity, TruthSnapshotReader,
};

use super::identity::fixture_snapshot_identity;

#[derive(Debug, Clone, Default)]
struct TestRelationalState {
    committed_patches: BTreeMap<TruthCommitIdentity, BridgeCommittedPatchEnvelope>,
    branch_heads: BTreeMap<TruthBranchIdentity, TruthCommitIdentity>,
    snapshots: BTreeMap<TruthSnapshotIdentity, Vec<SnapshotReadRecord>>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct TestRelationalSource {
    state: Arc<RwLock<TestRelationalState>>,
}

impl TestRelationalSource {
    pub(super) fn insert_committed_patch(&self, patch: BridgeCommittedPatchEnvelope) {
        let mut state = self
            .state
            .write()
            .expect("fixture bridge source lock poisoned");
        state.branch_heads.insert(
            patch.branch_identity().clone(),
            patch.commit_identity().clone(),
        );
        state
            .committed_patches
            .insert(patch.commit_identity().clone(), patch);
    }

    pub(super) fn insert_snapshot(
        &self,
        snapshot_identity: &str,
        records: Vec<SnapshotReadRecord>,
    ) {
        let snapshot_key = fixture_snapshot_identity(snapshot_identity);
        self.state
            .write()
            .expect("fixture bridge source lock poisoned")
            .snapshots
            .insert(snapshot_key, records);
    }
}

impl CommittedPatchSource for TestRelationalSource {
    fn load_committed_patch(
        &self,
        request: worth_runtime_bridge::facade::RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        self.state
            .read()
            .expect("fixture bridge source lock poisoned")
            .committed_patches
            .get(request.commit_identity())
            .cloned()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(format!(
                    "no committed patch registered for `{:?}`",
                    request.commit_identity()
                ))
            })
    }
}

impl SnapshotReadSource for TestRelationalSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        let records = self
            .state
            .read()
            .expect("fixture bridge source lock poisoned")
            .snapshots
            .get(identity)
            .cloned()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(format!(
                    "no snapshot registered for `{:?}`",
                    identity
                ))
            })?;
        Ok(Box::new(TestSnapshotReader {
            snapshot_identity: identity.clone(),
            records,
        }))
    }
}

impl TruthBranchHeadSource for TestRelationalSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        let state = self
            .state
            .read()
            .expect("fixture bridge source lock poisoned");
        let commit_identity = state.branch_heads.get(branch_identity).ok_or_else(|| {
            RelationalBridgeSourceError::new(format!(
                "no branch head registered for `{:?}`",
                branch_identity
            ))
        })?;
        state
            .committed_patches
            .get(commit_identity)
            .cloned()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(format!(
                    "branch head `{:?}` for `{:?}` had no patch envelope",
                    commit_identity, branch_identity
                ))
            })
    }
}

#[derive(Debug, Clone)]
struct TestSnapshotReader {
    snapshot_identity: TruthSnapshotIdentity,
    records: Vec<SnapshotReadRecord>,
}

impl TruthSnapshotReader for TestSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.snapshot_identity.clone()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, worth_runtime_bridge::facade::BridgeSnapshotReadError>
    {
        let fixture_value = self
            .records
            .first()
            .and_then(SnapshotReadRecord::scalar_aspect_value)
            .cloned()
            .unwrap_or_else(|| {
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("unknown")
            });
        let records = request
            .reads()
            .iter()
            .map(|read| SnapshotReadRecord::for_request(read, fixture_value.clone()))
            .collect::<Vec<_>>();
        Ok(SnapshotReadPacketResult::new(
            self.snapshot_identity.clone(),
            records,
        ))
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct NoopSignalSink;

impl InvalidationSink for NoopSignalSink {
    fn deliver_invalidation(
        &self,
        _delivery: BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            1,
            fixture_snapshot_identity(super::identity::SNAPSHOT_A),
        ))
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct FixedLineageSource;

impl ContinuityLineageSource for FixedLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::from_relational_lineage_id(1)],
            vec![
                BridgeHistoricalResolvedRecordIdentity::from_relational_record(
                    RelationalBridgeRecordIdentityParts::entity(0, 4, 2),
                ),
            ],
            vec![1],
        )
    }
}
