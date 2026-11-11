//! Real Store journal producer, selected original members and uncovered tail.
use crate::cold_source::selected_replacement::read_expected;
use crate::cold_source::{test_store, StoreColdSource};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    durability::{RecoveryVerificationMode, RelationalDurableRecoverySource},
    history::BranchId,
    identity::{KindId, PartitionId},
    mvcc::{
        RelationalCallerAssociation, RelationalPublicationOutcome, RelationalTransactionIntent,
    },
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    transactions::{CreateIntent, EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch},
};
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublished,
    OrderedJournalRegistry, PhysicalMutationDeadline, PhysicalRecordOpen, ServingPhysicalRuntime,
    RELATIONAL_BRANCH_TRANSITIONS, RELATIONAL_CANONICAL_COMMITS,
};

use worth_store_testkit_store::physical_record_journeys as physical;

mod selection;
use selection::Selected;

// The declared real runtime factory serves this journey and its Complete owner child.
fn runtime(
    serving: &Arc<ServingPhysicalRuntime>,
    source: &Arc<StoreColdSource>,
    selected: &Arc<Selected>,
    schema: &RelationalSchemaRegistry,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(schema.clone())
        .build_durable(Box::new(crate::journal_medium::StoreJournalMedium::new(
            serving.clone(),
            worth_store_testkit_store::fixture::placement(),
            source.clone(),
            selected.clone(),
        )))
        .unwrap()
}
mod media;
use media::{corrupt_original, journal_frame};

#[test]
fn journal_checkpoint_selection_rehomes_covered_history_and_preserves_uncovered_tail() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    let source = StoreColdSource::new(serving.clone());
    let selected = Arc::new(Selected::default());
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = &ExecutionRequest::serial(&serial);
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "selected.original".into(),
            schema_id: SchemaId("selected.original".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let producer = runtime(&serving, &source, &selected, &schema);
    let create = |owner: &RelationalRuntime, key: &str| {
        let (_, basis) = owner.observe_branch(&owner.main_branch_identity()).unwrap();
        let mut transaction = owner
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
        transaction.commit(execution, owner).unwrap()
    };
    let first = create(&producer, &"original-extent-key-".repeat(8192));
    let RecordRef::Entity(entity) = first.changed_records[0] else {
        panic!("actual entity");
    };
    producer
        .snapshots()
        .release_snapshot(&first.snapshot)
        .unwrap();
    drop(first);
    let (_, before_basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    let before = producer
        .owner_component_services()
        .basis_port()
        .export_recorded_current_component(execution, &before_basis)
        .unwrap();
    let before_lease = producer.retain_component_basis(&before_basis).unwrap();
    drop(before_basis);
    let association =
        RelationalCallerAssociation::from_bytes(b"original-store-journal-associated-effect")
            .unwrap();
    let (_, basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    let mut transaction = producer
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("associated").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: "associated".into(),
                    fields: Default::default(),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    drop(basis);
    let proposal = transaction
        .validate(execution, &producer)
        .unwrap()
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = producer
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = producer
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("genuine original publication performs");
    };
    let after_basis = performed.next_basis().clone();
    let associated = producer.settle_performed_publication(performed).unwrap();
    // Snapshot every public field of the original pure meaning. No live result,
    // Native owner or physical read authority is carried across the restart.
    let semantics =
        |value: &worth_relational::facade::transactions::RelationalCommitSemanticResult| {
            (
                value.receipt().clone(),
                value.created_entities().to_vec(),
                value.created_relations().to_vec(),
                value.changed_records().to_vec(),
                value.structural().clone(),
                value.schema().clone(),
                value.history().clone(),
                value.change().clone(),
                value.aspects().clone(),
                value.publication().clone(),
                value.validation().to_vec(),
            )
        };
    let semantic = semantics(associated.semantic_result());
    producer
        .snapshots()
        .release_snapshot(&associated.snapshot)
        .unwrap();
    drop(associated);
    let after = producer
        .owner_component_services()
        .basis_port()
        .export_recorded_retained_component(execution, &after_basis)
        .unwrap();
    let after_lease = producer.retain_component_basis(&after_basis).unwrap();
    drop(after_basis);
    let head = create(&producer, "checkpoint-current");
    producer
        .snapshots()
        .release_snapshot(&head.snapshot)
        .unwrap();
    drop(head);
    let original = journal_frame(&serving, 1);
    assert!(original.touched_extents > 0);
    assert!(original.first_chunk.len() < original.bytes.len());
    let prepared = crate::section::checkpoint::prepare(&producer, &source, execution)
        .unwrap()
        .with_selection(selected.clone());
    let covered = prepared.covered();
    let transitions = prepared.covered_transitions();
    *selected.subject.lock().unwrap() = prepared.publication_subject().cloned();
    prepared.validate_publication().unwrap();
    let (members, member_memory) = prepared.prepare_members(&[], execution).unwrap();
    let absent =
        |bytes: &[u8], target: &[u8]| !bytes.windows(target.len()).any(|part| part == target);
    assert!(absent(prepared.bytes(), &original.bytes));
    assert!(absent(prepared.bytes(), &original.first_chunk));
    for member in &members {
        if let AnchoredRecordMember::New { bytes, .. } = member {
            assert!(absent(bytes, &original.bytes));
            assert!(absent(bytes, &original.first_chunk));
        }
    }
    let published = serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                AnchoredRecordName::new("home.checkpoint").unwrap(),
                0,
                prepared.bytes().to_vec(),
                worth_store_testkit_store::fixture::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(members),
        )
        .unwrap();
    let version = published.version();
    *selected.receipt.lock().unwrap() = Some(match published {
        AnchoredRecordPublished::Published(value)
        | AnchoredRecordPublished::AlreadyPublished(value) => value,
    });
    execution
        .run_native_with_checkpoint(|context| prepared.acknowledge_selected(execution, context))
        .unwrap()
        .0
        .unwrap();
    producer.release_component_basis(before_lease).unwrap();
    producer.release_component_basis(after_lease).unwrap();
    drop(prepared);
    drop(member_memory);
    *selected.subject.lock().unwrap() = None;
    *selected.receipt.lock().unwrap() = None;
    let tail = create(&producer, "uncovered-tail");
    producer
        .snapshots()
        .release_snapshot(&tail.snapshot)
        .unwrap();
    drop(tail);
    assert_eq!(
        producer.history().latest_patch_stream_position().unwrap().0,
        covered.0 + 1
    );
    let tail_frame = journal_frame(&serving, covered.0 + 1);
    assert!(absent(&tail_frame.bytes, &original.bytes));
    assert!(absent(&tail_frame.bytes, &original.first_chunk));
    drop(producer);
    drop(source);
    let closed = Arc::try_unwrap(serving).ok().unwrap().close();
    assert_eq!(
        closed.records().posture(),
        worth_store::physical_runtime::RecordServingTerminalPosture::NoInspectionRequired
    );
    drop(closed);
    // Extent payload chunks are separated by durable physical headers. Damage
    // an entire actual observed chunk, never a guessed logical-record prefix.
    let damaged = corrupt_original(directory.path(), &original.first_chunk);
    assert!(
        damaged > 0,
        "actual original physical payload chunk must be found; never skip zero matches"
    );
    let (format, _, access) = physical::configuration();
    let media = physical::media(directory.path());
    let registry = OrderedJournalRegistry::declare(
        3,
        &[RELATIONAL_CANONICAL_COMMITS, RELATIONAL_BRANCH_TRANSITIONS],
        &["home.checkpoint"],
    )
    .unwrap();
    let admission = PhysicalRecordOpen::new(format, access, physical::durability(&media))
        .with_journal_registry(registry);
    let serving = Arc::new(physical::success(media.open_record_store(admission)));
    let source = StoreColdSource::new(serving.clone());
    let read = read_expected(&source, version, execution).unwrap();
    source
        .select_initial_members_from_read(&read, execution)
        .unwrap();
    let record = serving
        .anchored_records()
        .unwrap()
        .read(version.name(), 1 << 30)
        .unwrap()
        .unwrap();
    let checkpoint = crate::section::codec::decode(record.bytes(), execution).unwrap();
    drop(record);
    drop(read);
    let mut recovered = runtime(&serving, &source, &selected, &schema);
    let recovery = RelationalDurableRecoverySource::new(
        Some(checkpoint),
        crate::recovered_history::journal_tail(&serving, covered.0).unwrap(),
        crate::recovered_transitions::transition_tail(&serving, transitions.0).unwrap(),
        source.clone(),
    );
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    let (_, current) = recovered
        .observe_branch(&recovered.main_branch_identity())
        .unwrap();
    let snapshot = recovered
        .snapshots()
        .snapshot_for_observation(&current.observation())
        .unwrap();
    assert_eq!(
        recovered
            .read_truth()
            .read_snapshot(&snapshot, execution)
            .unwrap()
            .unwrap()
            .entities()
            .len(),
        4
    );
    recovered.snapshots().release_snapshot(&snapshot).unwrap();
    drop(current);
    assert_eq!(
        recovered
            .history()
            .entity_aspect_history(execution, &BranchId("main".into()), entity, None)
            .unwrap()
            .len(),
        1
    );
    let subject = recovered
        .durability_authority()
        .discover_associated_effects(execution, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = recovered
        .durability_authority()
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    assert_eq!(semantics(effect.semantic_result()), semantic);
    let proof = authority
        .admit_recorded_publication(execution, &recovered, effect, &before, &after)
        .unwrap();
    for (basis, len) in [
        (proof.custody().predecessor(), 1),
        (proof.custody().successor(), 2),
    ] {
        let snapshot = recovered
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .unwrap()
                .unwrap()
                .entities()
                .len(),
            len
        );
        recovered.snapshots().release_snapshot(&snapshot).unwrap();
    }
    assert_eq!(
        source.canonical_journal_attempts(),
        0,
        "no covered canonical journal demand, including component/association admission"
    );
    assert!(source.counters().member_reads > 0);
    assert!(
        source.counters().commit_reads > 0,
        "selected canonical bodies were admitted"
    );
    assert_eq!(
        recovered
            .history()
            .latest_patch_stream_position()
            .unwrap()
            .0,
        covered.0 + 1
    );
    // Last: an actual journal read must now fail. Integrity failure can mark
    // physical inspection posture, so it cannot precede the positive reads.
    assert!(serving
        .canonical_commits()
        .unwrap()
        .read(1, 1 << 30)
        .is_err());
}

mod complete;
