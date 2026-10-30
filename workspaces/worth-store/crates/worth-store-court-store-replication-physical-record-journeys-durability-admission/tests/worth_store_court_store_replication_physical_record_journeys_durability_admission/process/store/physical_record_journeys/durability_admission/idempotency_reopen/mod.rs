use {
    crate::durability_admission::idempotency_reopen::support::append_one,
    crate::durability_admission::idempotency_reopen::support::inspect_checkpoint_reopen,
    crate::durability_admission::idempotency_reopen::support::prepare,
    crate::durability_admission::idempotency_reopen::support::request,
    crate::durability_admission::idempotency_reopen::support::success_checkpoint,
    crate::durability_admission::idempotency_reopen::support::synchronize,
};

use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_with_idempotency_limits,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn fresh_process_rebuild_joins_checkpoint_compaction_with_the_retained_wal_suffix() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();

    let target_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([181; 32]),
        )
        .unwrap();
    let target = prepare(&submission, placement, target_key.clone(), b"tail-target");
    let target_mutation = target.mutation_identity();

    let anchor = worth_store_testkit_store::physical_record_journeys::durability_admission::wal_append::prepared(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([182; 32]),
        b"checkpoint-anchor",
    );
    let appended = append_one(&submission, anchor);
    synchronize(&submission, appended);

    let checkpoint = success_checkpoint(&serving, 183);
    assert_eq!(checkpoint.binding_compaction().binding_count(), 2);
    assert_eq!(checkpoint.binding_compaction().generation().get(), 1);

    let appended = append_one(&submission, target);
    synchronize(&submission, appended);
    let expected_checkpoint = inspect_checkpoint_reopen(&root);
    let expected_wal = worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory(&root).unwrap();
    assert_eq!(expected_wal.frame_count(), 2);
    assert_eq!(expected_wal.lsn_range(), Some((1, 3)));
    serving.close();

    let reopened = serving_from_open(&root);
    let reopen = reopened
        .durability_observation()
        .reopen()
        .expect("serving authority exists only after durability state reopens");
    assert_eq!(
        reopen.checkpoint_artifact_bytes(),
        expected_checkpoint.artifact_bytes
    );
    assert_eq!(
        reopen.checkpoint_bytes_read(),
        expected_checkpoint.bytes_read
    );
    assert_eq!(
        reopen.dirty_body_bytes_skipped(),
        expected_checkpoint.dirty_bytes
    );
    assert_eq!(
        reopen.binding_records_read(),
        expected_checkpoint.binding_records
    );
    assert_eq!(
        reopen.checkpoint_integrity_admissions(),
        expected_checkpoint.binding_records + expected_checkpoint.certificate_records + 3
    );
    assert_eq!(reopen.wal_members_read(), 1);

    let reopened_submission = reopened.certification_record_submission();
    assert_eq!(
        reopened_submission
            .wal_observation()
            .unwrap()
            .reopen_peak_buffer_bytes(),
        expected_wal.peak_segment_bytes()
    );
    let before_retry = reopened.media_counters();
    let duplicate = prepare(
        &reopened_submission,
        placement,
        target_key.clone(),
        b"tail-target",
    );
    assert_eq!(
        duplicate.disposition(),
        worth_store::physical_runtime::PhysicalMutationAdmissionDisposition::DuplicateUnresolved
    );
    assert_eq!(duplicate.mutation_identity(), target_mutation);
    assert_eq!(reopened.media_counters(), before_retry);

    assert!(matches!(
        reopened_submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                    b"conflicting-tail".as_slice()
                ])
                .unwrap(),
                placement,
                request(target_key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::IdempotencyConflict
        )
    ));
    reopened.close();
}

#[test]
fn terminal_fate_reopens_from_namespace_durable_compaction() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let terminal_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([186; 32]),
        )
        .unwrap();
    let terminal = prepare(
        &submission,
        placement,
        terminal_key.clone(),
        b"terminal-fate",
    );
    let terminal_mutation = terminal.mutation_identity();
    assert!(matches!(
        submission.cancel_prepared_before_group_seal(terminal),
        worth_store::physical_runtime::PhysicalPreSealCancellationOutcome::ProvenNoEffect(_)
    ));
    let anchor = worth_store_testkit_store::physical_record_journeys::durability_admission::wal_append::prepared(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([187; 32]),
        b"retained-wal-anchor",
    );
    let appended = append_one(&submission, anchor);
    synchronize(&submission, appended);
    let first = success_checkpoint(&serving, 188);
    assert_eq!(first.binding_compaction().generation().get(), 1);
    assert_eq!(first.binding_compaction().terminal_binding_count(), 1);
    serving.close();

    let reopened = serving_from_open(&root);
    let submission = reopened.certification_record_submission();
    let replay = submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"terminal-fate".as_slice()
            ])
            .unwrap(),
            placement,
            request(terminal_key),
        )
        .into_raw();
    match replay {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::ProvenNoEffect(fate),
        ) => {
            assert_eq!(fate.mutation_identity(), terminal_mutation);
        }
        _ => panic!("fresh process must replay the exact persisted terminal fate"),
    }
    reopened.close();
}

#[test]
fn completed_fate_reopens_with_exact_acknowledgment_basis() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([189; 32]),
        )
        .unwrap();
    let prepared = prepare(&submission, placement, key.clone(), b"completed-terminal");
    let completed = match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) => completed,
        _ => panic!("managed mutation must complete before compaction"),
    };
    let expected_identity = completed.mutation_identity();
    let expected_breadth = completed.completed_breadth();
    let expected_records = completed.persisted_records().to_vec();

    let checkpoint = success_checkpoint(&serving, 190);
    assert_eq!(checkpoint.binding_compaction().terminal_binding_count(), 1);
    serving.close();

    let reopened = serving_from_open(&root);
    let replay = reopened
        .certification_record_submission()
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"completed-terminal".as_slice(),
            ])
            .unwrap(),
            placement,
            request(key),
        )
        .into_raw();
    match replay {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Completed(completed),
        ) => {
            assert_eq!(completed.mutation_identity(), expected_identity);
            assert_eq!(completed.completed_breadth(), expected_breadth);
            assert_eq!(completed.persisted_records(), expected_records);
            assert_eq!(
                completed.into_acknowledgment().mutation_identity(),
                expected_identity
            );
        }
        _ => panic!("fresh process must reopen exact completed terminal fate"),
    }
    reopened.close();
}

#[test]
fn indeterminate_fate_compacts_and_reopen_blocks_ordinary_retry() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([191; 32]),
        )
        .unwrap();
    let prepared = prepare(
        &submission,
        placement,
        key.clone(),
        b"indeterminate-terminal",
    );
    let expected_identity = prepared.mutation_identity();
    serving.certification_reject_next_candidate_publication_after_physical_write();
    let fate = match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => fate,
        _ => panic!("candidate publication rejection must be indeterminate"),
    };
    assert_eq!(fate.mutation_identity(), expected_identity);
    assert_eq!(
        fate.diagnostic_evidence().mutation_identity(),
        expected_identity
    );
    assert_eq!(
        fate.stage(),
        worth_store::physical_runtime::PhysicalMutationIndeterminateStage::DataDispatch
    );
    assert_eq!(fate.completed_effect_count(), 1);

    let checkpoint = success_checkpoint(&serving, 192);
    assert_eq!(checkpoint.binding_compaction().terminal_binding_count(), 1);
    serving.close();

    let reopened = serving_from_open(&root);
    let replay = reopened
        .certification_record_submission()
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"indeterminate-terminal".as_slice(),
            ])
            .unwrap(),
            placement,
            request(key),
        )
        .into_raw();
    assert!(matches!(
        replay,
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                worth_store::physical_runtime::RecordAppendDenial::ServingRequiresInspection
            )
        )
    ));
    reopened.close();
}

#[test]
fn expired_terminal_fate_is_omitted_only_by_a_subsequent_durable_compaction() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let terminal_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([192; 32]),
        )
        .unwrap();
    let terminal = prepare(
        &submission,
        placement,
        terminal_key.clone(),
        b"expiring-terminal",
    );
    assert!(matches!(
        submission.cancel_prepared_before_group_seal(terminal),
        worth_store::physical_runtime::PhysicalPreSealCancellationOutcome::ProvenNoEffect(_)
    ));
    let anchor = worth_store_testkit_store::physical_record_journeys::durability_admission::wal_append::prepared(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([193; 32]),
        b"expiry-wal-anchor",
    );
    let appended = append_one(&submission, anchor);
    synchronize(&submission, appended);

    for (key, generation) in [(194, 1), (195, 2), (196, 3)] {
        let completed = success_checkpoint(&serving, key);
        assert_eq!(
            completed.binding_compaction().generation().get(),
            generation
        );
        assert_eq!(completed.binding_compaction().terminal_binding_count(), 1);
    }
    let expiry = success_checkpoint(&serving, 197);
    assert_eq!(expiry.binding_compaction().generation().get(), 4);
    assert_eq!(expiry.binding_compaction().terminal_binding_count(), 0);
    serving.close();

    let after_expiry = serving_from_open(&root);
    assert!(matches!(
        after_expiry
            .certification_record_submission()
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                    b"expiring-terminal".as_slice()
                ])
                .unwrap(),
                placement,
                request(terminal_key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::IdempotencyExpired
        )
    ));
    after_expiry.close();
}

#[path = "clean_custody/mod.rs"]
pub(crate) mod clean_custody;
#[path = "selective_integrity/mod.rs"]
pub(crate) mod selective_integrity;

#[path = "live_binding_limit.rs"]
mod live_binding_limit;
