use std::num::NonZeroU32;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalMutationAdmissionDisposition, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationIndeterminateStage, PhysicalMutationOutcome,
    PhysicalMutationPreparationDeferred, PhysicalMutationPreparationDenial,
    PhysicalMutationPreparationSuccess, PhysicalPreSealCancellationOutcome, RecordAppendBatch,
    RecordAppendDenial,
};

#[path = "idempotency_reopen/selective_integrity.rs"]
mod selective_integrity;
#[path = "idempotency_reopen/support.rs"]
mod support;

use support::{
    append_one, inspect_checkpoint_reopen, prepare, request, success_checkpoint, synchronize,
};

use super::super::{
    configuration, durability_with_idempotency_limits, media, serving_from_initialization,
    serving_from_open, success,
};

#[test]
fn fresh_process_rebuild_joins_checkpoint_compaction_with_the_retained_wal_suffix() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();

    let target_key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([181; 32]))
        .unwrap();
    let target = prepare(&submission, placement, target_key.clone(), b"tail-target");
    let target_mutation = target.mutation_identity();

    let anchor = super::wal_append::prepared(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([182; 32]),
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
    let expected_wal = super::independent_wal_oracle::inspect_wal_inventory(&root).unwrap();
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
        expected_checkpoint.binding_records + 3
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
        PhysicalMutationAdmissionDisposition::DuplicateUnresolved
    );
    assert_eq!(duplicate.mutation_identity(), target_mutation);
    assert_eq!(reopened.media_counters(), before_retry);

    assert!(matches!(
        reopened_submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"conflicting-tail".as_slice()]).unwrap(),
                placement,
                request(target_key),
            )
            .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::IdempotencyConflict)
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
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([186; 32]))
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
        PhysicalPreSealCancellationOutcome::ProvenNoEffect(_)
    ));
    let anchor = super::wal_append::prepared(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([187; 32]),
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
            RecordAppendBatch::try_from_iter([b"terminal-fate".as_slice()]).unwrap(),
            placement,
            request(terminal_key),
        )
        .into_raw();
    match replay {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(fate)) => {
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
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([189; 32]))
        .unwrap();
    let prepared = prepare(&submission, placement, key.clone(), b"completed-terminal");
    let completed = match prepared.execute() {
        PhysicalMutationOutcome::Completed(completed) => completed,
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
            RecordAppendBatch::try_from_iter([b"completed-terminal".as_slice()]).unwrap(),
            placement,
            request(key),
        )
        .into_raw();
    match replay {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(completed)) => {
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
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([191; 32]))
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
        PhysicalMutationOutcome::Indeterminate(fate) => fate,
        _ => panic!("candidate publication rejection must be indeterminate"),
    };
    assert_eq!(fate.mutation_identity(), expected_identity);
    assert_eq!(
        fate.diagnostic_evidence().mutation_identity(),
        expected_identity
    );
    assert_eq!(
        fate.stage(),
        PhysicalMutationIndeterminateStage::DataDispatch
    );
    assert_eq!(fate.completed_effect_count(), 1);

    let checkpoint = success_checkpoint(&serving, 192);
    assert_eq!(checkpoint.binding_compaction().terminal_binding_count(), 1);
    serving.close();

    let reopened = serving_from_open(&root);
    let replay = reopened
        .certification_record_submission()
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([b"indeterminate-terminal".as_slice()]).unwrap(),
            placement,
            request(key),
        )
        .into_raw();
    assert!(matches!(
        replay,
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::ServingRequiresInspection
        ))
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
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([192; 32]))
        .unwrap();
    let terminal = prepare(
        &submission,
        placement,
        terminal_key.clone(),
        b"expiring-terminal",
    );
    assert!(matches!(
        submission.cancel_prepared_before_group_seal(terminal),
        PhysicalPreSealCancellationOutcome::ProvenNoEffect(_)
    ));
    let anchor = super::wal_append::prepared(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([193; 32]),
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
                RecordAppendBatch::try_from_iter([b"expiring-terminal".as_slice()]).unwrap(),
                placement,
                request(terminal_key),
            )
            .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::IdempotencyExpired)
    ));
    after_expiry.close();
}

#[test]
fn total_live_binding_limit_remains_distinct_after_pending_work_becomes_terminal() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media_owner = media(&root);
    let policy = durability_with_idempotency_limits(
        &media_owner,
        NonZeroU32::new(2).unwrap(),
        NonZeroU32::new(1).unwrap(),
    );
    let (format, placement, access) = configuration();
    let serving = success(media_owner.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    let submission = serving.certification_record_submission();
    let first_key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([184; 32]))
        .unwrap();
    let first = prepare(&submission, placement, first_key, b"terminal-binding");
    assert!(matches!(
        submission.cancel_prepared_before_group_seal(first),
        PhysicalPreSealCancellationOutcome::ProvenNoEffect(_)
    ));

    let second_key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([185; 32]))
        .unwrap();
    assert!(matches!(
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"live-bound".as_slice()]).unwrap(),
                placement,
                request(second_key),
            )
            .into_raw(),
        TransitionOutcome::Deferred(PhysicalMutationPreparationDeferred::LiveBindingLimitReached)
    ));
    serving.close();
}
