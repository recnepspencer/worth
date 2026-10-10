use super::*;

#[derive(Clone)]
pub(super) struct StaticSource {
    pub(super) rows: std::sync::Arc<Vec<GroupedRowFixture>>,
}

impl CommittedPatchSource for StaticSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(native_grouped_patch_envelope(
            request.commit_identity().clone(),
            milestone_eight_patch_identity_for_commit(request.commit_identity()),
            milestone_eight_snapshot_identity(),
            milestone_eight_branch_identity(),
        ))
    }
}

#[derive(Clone)]
struct StaticSnapshotReader {
    rows: std::sync::Arc<Vec<GroupedRowFixture>>,
}

impl TruthSnapshotReader for StaticSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        milestone_eight_snapshot_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        Ok(SnapshotReadPacketResult::new(
            milestone_eight_snapshot_identity(),
            request
                .reads()
                .iter()
                .map(|read| {
                    let payload = self
                        .rows
                        .iter()
                        .find_map(|row| {
                            (read.relational_record_identity_parts()
                                == Some(milestone_eight_record_parts(row.member_key())))
                            .then(|| row.value_for_snapshot_read(read.aspect_key().as_str()))
                        })
                        .unwrap_or_else(|| {
                            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                                "unknown",
                            )
                        });
                    SnapshotReadRecord::for_request(read, payload)
                })
                .collect(),
        ))
    }
}

impl SnapshotReadSource for StaticSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if identity.relational_snapshot_parts() == Some(milestone_eight_snapshot_parts()) {
            Ok(Box::new(StaticSnapshotReader {
                rows: self.rows.clone(),
            }))
        } else {
            Err(RelationalBridgeSourceError::new(
                worth_runtime_bridge::facade::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                "unknown snapshot identity",
            ))
        }
    }
}

impl TruthBranchHeadSource for StaticSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(native_grouped_patch_envelope(
            milestone_eight_head_commit_identity(branch_identity),
            milestone_eight_patch_identity_for_branch(branch_identity),
            milestone_eight_snapshot_identity(),
            branch_identity.clone(),
        ))
    }
}

#[derive(Clone)]
pub(super) struct StaticSourceAdapter {
    pub(super) rows: std::sync::Arc<Vec<GroupedRowFixture>>,
}

impl BridgeSourceAdapter for StaticSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ])
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if identity.relational_snapshot_parts() == Some(milestone_eight_snapshot_parts()) {
            Ok(Box::new(StaticSnapshotReader {
                rows: self.rows.clone(),
            }))
        } else {
            Err(RelationalBridgeSourceError::new(
                worth_runtime_bridge::facade::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                "unknown snapshot identity",
            ))
        }
    }
}

pub(super) struct StaticSink;

impl InvalidationSink for StaticSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}
