use super::*;

#[test]
fn total_live_binding_limit_remains_distinct_after_pending_work_becomes_terminal() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media_owner = media(&root);
    let policy = durability_with_idempotency_limits(
        &media_owner,
        std::num::NonZeroU32::new(2).unwrap(),
        std::num::NonZeroU32::new(1).unwrap(),
    );
    let (format, placement, access) = configuration();
    let serving = success(media_owner.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    let submission = serving.certification_record_submission();
    let first_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([184; 32]),
        )
        .unwrap();
    let first = prepare(&submission, placement, first_key, b"terminal-binding");
    assert!(matches!(
        submission.cancel_prepared_before_group_seal(first),
        worth_store::physical_runtime::PhysicalPreSealCancellationOutcome::ProvenNoEffect(_)
    ));

    let second_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([185; 32]),
        )
        .unwrap();
    assert!(matches!(
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"live-bound".as_slice()]).unwrap(),
                placement,
                request(second_key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Deferred(worth_store::physical_runtime::PhysicalMutationPreparationDeferred::LiveBindingLimitReached)
    ));
    serving.close();
}
