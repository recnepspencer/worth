use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_runtime_bridge::facade::{
    BridgeCommittedPatchEnvelope, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, BridgeRuntimePolicy, BridgeSignalInvalidationDelivery,
    BridgeWritebackOutcomeClass, CoarseRoutingMode, CommittedPatchSource, InvalidationSink,
    MappingSelector, RelationalBridgeSnapshotIdentityParts, RelationalBridgeSourceError,
    RelationalCommittedPatchRequest, RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SnapshotReadContract, SnapshotReadSource, TruthBranchHeadSource,
    TruthBranchIdentity, TruthPatchScope, TruthSnapshotIdentity, TruthSnapshotReader,
    TruthWritebackAuthority, TruthWritebackAuthorityError, TruthWritebackReceipt,
    TruthWritebackRequest,
};

#[derive(Clone)]
struct RepresentativeBridgeSource;

#[derive(Clone)]
struct RepresentativeBridgeSignalSink;

#[derive(Clone)]
struct RepresentativeBridgeWritebackAuthority;

pub(crate) fn representative_bridge_authority_runtime() -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(RepresentativeBridgeSource)
        .with_signal_sink(RepresentativeBridgeSignalSink)
        .with_writeback_authority(RepresentativeBridgeWritebackAuthority)
        .register_mapping(representative_mapping_registration())
        .build()
        .expect("representative bridge authority runtime should build")
}

impl CommittedPatchSource for RepresentativeBridgeSource {
    fn load_committed_patch(
        &self,
        _request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        unreachable!("representative signal authority fixture does not load committed patches")
    }
}

impl SnapshotReadSource for RepresentativeBridgeSource {
    fn open_snapshot(
        &self,
        _identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        unreachable!("representative signal authority fixture does not open snapshots")
    }
}

impl TruthBranchHeadSource for RepresentativeBridgeSource {
    fn load_branch_head_patch(
        &self,
        _branch_identity: &TruthBranchIdentity,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        unreachable!("representative signal authority fixture does not load branch heads")
    }
}

impl InvalidationSink for RepresentativeBridgeSignalSink {
    fn deliver_invalidation(
        &self,
        _delivery: BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            0,
            TruthSnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(0, 0),
            ),
        ))
    }
}

impl TruthWritebackAuthority for RepresentativeBridgeWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError> {
        Ok(TruthWritebackReceipt::new(
            BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}

fn representative_mapping_registration() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("representative-signal-authority"),
        TruthPatchScope::for_entity_field(
            MappingSelector::any(),
            AspectKey::new("status").expect("representative aspect key should be valid"),
            FieldKey::new("value".to_string()).expect("representative field key should be valid"),
        ),
        SnapshotReadContract::scalar(
            AspectKey::new("status").expect("representative aspect key should be valid"),
            ScalarAspectType::String,
        ),
        SignalInvalidationScope::from_stable_name("representative-signal-authority"),
        CoarseRoutingMode::Direct,
    )
}
