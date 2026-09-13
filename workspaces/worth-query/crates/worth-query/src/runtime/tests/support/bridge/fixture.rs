use crate::runtime::tests::support::*;
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;

#[derive(Clone, Debug)]
pub(in crate::runtime::tests) struct TestBridgeSource;

impl worth_runtime_bridge::facade::CommittedPatchSource for TestBridgeSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(native_patch_envelope(
            request.commit_identity().clone(),
            "entity",
            "aspect",
            "field",
        ))
    }
}

impl SnapshotReadSource for TestBridgeSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Ok(Box::new(TestSnapshotReader {
            identity: identity.clone(),
        }))
    }
}

pub(in crate::runtime::tests) struct TestSnapshotReader {
    identity: TruthSnapshotIdentity,
}

impl TruthSnapshotReader for TestSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.identity.clone()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, worth_runtime_bridge::facade::BridgeSnapshotReadError>
    {
        Ok(SnapshotReadPacketResult::new(
            self.identity.clone(),
            request
                .reads()
                .iter()
                .map(|read| {
                    SnapshotReadRecord::for_request(
                        read,
                        worth_foundational::facade::AspectValue::Null,
                    )
                })
                .collect(),
        ))
    }
}

pub(in crate::runtime::tests) struct TestBridgeSink;

impl InvalidationSink for TestBridgeSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

#[derive(Clone, Debug)]
pub(in crate::runtime::tests) struct StaticWritebackAuthority;

impl worth_runtime_bridge::facade::TruthWritebackAuthority for StaticWritebackAuthority {
    fn execute_writeback(
        &self,
        request: worth_runtime_bridge::facade::TruthWritebackRequest,
    ) -> Result<
        worth_runtime_bridge::facade::TruthWritebackReceipt,
        worth_runtime_bridge::facade::TruthWritebackAuthorityError,
    > {
        Ok(worth_runtime_bridge::facade::TruthWritebackReceipt::new(
            worth_runtime_bridge::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}

pub(in crate::runtime::tests) fn test_bridge() -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(TestBridgeSource)
        .with_signal_sink(TestBridgeSink)
        .with_writeback_authority(StaticWritebackAuthority)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("external-test"),
            TruthPatchScope::for_entity_field(
                MappingSelector::any(),
                aspect_key("aspect"),
                field_key("field"),
            ),
            SnapshotReadContract::scalar(aspect_key("aspect"), ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("external-test"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("test bridge should build")
}

pub(crate) fn test_bridge_with_writeback_authority() -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(TestBridgeSource)
        .with_signal_sink(TestBridgeSink)
        .with_writeback_authority(StaticWritebackAuthority)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("external-test"),
            TruthPatchScope::for_entity_field(
                MappingSelector::any(),
                aspect_key("aspect"),
                field_key("field"),
            ),
            SnapshotReadContract::scalar(aspect_key("aspect"), ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("external-test"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("test bridge with writeback authority should build")
}

fn native_patch_envelope(
    commit_identity: TruthCommitIdentity,
    entity_identity: &str,
    aspect: &str,
    field: &str,
) -> BridgeCommittedPatchEnvelope {
    let commit_id = commit_identity
        .relational_commit_id()
        .expect("runtime bridge fixture commit identity must retain relational commit payload");
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new(
            commit_identity,
            TruthPatchIdentity::from_relational_patch_position(commit_id),
            TruthSnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(1, commit_id),
            ),
            TruthBranchIdentity::from_relational_branch_id("main"),
        ),
        vec![BridgeCommittedPatchItem::with_target(
            entity_identity,
            BridgeCommittedPatchTarget::entity_field_path(
                AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(aspect)),
                CanonicalFieldPath::single(field_key(field)),
            ),
        )],
    )
    .expect("runtime test fixture must build a native patch envelope")
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid runtime bridge test aspect key")
}

fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value.to_owned()).expect("valid runtime bridge test field key")
}
