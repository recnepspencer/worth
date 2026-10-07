use super::{native_profile_name_patch_item, DriftSnapshotReader, StaticSnapshotReader};
use crate::adapter::{
    BridgeSourceAdapter, CommittedPatchSource, RelationalBridgeSourceError,
    RelationalCommittedPatchRequest, SnapshotReadSource, TruthBranchHeadSource,
};
use crate::input::envelope::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity, TruthBranchIdentity,
    TruthPatchIdentity,
};
use crate::snapshot::{TruthSnapshotIdentity, TruthSnapshotReader};
use crate::source::{
    BridgeSourceCapability, BridgeSourceCapabilitySet, MaterializedTruthViewPacketSet,
    PlannedSourceReadPacketSet,
};

#[derive(Clone)]
pub(in crate::facade::tests) struct StaticSource;

impl CommittedPatchSource for StaticSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new(
                request.commit_identity().clone(),
                TruthPatchIdentity::from_relational_patch_position(1),
                TruthSnapshotIdentity::from_relational_snapshot(
                    crate::facade::RelationalBridgeSnapshotIdentityParts::new(1, 1),
                ),
                TruthBranchIdentity::from_relational_branch_id("analysis"),
            ),
            vec![native_profile_name_patch_item()],
        )
        .map_err(|error| RelationalBridgeSourceError::new(error.to_string()))
    }
}

impl SnapshotReadSource for StaticSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        open_static_snapshot(identity)
    }
}

impl TruthBranchHeadSource for StaticSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new(
                crate::facade::TruthCommitIdentity::from_relational_commit_id(100),
                TruthPatchIdentity::from_relational_patch_position(100),
                TruthSnapshotIdentity::from_relational_snapshot(
                    crate::facade::RelationalBridgeSnapshotIdentityParts::new(1, 1),
                ),
                branch_identity.clone(),
            ),
            vec![native_profile_name_patch_item()],
        )
        .map_err(|error| RelationalBridgeSourceError::new(error.to_string()))
    }
}

#[derive(Clone)]
pub(in crate::facade::tests) struct StaticSourceAdapter;

#[derive(Clone)]
pub(in crate::facade::tests) struct RejectingSourceAdapter;

#[derive(Clone)]
pub(in crate::facade::tests) struct DriftSourceAdapter;

#[derive(Clone)]
pub(in crate::facade::tests) struct ReorderingSourceAdapter;

impl BridgeSourceAdapter for StaticSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        native_source_capability_set()
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        open_static_snapshot(identity)
    }
}

impl BridgeSourceAdapter for RejectingSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        native_source_capability_set()
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Err(RelationalBridgeSourceError::new(format!(
            "refused snapshot `{}`",
            identity.as_str()
        )))
    }
}

impl BridgeSourceAdapter for DriftSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        native_source_capability_set()
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if is_primary_snapshot(identity) {
            Ok(Box::new(DriftSnapshotReader))
        } else {
            Err(unknown_snapshot_error(identity))
        }
    }
}

impl BridgeSourceAdapter for ReorderingSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        native_source_capability_set()
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        open_static_snapshot(identity)
    }

    fn materialize_packets(
        &self,
        planned_packet_set: &PlannedSourceReadPacketSet,
        execution_policy: crate::policy::BridgeExecutionPolicyBaseline,
    ) -> Result<MaterializedTruthViewPacketSet, crate::error::BridgeDeliveryError> {
        let observations = planned_packet_set
            .packets()
            .iter()
            .rev()
            .cloned()
            .map(|planned| {
                <StaticSourceAdapter as BridgeSourceAdapter>::materialize_packet(
                    &StaticSourceAdapter,
                    planned,
                    execution_policy,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MaterializedTruthViewPacketSet::new(
            planned_packet_set.clone(),
            observations,
        ))
    }
}

fn native_source_capability_set() -> BridgeSourceCapabilitySet {
    BridgeSourceCapabilitySet::new(vec![
        BridgeSourceCapability::SnapshotRead,
        BridgeSourceCapability::HistoricalRead,
        BridgeSourceCapability::BranchRead,
        BridgeSourceCapability::ReplayContinuityRead,
    ])
}

fn open_static_snapshot(
    identity: &TruthSnapshotIdentity,
) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
    if is_primary_snapshot(identity) {
        Ok(Box::new(StaticSnapshotReader))
    } else {
        Err(unknown_snapshot_error(identity))
    }
}

fn is_primary_snapshot(identity: &TruthSnapshotIdentity) -> bool {
    identity
        .relational_snapshot_parts()
        .is_some_and(|parts| parts.snapshot_id() == 1 && parts.version_id() == 1)
}

fn unknown_snapshot_error(identity: &TruthSnapshotIdentity) -> RelationalBridgeSourceError {
    RelationalBridgeSourceError::new(format!("unknown snapshot `{}`", identity.as_str()))
}
