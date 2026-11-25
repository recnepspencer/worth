//! Untrusted locator swaps must not reinterpret another intact canonical member.
use super::selected_source::SelectedSource;
use super::*;
use worth_relational::facade::{
    publication::PatchStreamPosition,
    storage::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    transactions::RelationalExecutionDenialCause,
};
use RelationalCanonicalCommitArtifactEncodeDenial as ExportDenied;

#[test]
fn selected_original_export_binds_its_indexed_subject_and_preserves_proven_damage() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (runtime, source, position, foreign) = crossed_runtime(execution);
    let receipt = runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(position)
        .unwrap();
    let frontier = runtime.history().latest_patch_stream_position();
    let offered = source.medium.offered().len();
    let export = || {
        runtime
            .durability()
            .canonical_commit_artifacts(execution)
            .next()
            .unwrap()
    };
    let failure = RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Envelope(position),
        cause: Cause::IntegrityMismatch,
    };
    assert_eq!(
        export().unwrap_err(),
        ExportDenied::StorageUnavailable(failure)
    );
    assert!(source.canonical_reads.lock().unwrap().contains(&foreign));
    let reads = source.canonical_reads.lock().unwrap().len();
    assert_eq!(
        runtime
            .history()
            .historical_committed_version(execution, receipt.version_id)
            .unwrap_err(),
        RelationalExecutionDenialCause::StorageUnavailable(failure)
    );
    assert_eq!(
        export().unwrap_err(),
        ExportDenied::StorageUnavailable(failure)
    );
    assert_eq!(source.canonical_reads.lock().unwrap().len(), reads);
    assert!(source.journal_reads.lock().unwrap().is_empty());
    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    assert_eq!(source.medium.offered().len(), offered);
}

fn crossed_runtime(
    execution: &ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    Arc<SelectedSource>,
    PatchStreamPosition,
    RelationalDurableMemberLocator,
) {
    let (original, medium) = initialized(execution);
    for key in 0..6 {
        let commit = create_entity_outcome(execution, &original, &format!("selected-{key}"));
        release_test_commit_snapshot(&original, &commit);
    }
    let image = capture_external(&original, &medium, execution);
    medium.file_checkpoint(image.checkpoint().clone());
    let bytes = image.checkpoint().bytes();
    let payload = bytes.strip_prefix(b"WORTH-REL-CHK23\n").unwrap();
    let rows = crate::messagepack_value_spans::sequence(crate::messagepack_value_spans::field(
        crate::messagepack_value_spans::field(payload, "manifest"),
        "canonical_members",
    ));
    let position =
        rmp_serde::from_slice(crate::messagepack_value_spans::field(rows[0], "position")).unwrap();
    let first = crate::messagepack_value_spans::field(rows[0], "member");
    let second = crate::messagepack_value_spans::field(rows[1], "member");
    let foreign = rmp_serde::from_slice(second).unwrap();
    let a = first.as_ptr() as usize - bytes.as_ptr() as usize;
    let b = second.as_ptr() as usize - bytes.as_ptr() as usize;
    assert!(a + first.len() <= b);
    let mut changed = bytes[..a].to_vec();
    changed.extend_from_slice(second);
    changed.extend_from_slice(&bytes[a + first.len()..b]);
    changed.extend_from_slice(first);
    changed.extend_from_slice(&bytes[b + second.len()..]);
    let source = SelectedSource::new(medium.clone());
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            image.source_artifact().covered(),
            image.source_artifact().covered_transitions(),
            RelationalNativeCheckpoint::from_untrusted_bytes(changed),
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut runtime = receiver(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    (runtime, source, position, foreign)
}

#[test]
fn complete_capture_stopped_request_precedes_proven_selected_damage() {
    use worth_execution::{ExecutionWorkCeiling, MapKernelStop, WorkCeilingDenial};
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (runtime, source, _, _) = crossed_runtime(execution);
    let first = runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .next()
        .unwrap();
    assert!(
        matches!(first, Err(ExportDenied::StorageUnavailable(failure)) if failure.cause == Cause::IntegrityMismatch)
    );
    let reads = source.canonical_reads.lock().unwrap().len();
    let (native, _) = execution
        .run(ExecutionWorkCeiling::new(Some(0)), |_| {
            execution.run_native_with_checkpoint(|context| {
                assert_eq!(context.checkpoint(1), Err(MapKernelStop::WorkCeiling));
                let stopped = execution.consult().unwrap_err();
                assert_eq!(
                    stopped,
                    WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)
                );
                match runtime.complete_checkpoint_service().prepare(
                    execution,
                    RelationalCheckpointArtifactPolicy::ExternalMembers {
                        current_members: &[],
                        source: None,
                    },
                ) {
                    Err(RelationalNativeCheckpointCaptureDenial::Request(cause)) => {
                        assert_eq!(cause, stopped.into())
                    }
                    other => panic!("the invoking work stop precedes retained damage: {other:?}"),
                }
            })
        })
        .unwrap();
    assert!(
        native.is_err(),
        "the same native owner retains the swallowed work stop"
    );
    assert_eq!(source.canonical_reads.lock().unwrap().len(), reads);
}

#[test]
fn complete_preserves_partial_update_without_demanding_untouched_aspect_reference() {
    use super::super::native_checkpoint_partitions::{
        original as capture_original, partition_image, physical_index,
    };
    use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
    use worth_foundational::facade::{AspectValue, ContractValidationInput, InternedString};
    use worth_relational::facade::{
        identity::KindId, schema::AspectBinding, storage::RecordLifecycleState, symbols::Symbol,
        transactions::*,
    };
    use worth_testkit_relational::{
        aspect_storage::publications::{begin_main_transaction, changed_entities},
        durable_publication::{field_patch, memory_runtime},
    };
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let plan = runtime
        .config()
        .schema
        .registry
        .entity_aspect_declaration_trace(KindId(1))
        .unwrap();
    assert!(plan.declarations.len() >= 2);
    let untouched = plan
        .declarations
        .iter()
        .find(|binding| matches!(binding.binding, AspectBinding::LifecycleTransition))
        .unwrap();
    assert_eq!(untouched.aspect_key.as_str(), "lifecycle");
    let created = create_entity_outcome(&execution, &runtime, "original-name");
    let entity = changed_entities(&created)[0];
    release_test_commit_snapshot(&runtime, &created);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: field_patch("name", "name", "updated-name"),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let updated = transaction.commit(&execution, &runtime).unwrap();
    let symbols = runtime
        .durability_authority()
        .checkpoint(&execution)
        .unwrap()
        .symbol_table;
    let symbol = |text| {
        symbols
            .entries
            .iter()
            .find(|(_, value)| value.as_str() == text)
            .map(|(id, _)| *id)
    };
    let untouched_symbol = symbol("lifecycle").expect("creation published real lifecycle meaning");
    let changed_symbol = symbol("name").unwrap();
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(&execution, updated.commit.commit_id)
        .unwrap()
        .unwrap();
    #[derive(serde::Deserialize)]
    struct RevisionReferences {
        revision_symbol_references: Vec<Symbol>,
    }
    let envelope: RevisionReferences =
        rmp_serde::from_slice(&rmp_serde::to_vec_named(&*envelope).unwrap()).unwrap();
    assert!(envelope
        .revision_symbol_references
        .binary_search(&changed_symbol)
        .is_ok());
    assert!(
        envelope
            .revision_symbol_references
            .binary_search(&untouched_symbol)
            .is_err(),
        "a name-only update has no lifecycle revision reference"
    );
    let identity = runtime.main_branch_identity();
    let original = runtime.admit_branch_basis(&identity).unwrap();
    let axes = original.reference().target().clone();
    let ordinary = capture_original(&execution, &runtime);
    let image = partition_image(&ordinary, updated.commit.commit_id, entity.partition_id);
    let physical = physical_index(&image.entity_arena.slots, entity.local_slot_value());
    let lifecycle = image.entity_arena.lifecycle[physical];
    assert_eq!(lifecycle, RecordLifecycleState::Live);
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(&execution, RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None })
        .unwrap();
    let image = partition_image(
        complete.checkpoint(),
        updated.commit.commit_id,
        entity.partition_id,
    );
    assert_eq!(image.entity_arena.lifecycle[physical], lifecycle);
    let state = image.entity_arena.extra[physical]
        .authoritative_aspect_state
        .as_ref()
        .unwrap();
    let name = state
        .entries()
        .iter()
        .find(|entry| entry.basis().key().as_str() == "name")
        .unwrap();
    let ContractValidationInput::Scalar(AspectValue::String(value)) = name.value() else {
        panic!("actual name value required")
    };
    let text = match value {
        InternedString::Raw(text) => text.as_str(),
        InternedString::Stored(text) => text.as_str(),
        InternedString::Symbol(symbol) => symbols
            .entries
            .iter()
            .find(|(id, _)| id.0 == symbol.0)
            .unwrap()
            .1
            .as_str(),
    };
    assert_eq!(text, "updated-name");
    let after = runtime.admit_branch_basis(&identity).unwrap();
    assert_eq!(after.reference().target(), &axes);
    assert!(after
        .custody_witness()
        .same_root(&original.custody_witness()));
    let unchanged = capture_original(&execution, &runtime);
    assert_eq!(
        partition_image(&unchanged, updated.commit.commit_id, entity.partition_id)
            .entity_arena
            .lifecycle[physical],
        lifecycle
    );
    release_test_commit_snapshot(&runtime, &updated);
}
