use {std::num::NonZeroU32};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::PhysicalEffectObligation, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalWalAppendFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::PhysicalWorkCapacityDimension, worth_store::physical_runtime::PhysicalWorkReadiness, worth_store::physical_runtime::RecordAppendBatch};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_group_limit, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture, worth_store_testkit_store::physical_record_journeys::success};

#[test]
fn bounded_work_deferral_returns_exact_cause_and_preparation_for_retry() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let media = media(&store_root);
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (profile, read_request, _) = work_fixture();
    let profile = profile.with_capacity(
        worth_store::physical_runtime::PhysicalWorkCapacity::new(1, 256, 256, 1024 * 1024, 1024 * 1024)
            .unwrap()
            .with_terminal_evidence_capacity(4)
            .unwrap(),
    );
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, policy)
                .with_physical_work_profile(profile),
        ),
    );
    let occupied = match serving
        .physical_read_submission()
        .submit(read_request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        other => panic!("the one command slot must be occupied: {other:?}"),
    };
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([87; 32]))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"capacity-retry-redo"]).unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap()),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("durable preparation must precede work admission"),
    };
    let identity = prepared.mutation_identity();
    let fingerprint = prepared.request_fingerprint();

    let continuation =
        match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new())) {
            worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotStarted(continuation) => {
                let worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause::Append(
                    worth_store::physical_runtime::PhysicalWalAppendFailureCause::SubmissionDeferred(deferred),
                ) = continuation.cause()
                else {
                    panic!("bounded work pressure must retain its exact deferred cause")
                };
                assert_eq!(
                    deferred.dimension(),
                    worth_store::physical_runtime::PhysicalWorkCapacityDimension::Commands
                );
                assert_eq!(deferred.capacity(), 1);
                assert_eq!(continuation.appended_member_count(), 0);
                assert_eq!(continuation.remaining_member_count(), 1);
                continuation
            }
            _ => panic!("bounded work pressure must retain one exact continuation"),
        };
    assert_eq!(submission.wal_observation().unwrap().appended_frames(), 0);

    let admitted = serving.admit_physical_work(occupied).unwrap();
    let ready = match serving.request_physical_work(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => panic!("the fixture dependency must be ready"),
    };
    let cancellation = serving
        .cancel_physical_work(ready.consumer_handle())
        .unwrap();
    assert_eq!(
        cancellation.obligation(),
        worth_store::physical_runtime::PhysicalEffectObligation::NotDispatched
    );
    drop(ready);

    let appended = match submission.continue_prepared_wal_group(continuation) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("the exact continuation must append once capacity returns"),
    };
    let appended = appended.members()[0].mutation();
    assert_eq!(appended.mutation_identity(), identity);
    assert_eq!(appended.reserved().request_fingerprint(), fingerprint);
    serving.close();
}
