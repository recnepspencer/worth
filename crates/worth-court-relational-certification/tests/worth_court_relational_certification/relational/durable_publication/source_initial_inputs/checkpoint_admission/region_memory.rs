//! The invoking recovery budget outlives reconstruction temporaries with actual roots.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction, durable_publication::field_patch,
};

#[test]
fn checkpoint_image_recovery_respects_memory_and_cancellation_and_releases_its_retained_budget() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let setup_request = &ExecutionRequest::serial(&setup);
    let (source, medium) = initialized(setup_request);
    let mut transaction = begin_main_transaction(&source);
    let mut batch = WorkerIntentBatch::new("four-region-original");
    for partition in 0..4 {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::new(partition),
            kind_id: KindId(1),
            client_key: format!("region-{partition}").into(),
            fields: field_patch("name", "name", &format!("region-{partition}")),
        })));
    }
    transaction
        .push_batch(batch, setup_request.allocation_policy())
        .unwrap();
    let committed = transaction.commit(setup_request, &source).unwrap();
    release_test_commit_snapshot(&source, &committed);
    let capture = capture_external(&source, &medium, setup_request);
    let artifact = capture
        .admission_image()
        .seal_home_cut(&cut(&capture), setup_request)
        .unwrap();
    let directory =
        crate::source_wire_value::read(root_projection::root_payload(artifact.as_bytes()));
    assert_eq!(
        directory["roots"][0][0]["regions"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    medium.file_checkpoint(capture.checkpoint().clone());
    let input = home_image_input(&medium, &capture, setup_request).unwrap();
    drop((capture, source));
    let reads = medium.canonical_member_reads();

    let denied_budget = SerialMemoryBudget::new(Some(0));
    let denied = SerialRequest::from_memory(
        denied_budget.clone(),
        CancellationSource::new().token(),
        None,
    );
    let denied_input = input.copy_home_input_for_request(setup_request).unwrap();
    let cause = recover(&medium, denied_input, &ExecutionRequest::serial(&denied)).unwrap_err();
    assert!(
        matches!(
            cause,
            RelationalRecoveryStop::Request(
                RelationalExecutionDenialCause::PolicyMemoryExhausted { admitted: 0, .. }
            )
        ),
        "{cause:?}"
    );
    assert_eq!(denied_budget.charged_memory_bytes(), 0);
    assert_eq!(medium.canonical_member_reads(), reads);

    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let cancelled_budget = SerialMemoryBudget::new(None);
    let cancelled =
        SerialRequest::from_memory(cancelled_budget.clone(), cancellation.token(), None);
    let cancelled_input = input.copy_home_input_for_request(setup_request).unwrap();
    assert!(matches!(
        recover(
            &medium,
            cancelled_input,
            &ExecutionRequest::serial(&cancelled)
        ),
        Err(RelationalRecoveryStop::Request(
            RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert_eq!(cancelled_budget.charged_memory_bytes(), 0);
    assert_eq!(medium.canonical_member_reads(), reads);

    let budget = SerialMemoryBudget::new(None);
    let caller =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let retry = input.copy_home_input_for_request(setup_request).unwrap();
    let recovered = recover(&medium, input, &ExecutionRequest::serial(&caller)).unwrap();
    drop(caller);
    let (observation, basis) = recovered
        .observe_branch(&recovered.main_branch_identity())
        .unwrap();
    assert!(
        budget.charged_memory_bytes() > 0,
        "the actual root retains the invoking admission after temporaries and request drop"
    );
    assert!(matches!(
        observation.reference().target(),
        worth_foundational::facade::FoundationalBranchTarget::Basis(_)
    ));
    assert_eq!(medium.canonical_member_reads(), reads);
    let retained = budget.charged_memory_bytes();
    drop(basis);
    drop(recovered);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let mut denied = 0;
    for limit in [retained / 2, retained.saturating_sub(1), retained] {
        let input = retry.copy_home_input_for_request(setup_request).unwrap();
        let budget = SerialMemoryBudget::new(Some(limit));
        let caller =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        match recover(&medium, input, &ExecutionRequest::serial(&caller)) {
            Ok(runtime) => drop(runtime),
            Err(RelationalRecoveryStop::Request(
                RelationalExecutionDenialCause::PolicyMemoryExhausted { .. },
            ))
            | Err(RelationalRecoveryStop::AdmissionImage(Refusal::Request(
                RelationalExecutionDenialCause::PolicyMemoryExhausted { .. },
            ))) => denied += 1,
            Err(cause) => {
                panic!("a bounded recovery request cannot become checkpoint corruption: {cause:?}")
            }
        }
        assert_eq!(budget.charged_memory_bytes(), 0);
        assert_eq!(medium.canonical_member_reads(), reads);
    }
    assert!(
        denied > 0,
        "a budget below the actual retained footprint must deny"
    );
}

fn recover(
    medium: &Arc<TestDurableMedium>,
    input: RelationalDurableRecoverySource,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalRuntime, RelationalRecoveryStop> {
    let mut runtime = receiver(medium);
    let plan = runtime.durability().recovery_plan_from_source(
        request,
        input,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )?;
    let (_, authority) = runtime
        .durability_recovery()
        .recover_with_authority(request, plan)?;
    drop(authority);
    Ok(runtime)
}

#[test]
fn head_materialization_budget_survives_edit_and_last_basis_drop() {
    materialized_payload_survives_edit(false);
}
#[test]
fn full_materialization_budget_survives_mirror_edits_until_runtime_drop() {
    materialized_payload_survives_edit(true);
}

fn materialized_payload_survives_edit(full: bool) {
    use worth_relational::facade::mvcc::RelationalTransactionReadDenial;
    const PAYLOAD: usize = 32 * 1024;
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    let (source, medium) = initialized(request);
    let payload = create_entity_outcome(request, &source, &"p".repeat(PAYLOAD));
    let retained_entity = payload.semantic_result().created_entities()[0].assigned;
    release_test_commit_snapshot(&source, &payload);
    let editable = create_entity_outcome(request, &source, "small-editable");
    let editable_entity = editable.semantic_result().created_entities()[0].assigned;
    release_test_commit_snapshot(&source, &editable);
    drop((payload, editable));
    let image = capture_external(&source, &medium, request);
    let members = selected_members(&image, request);
    medium.file_checkpoint(image.checkpoint().clone());
    let input = home_image_input(&medium, &image, request).unwrap();
    drop((image, source));
    let recovered = recover(&medium, input, request).unwrap();
    let edit = |name: &str| {
        let mut transaction = begin_main_transaction(&recovered);
        transaction
            .push_batch(
                WorkerIntentBatch::new("edit-other-record").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: editable_entity,
                        fields: field_patch("name", "name", name),
                    }),
                )),
                request.allocation_policy(),
            )
            .unwrap();
        let result = transaction.commit(request, &recovered).unwrap();
        release_test_commit_snapshot(&recovered, &result);
    };
    let budget = SerialMemoryBudget::new(None);
    let caller =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let materialization = &ExecutionRequest::serial(&caller);
    if full {
        // The public retention owner materializes and promotes its actual
        // current mirror Full before extracting and freezing its partition.
        edit("before-full-materialization");
        let denied_budget = SerialMemoryBudget::new(Some(0));
        let denied = SerialRequest::from_memory(
            denied_budget.clone(),
            CancellationSource::new().token(),
            None,
        );
        let stopped = recovered
            .retention()
            .try_run_pass(&ExecutionRequest::serial(&denied))
            .unwrap_err();
        assert!(matches!(
            stopped.stop(),
            worth_relational::facade::runtime::RelationalRetentionPassStop::Request(
                RelationalExecutionDenialCause::PolicyMemoryExhausted { admitted: 0, .. }
            )
        ));
        assert_eq!(denied_budget.charged_memory_bytes(), 0);
        let progress = recovered.retention().try_run_pass(materialization).unwrap();
        assert!(progress.slots_scanned() >= 2);
    } else {
        let denied_budget = SerialMemoryBudget::new(Some(0));
        let denied = SerialRequest::from_memory(
            denied_budget.clone(),
            CancellationSource::new().token(),
            None,
        );
        let mut transaction = begin_main_transaction(&recovered);
        assert!(matches!(
            transaction.read_entity(&ExecutionRequest::serial(&denied), retained_entity),
            Err(RelationalTransactionReadDenial::Request(_))
        ));
        drop(transaction);
        assert_eq!(denied_budget.charged_memory_bytes(), 0);
        let mut transaction = begin_main_transaction(&recovered);
        assert!(transaction
            .read_entity(materialization, retained_entity)
            .unwrap()
            .base()
            .is_some());
        drop(transaction);
    }
    assert!(
        budget.charged_memory_bytes() >= PAYLOAD as u64,
        "materialization must retain its actual payload backing"
    );
    drop(caller);
    // This commit extracts the resident partition into an edit and freezes a
    // new region. The large, untouched record must keep its original tickets.
    edit("after-materialization");
    if full {
        // Full maintenance edits the mirror; a branch basis may independently
        // retain Head. Observe the actual mirror owner rather than that basis.
        assert!(
            budget.charged_memory_bytes() >= PAYLOAD as u64,
            "the edited mirror must retain its materialized Full payload"
        );
        drop((recovered, members, medium));
    } else {
        let (_, basis) = recovered
            .observe_branch(&recovered.main_branch_identity())
            .unwrap();
        drop((recovered, members, medium));
        assert!(
            budget.charged_memory_bytes() >= PAYLOAD as u64,
            "the surviving edited basis must retain the original native payload ticket"
        );
        drop(basis);
    }
    assert_eq!(
        budget.charged_memory_bytes(),
        0,
        "the last actual owner must release every materialization ticket"
    );
}
