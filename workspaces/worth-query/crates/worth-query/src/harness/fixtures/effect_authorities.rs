use worth_relational::facade::commit_strategies::{
    CommitStrategyId, CommitStrategyRegistration, IntentReconciliationStrategy,
};
use worth_relational::facade::config::PublicationConfig;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::transactions::{
    CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, RecordRef,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_relational::facade::{identity::KindId, identity::PartitionId, symbols::ClientKey};
use worth_runtime_bridge::facade::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, BridgeSnapshotReadError, CoarseRoutingMode, CommittedPatchSource,
    ExecutionRequest, InvalidationSink, MappingSelector, RelationalBridgeSnapshotIdentityParts,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadPacket,
    SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadSource, TruthBranchIdentity,
    TruthPatchIdentity, TruthPatchScope, TruthSnapshotIdentity, TruthSnapshotReader,
    TruthWritebackAuthority, TruthWritebackAuthorityError, TruthWritebackReceipt,
    TruthWritebackRequest,
};

use crate::aspect_field_authoring::{
    entity_string_field_aspect, lifecycle_string_aspect, single_native_string_aspect_field_patch,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;

pub(crate) fn relational_runtime_with_intent_strategy() -> RelationalRuntime {
    let descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(211));
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8,
            max_active_snapshot_handles: 4_096,
            max_transaction_savepoints: 4_096,
            max_prepared_candidates: 1_024,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
        })
        .commit_strategy(
            CommitStrategyRegistration::new(descriptor.clone()).expect("strategy registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build()
}

pub(crate) fn create_entity(
    runtime: &mut RelationalRuntime,
    name: &str,
    branch: BranchId,
) -> worth_relational::facade::identity::EntityId {
    let mut txn = {
        let identity = runtime.branch_identity(&branch).expect("branch identity");
        let transaction_validation_input = runtime
            .admit_branch_basis(&identity)
            .expect("branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
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
    .expect("test staging stays within configured resource budgets");
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
        .expect("effect fixture releases its exact seed snapshot");
    entity
}

pub(crate) fn update_entity_name(
    runtime: &mut RelationalRuntime,
    entity_id: worth_relational::facade::identity::EntityId,
    name: &str,
    branch: BranchId,
) -> worth_relational::facade::history::CommitId {
    let mut txn = {
        let identity = runtime.branch_identity(&branch).expect("branch identity");
        let transaction_validation_input = runtime
            .admit_branch_basis(&identity)
            .expect("branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        WorkerIntentBatch::new(format!("update-{name}")).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: single_native_string_aspect_field_patch("name", "name", name)
                    .expect("update name aspect patch"),
            }),
        )),
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            runtime,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("intervening update should succeed");
    let commit_id = outcome.outcome().commit.commit_id;
    runtime
        .snapshots()
        .release_snapshot(&outcome.snapshot)
        .expect("effect fixture releases its exact update snapshot");
    commit_id
}

pub(crate) fn test_bridge() -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(TestBridgeSource)
        .with_signal_sink(TestBridgeSink)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("external-test"),
            TruthPatchScope::new(
                MappingSelector::any(),
                worth_runtime_bridge::facade::AspectKeySelector::exact(
                    worth_foundational::facade::AspectKey::new("aspect")
                        .expect("valid bridge mapping aspect key"),
                ),
                worth_runtime_bridge::facade::TruthPatchTargetSelector::any(),
            ),
            worth_runtime_bridge::facade::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("aspect")
                    .expect("valid bridge mapping aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
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
            TruthPatchScope::new(
                MappingSelector::any(),
                worth_runtime_bridge::facade::AspectKeySelector::exact(
                    worth_foundational::facade::AspectKey::new("aspect")
                        .expect("valid bridge mapping aspect key"),
                ),
                worth_runtime_bridge::facade::TruthPatchTargetSelector::any(),
            ),
            worth_runtime_bridge::facade::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("aspect")
                    .expect("valid bridge mapping aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("external-test"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("test bridge with writeback authority should build")
}

pub(crate) fn runtime_snapshot_identity(runtime: &RelationalRuntime) -> WorthQuerySnapshotIdentity {
    branch_snapshot_identity(runtime, "main")
}

pub(crate) fn branch_snapshot_identity(
    runtime: &RelationalRuntime,
    branch: &str,
) -> WorthQuerySnapshotIdentity {
    crate::memory_workspace::snapshot_identity_from_branch(runtime, &BranchId(branch.to_string()))
        .expect("branch snapshot fixture requires a current owner basis")
        .expect("branch snapshot fixture requires a current head")
}

pub(crate) fn exact_branch_head_commit_id(
    runtime: &RelationalRuntime,
    branch: &str,
) -> worth_relational::facade::history::CommitId {
    let branch_id = BranchId(branch.to_string());
    let identity = runtime
        .branch_identity(&branch_id)
        .expect("branch-head fixture requires a registered branch");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("branch-head fixture requires an owner-admitted basis");
    let observation = basis.observation();
    runtime
        .history()
        .branch_head_for_observation(&observation)
        .expect("branch-head fixture requires a local observation")
        .expect("branch-head fixture requires a committed head")
        .commit_id
}

pub(crate) fn exact_branch_snapshot(
    runtime: &mut RelationalRuntime,
    branch: &str,
) -> worth_relational::facade::snapshots::SnapshotHandle {
    let branch_id = BranchId(branch.to_string());
    let identity = runtime
        .branch_identity(&branch_id)
        .expect("snapshot fixture requires a registered branch");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("snapshot fixture requires an owner-admitted basis");
    let observation = basis.observation();
    runtime
        .snapshots()
        .snapshot_for_observation(&observation)
        .expect("snapshot fixture requires an exact retained basis")
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
        Ok(BridgeCommittedPatchEnvelope::new(
            worth_runtime_bridge::facade::BridgeCommittedPatchEnvelopeIdentity::new(
                request.commit_identity().clone(),
                TruthPatchIdentity::from_relational_patch_position(
                    request
                        .commit_identity()
                        .relational_commit_id()
                        .unwrap_or_else(|| {
                            stable_fixture_position(
                                "effect-patch",
                                request
                                    .commit_identity()
                                    .bridge_admission_evidence()
                                    .terminal_projection_for_reporting(),
                            )
                        }),
                ),
                TruthSnapshotIdentity::from_relational_snapshot(
                    RelationalBridgeSnapshotIdentityParts::new(1, 1),
                ),
                TruthBranchIdentity::from_relational_branch_id("main"),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                "entity",
                worth_runtime_bridge::facade::BridgeCommittedPatchTarget::entity_field_path(
                    worth_foundational::facade::AspectLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        worth_foundational::facade::AspectKey::new("aspect")
                            .expect("valid native bridge patch aspect key"),
                    ),
                    worth_foundational::facade::CanonicalFieldPath::single(
                        worth_foundational::facade::FieldKey::new("field".to_owned())
                            .expect("valid native bridge patch field key"),
                    ),
                ),
            )],
        )
        .expect("native bridge patch envelope fixture must construct"))
    }
}

fn stable_fixture_position(namespace: &str, evidence: &str) -> u64 {
    let mut acc = 14_695_981_039_346_656_037_u64;
    for byte in namespace.bytes().chain([0]).chain(evidence.bytes()) {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1_099_511_628_211);
    }
    acc
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
        _execution: ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
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

#[derive(Clone, Debug)]
struct TestBridgeSink;

impl InvalidationSink for TestBridgeSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: ExecutionRequest<'_, '_>,
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
