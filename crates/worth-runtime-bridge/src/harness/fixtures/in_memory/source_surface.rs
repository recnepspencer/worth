#[derive(Debug, Clone, Default)]
struct InMemoryRelationalState {
    committed_patches: BTreeMap<crate::facade::TruthCommitIdentity, BridgeCommittedPatchEnvelope>,
    branch_heads: BTreeMap<crate::facade::TruthBranchIdentity, crate::facade::TruthCommitIdentity>,
    snapshots: BTreeMap<TruthSnapshotIdentity, SnapshotFixture>,
    continuity_authorities: BTreeMap<String, BridgeHistoricalLineageAuthority>,
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryRelationalBridgeSource {
    state: Arc<RwLock<InMemoryRelationalState>>,
}

impl InMemoryRelationalBridgeSource {
    pub fn insert_committed_patch(&self, patch: BridgeCommittedPatchEnvelope) {
        let mut state = self.state.write().expect("bridge source lock poisoned");
        state
            .committed_patches
            .insert(patch.commit_identity().clone(), patch.clone());
        state.branch_heads.insert(
            patch.branch_identity().clone(),
            patch.commit_identity().clone(),
        );
    }

    pub fn set_branch_head(
        &self,
        branch_identity: &crate::facade::TruthBranchIdentity,
        commit_identity: &crate::facade::TruthCommitIdentity,
    ) {
        self.state
            .write()
            .expect("bridge source lock poisoned")
            .branch_heads
            .insert(branch_identity.clone(), commit_identity.clone());
    }

    pub fn insert_snapshot(&self, snapshot: SnapshotFixture) {
        self.state
            .write()
            .expect("bridge source lock poisoned")
            .snapshots
            .insert(snapshot.identity().clone(), snapshot);
    }

    pub fn insert_continuity_authority(
        &self,
        entity_identity: impl Into<String>,
        authority: BridgeHistoricalLineageAuthority,
    ) {
        self.state
            .write()
            .expect("bridge source lock poisoned")
            .continuity_authorities
            .insert(entity_identity.into(), authority);
    }
}

impl CommittedPatchSource for InMemoryRelationalBridgeSource {
    fn load_committed_patch(
        &self,
        request: crate::adapter::RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        self.state
            .read()
            .expect("bridge source lock poisoned")
            .committed_patches
            .get(request.commit_identity())
            .cloned()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(format!(
                    "no committed patch registered for `{}`",
                    request.commit_identity().as_str()
                ))
            })
    }
}

impl SnapshotReadSource for InMemoryRelationalBridgeSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        let snapshot = self
            .state
            .read()
            .expect("bridge source lock poisoned")
            .snapshots
            .get(identity)
            .cloned()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(format!(
                    "no snapshot registered for `{}`",
                    identity.as_str()
                ))
            })?;
        Ok(Box::new(InMemorySnapshotReader { snapshot }))
    }
}

impl TruthBranchHeadSource for InMemoryRelationalBridgeSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &crate::facade::TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        let state = self.state.read().expect("bridge source lock poisoned");
        let commit_identity = state.branch_heads.get(branch_identity).ok_or_else(|| {
            RelationalBridgeSourceError::new(format!(
                "no branch head registered for `{}`",
                branch_identity.as_str()
            ))
        })?;
        state
            .committed_patches
            .get(commit_identity)
            .cloned()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(format!(
                    "branch head `{}` for `{}` had no registered committed patch envelope",
                    commit_identity.as_str(),
                    branch_identity.as_str()
                ))
            })
    }
}

impl ContinuityLineageSource for InMemoryRelationalBridgeSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        self.state
            .read()
            .expect("bridge source lock poisoned")
            .continuity_authorities
            .get(request.prior_slice().entity_identity())
            .cloned()
            .ok_or_else(|| {
                BridgeLineageSourceError::new(
                    BridgeLineageSourceErrorKind::HistoricalResolutionFailure,
                    format!(
                        "no continuity lineage authority registered for `{}`",
                        request.prior_slice().entity_identity()
                    ),
                )
            })
    }
}

#[derive(Debug, Clone)]
struct InMemorySnapshotReader {
    snapshot: SnapshotFixture,
}

impl TruthSnapshotReader for InMemorySnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.snapshot.identity().clone()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError> {
        let record_lookup = self
            .snapshot
            .records()
            .iter()
            .map(|record| (record.correlation_id().clone(), record.clone()))
            .collect::<BTreeMap<_, _>>();
        let records = request
            .reads()
            .iter()
            .filter_map(|read| record_lookup.get(read.correlation_id()).cloned())
            .collect::<Vec<_>>();
        Ok(SnapshotReadPacketResult::new(
            self.snapshot.read_result_identity().clone(),
            records,
        ))
    }
}

use super::*;
