use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use crate::facade::{
    BridgeCommittedPatchEnvelope, CommittedPatchSource, RelationalBridgeSourceError,
    RelationalCommittedPatchRequest, SnapshotReadPacket, SnapshotReadPacketResult,
    SnapshotReadRecord, SnapshotReadSource, TruthBranchHeadSource, TruthBranchIdentity,
    TruthCommitIdentity, TruthSnapshotIdentity, TruthSnapshotReader,
};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::publication::PublishedAuthoritativePatchEnvelope;

use super::patch_envelopes::publication_patch_to_bridge_envelope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationBridgeSnapshot {
    identity: TruthSnapshotIdentity,
    read_result_identity: TruthSnapshotIdentity,
    records: Vec<SnapshotReadRecord>,
}

impl PublicationBridgeSnapshot {
    pub fn new(identity: TruthSnapshotIdentity, records: Vec<SnapshotReadRecord>) -> Self {
        Self {
            read_result_identity: identity.clone(),
            identity,
            records,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PublicationBridgeCatalog {
    state: Arc<RwLock<PublicationBridgeCatalogState>>,
}

#[derive(Debug, Default)]
struct PublicationBridgeCatalogState {
    committed_patches: BTreeMap<TruthCommitIdentity, BridgeCommittedPatchEnvelope>,
    snapshots: BTreeMap<TruthSnapshotIdentity, PublicationBridgeSnapshot>,
}

impl PublicationBridgeCatalog {
    pub fn register_patch(
        &self,
        commit_id: CommitId,
        branch_id: &BranchId,
        snapshot_identity: TruthSnapshotIdentity,
        patch: &PublishedAuthoritativePatchEnvelope,
    ) -> Result<(), RelationalBridgeSourceError> {
        let envelope = match publication_patch_to_bridge_envelope(
            commit_id,
            branch_id,
            snapshot_identity,
            patch,
        ) {
            worth_proof::TransitionOutcome::Success(envelope) => envelope,
            worth_proof::TransitionOutcome::Denied(denial) => {
                return Err(RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    format!("publication patch could not be admitted by Bridge: {denial}"),
                ));
            }
        };
        self.state
            .write()
            .expect("publication bridge catalog lock poisoned")
            .committed_patches
            .insert(envelope.commit_identity().clone(), envelope);
        Ok(())
    }

    pub fn register_snapshot(&self, snapshot: PublicationBridgeSnapshot) {
        self.state
            .write()
            .expect("publication bridge catalog lock poisoned")
            .snapshots
            .insert(snapshot.identity.clone(), snapshot);
    }
}

impl CommittedPatchSource for PublicationBridgeCatalog {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        self.state
            .read()
            .expect("publication bridge catalog lock poisoned")
            .committed_patches
            .get(request.commit_identity())
            .cloned()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    "no publication bridge patch registered for commit",
                )
            })
    }
}

impl SnapshotReadSource for PublicationBridgeCatalog {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        let snapshot = self
            .state
            .read()
            .expect("publication bridge catalog lock poisoned")
            .snapshots
            .get(identity)
            .cloned()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    "no publication bridge snapshot registered",
                )
            })?;
        Ok(Box::new(PublicationSnapshotReader { snapshot }))
    }
}

impl TruthBranchHeadSource for PublicationBridgeCatalog {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        self.state
            .read()
            .expect("publication bridge catalog lock poisoned")
            .committed_patches
            .values()
            .filter(|envelope| envelope.branch_identity() == branch_identity)
            .cloned()
            .next_back()
            .ok_or_else(|| {
                RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    "no publication bridge branch head registered",
                )
            })
    }
}

#[derive(Debug, Clone)]
struct PublicationSnapshotReader {
    snapshot: PublicationBridgeSnapshot,
}

impl TruthSnapshotReader for PublicationSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.snapshot.identity.clone()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, crate::facade::BridgeSnapshotReadError> {
        let records_by_key = self
            .snapshot
            .records
            .iter()
            .map(|record| (record.correlation_id().clone(), record.clone()))
            .collect::<BTreeMap<_, _>>();
        let records = request
            .reads()
            .iter()
            .filter_map(|read| records_by_key.get(read.correlation_id()).cloned())
            .collect::<Vec<_>>();
        Ok(SnapshotReadPacketResult::new(
            self.snapshot.read_result_identity.clone(),
            records,
        ))
    }
}
