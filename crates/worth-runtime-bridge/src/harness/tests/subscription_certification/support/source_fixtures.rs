use crate::facade::BridgeCommittedPatchEnvelope;
use crate::input::envelope::{BridgeCommittedPatchItem, TruthBranchIdentity, TruthPatchIdentity};
use crate::snapshot::{SnapshotReadPacket, TruthSnapshotIdentity, TruthSnapshotReader};

#[derive(Clone)]
pub(super) struct StaticSource;

struct StaticSnapshotReader;

impl TruthSnapshotReader for StaticSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::snapshot::SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError>
    {
        Ok(crate::snapshot::SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            request
                .reads()
                .iter()
                .map(|read| {
                    crate::snapshot::SnapshotReadRecord::for_request(
                        read,
                        worth_foundational::facade::AspectValue::String("fixture-value".into()),
                    )
                })
                .collect(),
        ))
    }
}

impl crate::adapter::CommittedPatchSource for StaticSource {
    fn load_committed_patch(
        &self,
        request: crate::adapter::RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::input::envelope::BridgeCommittedPatchEnvelope,
        crate::adapter::RelationalBridgeSourceError,
    > {
        BridgeCommittedPatchEnvelope::new(
            crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
                request.commit_identity().clone(),
                TruthPatchIdentity::from_relational_patch_position(1),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                TruthBranchIdentity::from_relational_branch_id("analysis"),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                "entity-1",
                profile_name_patch_target(),
            )],
        )
        .map_err(|error| {
            crate::adapter::RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                error.to_string(),
            )
        })
    }
}

impl crate::adapter::SnapshotReadSource for StaticSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, crate::adapter::RelationalBridgeSourceError> {
        if crate::truth_identity_fixtures::truth_snapshot_fixture_matches(identity, "snapshot-a") {
            Ok(Box::new(StaticSnapshotReader))
        } else {
            Err(crate::adapter::RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                format!("unknown snapshot `{}`", identity.as_str()),
            ))
        }
    }
}

impl crate::adapter::TruthBranchHeadSource for StaticSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::input::envelope::BridgeCommittedPatchEnvelope,
        crate::adapter::RelationalBridgeSourceError,
    > {
        BridgeCommittedPatchEnvelope::new(
            crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
                crate::facade::TruthCommitIdentity::from_relational_commit_id(100),
                TruthPatchIdentity::from_relational_patch_position(100),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                branch_identity.clone(),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                "entity-1",
                profile_name_patch_target(),
            )],
        )
        .map_err(|error| {
            crate::adapter::RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                error.to_string(),
            )
        })
    }
}

pub(super) struct StaticSink;

impl crate::adapter::InvalidationSink for StaticSink {
    fn deliver_invalidation(
        &self,
        delivery: crate::routing::BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::delivery::BridgeDeliveryReceipt, crate::adapter::SignalBridgeSinkError> {
        Ok(crate::delivery::BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

#[derive(Clone)]
pub(crate) struct MisbindingSource;

struct MisbindingSnapshotReader;

impl TruthSnapshotReader for MisbindingSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-bad")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::snapshot::SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError>
    {
        Ok(crate::snapshot::SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-bad"),
            request
                .reads()
                .iter()
                .map(|read| {
                    crate::snapshot::SnapshotReadRecord::for_request(
                        read,
                        worth_foundational::facade::AspectValue::String("fixture-value".into()),
                    )
                })
                .collect(),
        ))
    }
}

impl crate::adapter::CommittedPatchSource for MisbindingSource {
    fn load_committed_patch(
        &self,
        request: crate::adapter::RelationalCommittedPatchRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::input::envelope::BridgeCommittedPatchEnvelope,
        crate::adapter::RelationalBridgeSourceError,
    > {
        StaticSource.load_committed_patch(request, execution)
    }
}

impl crate::adapter::SnapshotReadSource for MisbindingSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, crate::adapter::RelationalBridgeSourceError> {
        if crate::truth_identity_fixtures::truth_snapshot_fixture_matches(identity, "snapshot-a") {
            Ok(Box::new(MisbindingSnapshotReader))
        } else {
            Err(crate::adapter::RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                format!("unknown snapshot `{}`", identity.as_str()),
            ))
        }
    }
}

impl crate::adapter::TruthBranchHeadSource for MisbindingSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::input::envelope::BridgeCommittedPatchEnvelope,
        crate::adapter::RelationalBridgeSourceError,
    > {
        StaticSource.load_branch_head_patch(branch_identity, execution)
    }
}

#[derive(Clone)]
pub(crate) struct WrongBranchHeadSource;

impl crate::adapter::CommittedPatchSource for WrongBranchHeadSource {
    fn load_committed_patch(
        &self,
        request: crate::adapter::RelationalCommittedPatchRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::input::envelope::BridgeCommittedPatchEnvelope,
        crate::adapter::RelationalBridgeSourceError,
    > {
        StaticSource.load_committed_patch(request, execution)
    }
}

impl crate::adapter::SnapshotReadSource for WrongBranchHeadSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, crate::adapter::RelationalBridgeSourceError> {
        StaticSource.open_snapshot(identity, execution)
    }
}

impl crate::adapter::TruthBranchHeadSource for WrongBranchHeadSource {
    fn load_branch_head_patch(
        &self,
        _branch_identity: &TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::input::envelope::BridgeCommittedPatchEnvelope,
        crate::adapter::RelationalBridgeSourceError,
    > {
        BridgeCommittedPatchEnvelope::new(
            crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
                crate::truth_identity_fixtures::truth_commit_fixture("head-wrong"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-wrong"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                crate::truth_identity_fixtures::truth_branch_fixture("wrong-branch"),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                "entity-1",
                profile_name_patch_target(),
            )],
        )
        .map_err(|error| {
            crate::adapter::RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                error.to_string(),
            )
        })
    }
}

fn profile_name_patch_target() -> crate::facade::BridgeCommittedPatchTarget {
    crate::facade::BridgeCommittedPatchTarget::entity_field_path(
        worth_foundational::facade::AspectLocator::new(
            worth_foundational::facade::LocatorAuthority::Authoritative,
            profile_aspect_key(),
        ),
        worth_foundational::facade::CanonicalFieldPath::single(profile_name_field_key()),
    )
}

pub(super) fn profile_aspect_key() -> worth_foundational::facade::AspectKey {
    worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key")
}

pub(super) fn profile_name_field_key() -> worth_foundational::facade::FieldKey {
    worth_foundational::facade::FieldKey::new("name".to_owned()).expect("valid native field key")
}
