use super::*;

#[test]
fn submitted_write_receipt_carries_typed_committed_journal_position() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.journal-position.single")
        .expect("task runtime should open a named workspace");
    let command = WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("identity.id"),
            test_authored_string_aspect_value("task-1"),
        )
        .set_aspect(
            test_aspect_touch("title.value"),
            test_authored_string_aspect_value("First"),
        )
        .build_insert("Task")
        .expect("insert command should build");
    let receipt = workspace
        .submissions()
        .expect("submission lane should be admitted")
        .submit(request_execution, command)
        .expect("submission should commit");
    let position = receipt.journal_position();

    assert_eq!(
        position.authority(),
        WorthQueryJournalPositionAuthority::Committed
    );
    assert_eq!(position.ordinal_for_reporting(), 1);
    assert_ne!(
        position.evidence_identity(),
        *receipt.commit_evidence_identity(),
        "journal position identity must not collapse into commit evidence identity"
    );
}

#[test]
fn submitted_schedule_records_monotonic_positions_and_stable_replay() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first_run =
        submitted_schedule_positions(request_execution, "tasks.journal-position.replay-a");
    let second_run =
        submitted_schedule_positions(request_execution, "tasks.journal-position.replay-b");

    assert_eq!(first_run.ordinals, vec![1, 2, 3]);
    assert_eq!(first_run.ordinals, second_run.ordinals);
    assert_eq!(
        first_run.evidence_identities,
        second_run.evidence_identities
    );
    assert_eq!(
        first_run.unique_identity_count(),
        first_run.evidence_identities.len()
    );
}

#[test]
fn journal_identity_certification_closes_only_with_real_inventory_and_schedule_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first_run =
        submitted_schedule_positions(request_execution, "tasks.journal-position.certification-a");
    let second_run =
        submitted_schedule_positions(request_execution, "tasks.journal-position.certification-b");
    let first_schedule = first_run.position_schedule();
    let second_schedule = second_run.position_schedule();
    let inventory = journal_identity_inventory_evidence();
    let schedule =
        WorthQueryJournalIdentityScheduleEvidence::derive(&first_schedule, &second_schedule);
    let replay =
        super::super::journal_identity_support::journal_replay_surface_evidence(request_execution);
    let certification = WorthQueryJournalIdentityCertification::from_evidence(
        inventory.clone(),
        schedule.clone(),
        replay.clone(),
    );

    assert!(certification.closed());
    assert!(!certification.artifact_digest().is_empty());
    assert!(!certification.failure_digest().is_empty());
    super::super::journal_identity_support::assert_closed_replay_boundary_certification(
        &certification,
    );
    assert!(!WorthQueryJournalIdentityCertification::from_evidence(
        inventory.with_forbidden_failure_for_sabotage(),
        schedule.clone(),
        replay.clone()
    )
    .closed());
    assert!(!WorthQueryJournalIdentityCertification::from_evidence(
        inventory.clone(),
        WorthQueryJournalIdentityScheduleEvidence::derive(
            &first_run.duplicate_first_position_schedule(),
            &second_schedule
        ),
        replay.clone()
    )
    .closed());
    assert!(!WorthQueryJournalIdentityCertification::from_evidence(
        inventory,
        schedule,
        replay.with_gap_for_sabotage()
    )
    .closed());
    let truth_mismatch = WorthQueryJournalIdentityCertification::from_evidence(
        journal_identity_inventory_evidence(),
        WorthQueryJournalIdentityScheduleEvidence::derive(&first_schedule, &second_schedule),
        super::super::journal_identity_support::journal_replay_surface_evidence(request_execution)
            .with_truth_mismatch_for_sabotage(),
    );
    assert_eq!(
        truth_mismatch.posture(),
        WorthQueryJournalIdentityBoundaryPosture::Partial
    );
}

#[test]
fn batch_receipt_carries_component_journal_positions_in_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (positions, inspection_identities) = submitted_batch_positions_with_inspection(
        request_execution,
        "tasks.journal-position.batch",
    );

    assert_eq!(positions.ordinals, vec![1, 2, 3]);
    assert_eq!(positions.unique_identity_count(), 3);
    assert_eq!(positions.evidence_identities, inspection_identities);
}

#[test]
fn preview_receipt_carries_preview_journal_position_without_commit_collision() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let (first_receipt, second_receipt) = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("journal-preview"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should admit");
        let first_receipt = preview
            .insert(request_execution, "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("preview-task"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Preview"),
                )
            })
            .expect("preview insert should stage");
        let second_receipt = preview
            .insert(request_execution, "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("preview-task-2"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Preview Two"),
                )
            })
            .expect("second preview insert should stage");
        (first_receipt, second_receipt)
    };

    assert_eq!(
        first_receipt.journal_position().authority(),
        WorthQueryJournalPositionAuthority::Preview
    );
    assert_eq!(first_receipt.journal_position().ordinal_for_reporting(), 1);
    assert_eq!(second_receipt.journal_position().ordinal_for_reporting(), 2);
    assert_ne!(
        first_receipt.journal_position().evidence_identity(),
        *first_receipt.commit_evidence_identity()
    );
    assert_ne!(
        first_receipt.journal_position().evidence_identity(),
        second_receipt.journal_position().evidence_identity()
    );
}
