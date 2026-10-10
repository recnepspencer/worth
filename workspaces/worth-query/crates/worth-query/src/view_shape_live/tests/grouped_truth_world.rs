use worth_foundational::facade::{
    AspectKey, AspectLocator, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_runtime_bridge::facade::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem,
    BridgeCommittedPatchTarget, BridgeDeliveryReceipt, BridgeSignalInvalidationDelivery,
    BridgeSnapshotReadError, BridgeSourceAdapter, BridgeSourceCapability,
    BridgeSourceCapabilitySet, CommittedPatchSource, InvalidationSink,
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, SignalBridgeSinkError,
    SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadSource,
    TruthBranchHeadSource, TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity,
    TruthSnapshotIdentity, TruthSnapshotReader,
};

#[derive(Clone)]
pub(super) struct GroupedRowFixture {
    member_key: String,
    display_name: AspectValue,
    lane: AspectValue,
}

impl GroupedRowFixture {
    fn new(member_key: &str, display_name: &str, lane: &str) -> Self {
        Self {
            member_key: member_key.to_string(),
            display_name: crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                display_name,
            ),
            lane: crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(lane),
        }
    }

    pub(super) fn value_for_snapshot_read(&self, aspect_key: &str) -> AspectValue {
        match aspect_key {
            "identity.id" => crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                self.member_key.as_str(),
            ),
            "profile.display_name" => self.display_name.clone(),
            "status.lane" => self.lane.clone(),
            _ => crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("unknown"),
        }
    }
}

pub(super) fn grouped_row(member_key: &str, display_name: &str, lane: &str) -> GroupedRowFixture {
    GroupedRowFixture::new(member_key, display_name, lane)
}

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
            TruthPatchIdentity::from_relational_patch_position(1),
            grouped_snapshot_identity(),
            TruthBranchIdentity::from_bridge_harness_label("analysis"),
        ))
    }
}

#[derive(Clone)]
struct StaticSnapshotReader {
    rows: std::sync::Arc<Vec<GroupedRowFixture>>,
}

impl TruthSnapshotReader for StaticSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        grouped_snapshot_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        Ok(SnapshotReadPacketResult::new(
            grouped_snapshot_identity(),
            request
                .reads()
                .iter()
                .map(|read| {
                    let payload = self
                        .rows
                        .iter()
                        .enumerate()
                        .find_map(|(index, row)| {
                            (read.relational_record_identity_parts()
                                == Some(grouped_member_record_identity(index)))
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
        if identity == &grouped_snapshot_identity() {
            Ok(Box::new(StaticSnapshotReader {
                rows: self.rows.clone(),
            }))
        } else {
            Err(RelationalBridgeSourceError::new(format!(
                "unknown snapshot `{}`",
                identity
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting()
            )))
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
            TruthCommitIdentity::from_relational_commit_id(100),
            TruthPatchIdentity::from_relational_patch_position(100),
            grouped_snapshot_identity(),
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
        if identity == &grouped_snapshot_identity() {
            Ok(Box::new(StaticSnapshotReader {
                rows: self.rows.clone(),
            }))
        } else {
            Err(RelationalBridgeSourceError::new(format!(
                "unknown snapshot `{}`",
                identity
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting()
            )))
        }
    }
}

pub(super) fn grouped_snapshot_identity() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        1, 1,
    ))
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

pub(super) fn grouped_member_record_identity(index: usize) -> RelationalBridgeRecordIdentityParts {
    RelationalBridgeRecordIdentityParts::entity(1, (index + 1) as u64, 1)
}

fn native_grouped_patch_envelope(
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    branch_identity: TruthBranchIdentity,
) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new(
            commit_identity,
            patch_identity,
            snapshot_identity,
            branch_identity,
        ),
        vec![BridgeCommittedPatchItem::with_target(
            "result:task-1",
            BridgeCommittedPatchTarget::entity_field_path(
                AspectLocator::new(
                    LocatorAuthority::Authoritative,
                    AspectKey::new("status").expect("test grouped aspect key must be foundational"),
                ),
                CanonicalFieldPath::single(
                    FieldKey::new("lane".to_owned())
                        .expect("test grouped field key must be foundational"),
                ),
            ),
        )],
    )
    .expect("view-shape live native grouped patch envelope should construct")
}
