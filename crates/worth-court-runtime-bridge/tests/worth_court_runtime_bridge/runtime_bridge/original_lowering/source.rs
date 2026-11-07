//! An ordinary empty snapshot adapter; no Native owner or private source door.
use super::*;
use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};

pub(super) fn snapshot() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        1, 1,
    ))
}
pub(super) fn branch() -> TruthBranchIdentity {
    TruthBranchIdentity::from_relational_branch_id("lowering")
}
pub(super) fn runtime() -> RuntimeBridge {
    let aspect = AspectKey::new("empty").unwrap();
    RuntimeBridgeBuilder::new()
        .with_committed_patch_source(EmptySnapshot)
        .with_snapshot_read_source(EmptySnapshot)
        .with_signal_sink(EmptySnapshot)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("lowering-unused-mapping"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("unused"),
                aspect.clone(),
                FieldKey::new("value").unwrap(),
            ),
            SnapshotReadContract::scalar(aspect, ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("unused"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .unwrap()
}
pub(super) fn planned(
    runtime: &RuntimeBridge,
    execution: ExecutionRequest<'_, '_>,
) -> PlannedTruthViewPacket {
    runtime
        .plan_truth_view_packet(
            HistoricalEvaluationDeclaration::new(
                BridgeTruthViewSelector::branch_snapshot(branch(), snapshot()),
                BridgeReplayMode::Enabled,
                BridgeDiagnosticsTier::Standard,
                BridgeDeliveryIntent::PrepareSignalEvaluation,
            ),
            SnapshotReadPacket::new(vec![]),
            execution,
        )
        .unwrap()
}
pub(super) fn admit(
    runtime: &RuntimeBridge,
    planned: PlannedTruthViewPacket,
    execution: ExecutionRequest<'_, '_>,
) -> Result<BridgeBoundExecutionBasis, BridgeExecutionBasisDenial> {
    runtime.admit_managed_execution_basis(
        BridgeManagedExecutionIntent::new("original-lowering", "one-attempt"),
        BridgeManagedExecutionStepContract::new(
            "one-step",
            BridgeManagedExecutionStepLimits::new(8, 4, 2).with_memory_ceilings(64, 32),
            BridgeManagedExecutionPartialEffectPosture::None,
        )
        .unwrap(),
        BridgeAsyncRequestTruthViewBasis::branch_head(branch(), snapshot()),
        planned,
        execution,
    )
}
struct EmptySnapshot;
impl CommittedPatchSource for EmptySnapshot {
    fn load_committed_patch(
        &self,
        _: RelationalCommittedPatchRequest,
        _: ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        panic!("snapshot admission does not read a committed patch")
    }
}
impl SnapshotReadSource for EmptySnapshot {
    fn open_snapshot(
        &self,
        selected: &TruthSnapshotIdentity,
        _: ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        assert_eq!(selected, &snapshot());
        Ok(Box::new(EmptySnapshot))
    }
}
impl TruthSnapshotReader for EmptySnapshot {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        snapshot()
    }
    fn read_packet(
        &self,
        packet: &SnapshotReadPacket,
        _: ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        assert!(packet.reads().is_empty());
        Ok(SnapshotReadPacketResult::new(snapshot(), vec![]))
    }
}
impl InvalidationSink for EmptySnapshot {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        panic!("snapshot admission does not send invalidation")
    }
}
