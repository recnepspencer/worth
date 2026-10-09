use worth_foundational::facade::{
    AspectKey, AspectLocator, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
    ScalarAspectType,
};
use worth_relational::facade::commit_strategies::{
    CommitStrategyId, CommitStrategyRegistration, IntentReconciliationStrategy,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch,
};
use worth_relational::facade::{identity::KindId, identity::PartitionId, symbols::ClientKey};
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity,
    BridgeCommittedPatchItem, BridgeCommittedPatchTarget, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, CoarseRoutingMode, CommittedPatchSource, InvalidationSink,
    MappingSelector, RelationalBridgeSnapshotIdentityParts, RelationalBridgeSourceError,
    RelationalCommittedPatchRequest, RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult,
    SnapshotReadRecord, SnapshotReadSource, TruthBranchIdentity, TruthCommitIdentity,
    TruthPatchIdentity, TruthPatchScope, TruthPatchTargetSelector, TruthSnapshotIdentity,
    TruthSnapshotReader, TruthWritebackAuthority, TruthWritebackAuthorityError,
    TruthWritebackReceipt, TruthWritebackRequest,
};

use crate::aspect_field_authoring::{
    entity_string_field_aspect, lifecycle_string_aspect, single_native_string_aspect_field_patch,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;

pub(super) fn relational_runtime_with_intent_strategy() -> RelationalRuntime {
    let descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(211));
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .commit_strategy(
            CommitStrategyRegistration::new(descriptor.clone()).expect("strategy registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build()
}

pub(super) fn create_entity(
    runtime: &mut RelationalRuntime,
    name: &str,
    branch: BranchId,
) -> worth_relational::facade::identity::EntityId {
    let identity = runtime
        .branch_identity(&branch)
        .expect("seed branch identity remains owner-issued");
    let options = runtime
        .admit_branch_basis(&identity)
        .expect("seed branch remains owner-admissible");
    let mut txn = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    txn.push_batch(
        WorkerIntentBatch::new(format!("create-{name}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(name),
                fields: single_native_string_aspect_field_patch("name", "name", name)
                    .expect("seed name aspect patch"),
            }),
        )),
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("seeded effect fixture staging fits the configured transaction budget");
    let outcome = txn
        .commit(
            runtime,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("seed commit should succeed");
    let entity = outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(entity_id) => Some(*entity_id),
            RecordRef::Relation(_) => None,
        })
        .expect("seed commit should touch one entity");
    runtime
        .snapshots()
        .release_snapshot(&outcome.snapshot)
        .expect("seed fixture releases its exact published snapshot");
    entity
}

pub(super) fn fork_seed_branch_from_main(runtime: &mut RelationalRuntime, branch: &str) {
    crate::runtime::fork_branch_from_exact_source(
        runtime,
        BranchId(branch.to_string()),
        &BranchId("main".to_string()),
    )
    .expect("seed branch should fork from the owner-admitted main basis");
}

pub(super) fn test_bridge_with_writeback_authority() -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(TestBridgeSource)
        .with_signal_sink(TestBridgeSink)
        .with_writeback_authority(StaticWritebackAuthority)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("external-test"),
            TruthPatchScope::new(
                MappingSelector::any(),
                AspectKeySelector::exact(aspect_key("aspect")),
                TruthPatchTargetSelector::any(),
            ),
            SnapshotReadContract::scalar(aspect_key("aspect"), ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("external-test"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("test bridge with writeback authority should build")
}

pub(super) fn branch_snapshot_identity(
    runtime: &RelationalRuntime,
    branch: &str,
) -> WorthQuerySnapshotIdentity {
    crate::memory_workspace::snapshot_identity_from_branch(runtime, &BranchId(branch.to_string()))
        .expect("certification branch snapshot requires a current owner basis")
        .expect("certification branch requires a current head")
}

fn test_schema_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_string_field_aspect("name", "name").expect("name aspect"),
                lifecycle_string_aspect("lifecycle").expect("lifecycle aspect"),
            ]),
        })
        .expect("test entity kind should register")
}

#[derive(Clone, Debug)]
struct TestBridgeSource;

impl CommittedPatchSource for TestBridgeSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(native_patch_envelope(
            request.commit_identity().clone(),
            TruthSnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(2_000, 1),
            ),
            TruthBranchIdentity::from_relational_branch_id("main"),
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

#[derive(Clone, Debug)]
struct TestSnapshotReader {
    identity: TruthSnapshotIdentity,
}

impl TruthSnapshotReader for TestSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.identity.clone()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, worth_runtime_bridge::facade::BridgeSnapshotReadError>
    {
        Ok(SnapshotReadPacketResult::new(
            self.identity.clone(),
            request
                .reads()
                .iter()
                .map(|read| SnapshotReadRecord::for_request(read, AspectValue::Null))
                .collect(),
        ))
    }
}

#[derive(Clone, Debug)]
struct TestBridgeSink;

impl InvalidationSink for TestBridgeSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

#[derive(Clone, Debug)]
struct StaticWritebackAuthority;

impl TruthWritebackAuthority for StaticWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError> {
        Ok(TruthWritebackReceipt::new(
            worth_runtime_bridge::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}

fn native_patch_envelope(
    commit_identity: TruthCommitIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    branch_identity: TruthBranchIdentity,
    entity_identity: &str,
    aspect: &str,
    field: &str,
) -> BridgeCommittedPatchEnvelope {
    let patch_identity = TruthPatchIdentity::from_relational_patch_position(
        commit_identity.relational_commit_id().unwrap_or_else(|| {
            stable_fixture_position(
                "seeded-effect-patch",
                commit_identity
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
        }),
    );
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new(
            commit_identity,
            patch_identity,
            snapshot_identity,
            branch_identity,
        ),
        vec![BridgeCommittedPatchItem::with_target(
            entity_identity,
            BridgeCommittedPatchTarget::entity_field_path(
                AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(aspect)),
                CanonicalFieldPath::single(field_key(field)),
            ),
        )],
    )
    .expect("seeded effect fixture must build a native patch envelope")
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid seeded effect bridge aspect key")
}

fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value.to_owned()).expect("valid seeded effect bridge field key")
}

fn stable_fixture_position(namespace: &str, evidence: &str) -> u64 {
    let mut acc = 14_695_981_039_346_656_037_u64;
    for byte in namespace.bytes().chain([0]).chain(evidence.bytes()) {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1_099_511_628_211);
    }
    acc
}
