//! Exact private-runtime exception: M3 home lifecycle, cold member prerequisite.
use super::*;
use crate::cold_source::StoreColdSource;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        OpaqueCapturedCheckpointSubject, RecoveryVerificationMode,
        RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
        RelationalNativeCheckpoint,
    },
    history::BranchId,
    identity::{EntityId, KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    transactions::{CreateIntent, EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch},
};
use worth_store::physical_runtime::{
    AnchoredRecordMembership, AnchoredRecordName, AnchoredRecordPublication,
    PhysicalMutationDeadline,
};

// This fixture does not claim Home acknowledgement. The provider publishes the
// actual section to its declared physical anchor; native recovery verifies it.
#[derive(Debug)]
struct NoAcknowledgement;
impl crate::RelationalCheckpointSelectionSource for NoAcknowledgement {
    fn selected_membership(
        &self,
        _: &OpaqueCapturedCheckpointSubject,
    ) -> Option<AnchoredRecordMembership> {
        None
    }
}

pub(super) struct Directory {
    pub(super) runtime: RelationalRuntime,
    pub(super) source: Arc<StoreColdSource>,
    pub(super) serving: Arc<ServingPhysicalRuntime>,
    pub(super) members: Vec<RelationalDurableMemberLocator>,
    pub(super) generation: u64,
    pub(super) entity: EntityId,
    recovery_reads: u64,
    pub(super) _directory: tempfile::TempDir,
}

pub(super) fn directory(execution: &ExecutionRequest<'_, '_>) -> Directory {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(crate::cold_source::test_store::initialize(directory.path()));
    let source = StoreColdSource::new(serving.clone());
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "retirement.entity".into(),
            schema_id: SchemaId("retirement".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let build = || {
        RelationalRuntimeApi::builder()
            .schema_registry(schema.clone())
            .build_durable(Box::new(crate::journal_medium::StoreJournalMedium::new(
                serving.clone(),
                worth_store_testkit_store::fixture::placement(),
                source.clone(),
                Arc::new(NoAcknowledgement),
            )))
            .unwrap()
    };
    let producer = build();
    let mut entity = None;
    for key in ["first-original", "second-original"] {
        let (_, basis) = producer
            .observe_branch(&producer.main_branch_identity())
            .unwrap();
        let mut transaction = producer
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                    EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: key.into(),
                        fields: Default::default(),
                    },
                ))),
                execution.allocation_policy(),
            )
            .unwrap();
        let result = transaction.commit(execution, &producer).unwrap();
        if entity.is_none() {
            let RecordRef::Entity(id) = result.changed_records[0] else {
                panic!("original entity")
            };
            entity = Some(id);
        }
        producer
            .snapshots()
            .release_snapshot(&result.snapshot)
            .unwrap();
    }
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(execution, execution.allocation_policy(), worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: Some(source.clone()) })
        .unwrap();
    let selected_native = RelationalDurableCheckpointSource::new(
        native.covered_position().unwrap(),
        native.covered_branch_transitions().unwrap(),
        RelationalNativeCheckpoint::from_untrusted_bytes(native.bytes().to_vec()),
    );
    let prepared =
        crate::section::checkpoint::prepare_captured(&source, native, execution).unwrap();
    let (members, _memory) = prepared.prepare_members(&[], execution).unwrap();
    let records = serving.anchored_records().unwrap();
    let anchor = AnchoredRecordName::new("home.checkpoint").unwrap();
    let published = records
        .publish(
            AnchoredRecordPublication::new(
                anchor,
                0,
                prepared.bytes().to_vec(),
                worth_store_testkit_store::fixture::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(members),
        )
        .unwrap();
    source.select_members(published.version().generation(), published.members());
    let read = records.read(anchor, 1 << 30).unwrap().unwrap();
    assert_eq!(read.bytes(), prepared.bytes());
    let generation = read.version().generation();
    drop(read);
    drop(prepared);
    drop(producer);
    let mut runtime = build();
    let covered = selected_native.covered().0;
    let transitions = selected_native.covered_transitions().0;
    let recovery = RelationalDurableRecoverySource::new(
        Some(selected_native),
        crate::recovered_history::journal_tail(&serving, covered).unwrap(),
        crate::recovered_transitions::transition_tail(&serving, transitions).unwrap(),
        source.clone(),
    );
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    let recovery_reads = source.counters().member_reads;
    let members = source.member_locators();
    assert!(
        members.len() > 1,
        "actual native closure permits partial retirement"
    );
    Directory {
        runtime,
        source,
        serving,
        members,
        generation,
        entity: entity.unwrap(),
        recovery_reads,
        _directory: directory,
    }
}

pub(super) fn assert_cold_reads(fixture: &Directory, execution: &ExecutionRequest<'_, '_>) {
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let snapshot = fixture
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let read = fixture
        .runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    assert_eq!(read.entities().len(), 2);
    let history = fixture
        .runtime
        .history()
        .entity_aspect_history(execution, &BranchId("main".into()), fixture.entity, None)
        .unwrap();
    assert_eq!(history.len(), 1, "actual original create history survives");
    fixture
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
    assert!(
        fixture.source.counters().member_reads > fixture.recovery_reads,
        "the recovered owner actually demanded cold member bytes"
    );
    let selected = fixture
        .serving
        .anchored_records()
        .unwrap()
        .read(AnchoredRecordName::new("home.checkpoint").unwrap(), 1 << 30)
        .unwrap()
        .unwrap();
    assert_eq!(
        selected.version().generation(),
        fixture.generation,
        "release never republishes"
    );
}
