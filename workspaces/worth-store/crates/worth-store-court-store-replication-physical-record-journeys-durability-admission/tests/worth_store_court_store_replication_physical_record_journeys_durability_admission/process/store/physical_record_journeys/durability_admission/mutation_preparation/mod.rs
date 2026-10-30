#[allow(unused_imports)]
use crate::physical_record_journeys::durability_admission::mutation_preparation::*;

use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_with_group_limit,
    worth_store_testkit_store::physical_record_journeys::durability_with_pending_limit,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn durable_preparation_binds_once_without_media_effect_or_duplicate_identity_allocation() {
    let parent = tempfile::tempdir().unwrap();
    let media = media(&parent.path().join("store"));
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([21; 32]),
        )
        .unwrap();
    let before = serving.media_counters();
    let first = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"canonical payload".as_slice()
            ])
            .unwrap(),
            placement,
            request(key.clone()),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("fresh durable preparation must succeed"),
    };
    assert_eq!(
        first.disposition(),
        worth_store::physical_runtime::PhysicalMutationAdmissionDisposition::Fresh
    );
    assert_eq!(first.idempotency_identity(), key.identity());
    assert_eq!(first.idempotency_lease(), key.lease());
    assert_eq!(first.resources().record_count(), 1);
    assert_eq!(first.resources().payload_bytes(), 17);
    assert_eq!(first.resources().prepared_payload_bytes(), 17);
    assert_eq!(
        first.signal_profile(),
        serving.physical_signal_profile_identity()
    );
    assert_eq!(
        first
            .durability_policy_basis()
            .aspect_identity()
            .aspect_key()
            .as_str(),
        "store.physical.durability.policy-binding-basis"
    );

    let duplicate = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"canonical payload".as_slice()
            ])
            .unwrap(),
            placement,
            request(key.clone()),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("same-key same-fingerprint retry must deduplicate"),
    };
    assert_eq!(
        duplicate.disposition(),
        worth_store::physical_runtime::PhysicalMutationAdmissionDisposition::DuplicateUnresolved
    );
    assert_eq!(duplicate.mutation_identity(), first.mutation_identity());
    assert_eq!(duplicate.request_fingerprint(), first.request_fingerprint());

    assert!(matches!(
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                    b"different payload".as_slice()
                ])
                .unwrap(),
                placement,
                request(key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::IdempotencyConflict
        )
    ));
    let next_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([22; 32]),
        )
        .unwrap();
    let next = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"next payload".as_slice()
            ])
            .unwrap(),
            placement,
            request(next_key),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("next fresh durable preparation must succeed"),
    };
    assert_eq!(
        next.mutation_identity().operation_identity().get(),
        first.mutation_identity().operation_identity().get() + 1,
        "duplicate and conflict admission must not consume operation identities"
    );
    assert_eq!(serving.media_counters(), before);
    serving.close();
}

#[test]
fn source_failure_and_released_or_foreign_authority_remain_pre_effect() {
    let parent = tempfile::tempdir().unwrap();
    let media_owner = media(&parent.path().join("store"));
    let policy = durability_with_group_limit(&media_owner, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(media_owner.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([23; 32]),
        )
        .unwrap();
    let before = serving.media_counters();
    let short = worth_store::physical_runtime::RecordAppendBatch::builder()
        .push_source(ShortSource { emitted: false })
        .build()
        .unwrap();
    assert!(matches!(
        submission
            .prepare_durable_append(short, placement, request(key.clone()))
            .into_raw(),
        worth_proof::TransitionOutcome::Failed(worth_store::physical_runtime::PhysicalMutationPreparationFailure::Stream(failure))
            if failure.kind() == worth_store::physical_runtime::RecordStreamFailureKind::SourceEndedEarly
                && failure.completed_range() == (0..2)
    ));
    assert_eq!(serving.media_counters(), before);
    assert!(matches!(
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                    b"recovered".as_slice()
                ])
                .unwrap(),
                placement,
                request(key.clone()),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(_)
        )
    ));

    let foreign_parent = tempfile::tempdir().unwrap();
    let foreign_media = media(&foreign_parent.path().join("store"));
    let foreign_policy =
        durability_with_group_limit(&foreign_media, std::num::NonZeroU32::new(32).unwrap());
    let foreign = success(foreign_media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format,
            placement,
            access,
            foreign_policy,
        ),
    ));
    let foreign_before = foreign.media_counters();
    assert!(matches!(
        foreign
            .record_submission()
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                    b"foreign".as_slice()
                ])
                .unwrap(),
                placement,
                request(key.clone()),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::RebindRequired(
            worth_store::physical_runtime::PhysicalMutationPreparationRebindRequired::ForeignStore
        )
    ));
    assert_eq!(foreign.media_counters(), foreign_before);
    foreign.close();

    serving.close();
    assert!(matches!(
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"closed".as_slice()]).unwrap(),
                placement,
                request(key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Stale(worth_store::physical_runtime::PhysicalMutationPreparationStale::PublicationAuthorityReleased)
    ));
}

#[test]
fn admitted_pending_bound_and_reopened_policy_identity_are_enforced_before_effect() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media_owner = media(&root);
    let policy = durability_with_pending_limit(
        &media_owner,
        std::num::NonZeroU32::new(32).unwrap(),
        std::num::NonZeroU32::new(1).unwrap(),
    );
    let (format, placement, access) = configuration();
    let serving = success(media_owner.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    let submission = serving.record_submission();
    let retained_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([31; 32]),
        )
        .unwrap();
    let bounded_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([32; 32]),
        )
        .unwrap();
    let first = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"retained".as_slice()
            ])
            .unwrap(),
            placement,
            request(retained_key.clone()),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("first pending mutation must admit"),
    };
    let before_bound = serving.media_counters();
    assert!(matches!(
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"bounded".as_slice()]).unwrap(),
                placement,
                request(bounded_key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Deferred(
            worth_store::physical_runtime::PhysicalMutationPreparationDeferred::PendingUnresolvedLimitReached
        )
    ));
    assert_eq!(serving.media_counters(), before_bound);
    serving.close();

    let reopened_media = media(&root);
    let changed_policy =
        durability_with_group_limit(&reopened_media, std::num::NonZeroU32::new(64).unwrap());
    let reopened = success(reopened_media.open_record_store(
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, changed_policy),
    ));
    let before_rebind = reopened.media_counters();
    assert!(matches!(
        reopened
            .record_submission()
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"retained".as_slice()]).unwrap(),
                placement,
                request(retained_key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::RebindRequired(
            worth_store::physical_runtime::PhysicalMutationPreparationRebindRequired::ForeignDurabilityPolicy
        )
    ));
    assert_eq!(reopened.media_counters(), before_rebind);
    assert_eq!(
        first.disposition(),
        worth_store::physical_runtime::PhysicalMutationAdmissionDisposition::Fresh
    );
    reopened.close();
}

#[test]
fn streamed_and_owned_payloads_share_equivalence_despite_key_and_deadline_changes() {
    let parent = tempfile::tempdir().unwrap();
    let media_owner = media(&parent.path().join("store"));
    let policy = durability_with_group_limit(&media_owner, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(media_owner.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        ),
    ));
    let submission = serving.record_submission();
    let owned_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([41; 32]),
        )
        .unwrap();
    let streamed_key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([42; 32]),
        )
        .unwrap();
    let before = serving.media_counters();
    let owned = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"same bytes".as_slice()
            ])
            .unwrap(),
            placement,
            request_at(owned_key, 1_000),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("owned payload must prepare"),
    };
    let streamed_batch = worth_store::physical_runtime::RecordAppendBatch::builder()
        .push_source(crate::physical_record_journeys::durability_admission::mutation_preparation::ExactSource::new(b"same bytes"))
        .build()
        .unwrap();
    let streamed = match submission
        .prepare_durable_append(streamed_batch, placement, request_at(streamed_key, 2_000))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("streamed payload must prepare"),
    };
    assert_eq!(
        streamed.request_fingerprint(),
        owned.request_fingerprint(),
        "key, lease, deadline, and source representation are not request equivalence"
    );
    assert_eq!(streamed.resources(), owned.resources());
    assert_eq!(serving.media_counters(), before);
    serving.close();
}
