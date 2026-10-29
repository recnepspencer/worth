#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::wal_preparation_authority::*;
use {std::num::NonZeroU32, std::path::Path};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint};
use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::PhysicalDurabilityGroupAdmissionDenial, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordSubmission, worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalReservationDenial, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_group_limit, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};

#[test]
fn concurrent_append_denial_preserves_the_second_preparation_for_retry() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let media = media(&store_root);
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let first = prepare(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([84; 32]),
    );
    let second = prepare(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([85; 32]),
    );
    let second_identity = second.mutation_identity();
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let first_submission = submission.clone();
    let first_append =
        std::thread::spawn(move || first_submission.append_prepared_wal_group(singleton(first)));
    assert!(gate.await_arrival());

    let continuation = match submission.append_prepared_wal_group(singleton(second)) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotStarted(continuation)
            if matches!(
                continuation.cause(),
                worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause::Reservation(
                    worth_store::physical_runtime::PhysicalWalReservationDenial::AppendInFlight
                )
            ) =>
        {
            continuation
        }
        _ => panic!("the concurrent append must retain one exact continuation"),
    };
    assert_eq!(continuation.appended_member_count(), 0);
    assert_eq!(continuation.remaining_member_count(), 1);
    gate.release();
    assert!(matches!(
        first_append.join().unwrap(),
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(_)
    ));
    let appended = match submission.continue_prepared_wal_group(continuation) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("the retained continuation must append after the first group"),
    };
    assert_eq!(
        appended.members()[0].mutation().mutation_identity(),
        second_identity
    );
    assert_eq!(submission.wal_observation().unwrap().appended_frames(), 2);
    serving.close();
}

#[test]
fn foreign_store_preparation_is_denied_without_effect_and_remains_appendable_by_its_owner() {
    let parent = tempfile::tempdir().unwrap();
    let first_root = parent.path().join("first");
    let second_root = parent.path().join("second");
    let first_media = media(&first_root);
    let second_media = media(&second_root);
    let first_policy = durability_with_group_limit(&first_media, std::num::NonZeroU32::new(32).unwrap());
    let second_policy = durability_with_group_limit(&second_media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let first = success(
        first_media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format,
            placement,
            access,
            first_policy,
        )),
    );
    let (format, _, access) = configuration();
    let second = success(
        second_media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format,
            placement,
            access,
            second_policy,
        )),
    );
    let first_submission = first.certification_record_submission();
    let second_submission = second.certification_record_submission();
    let prepared = prepare(
        &first_submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([81; 32]),
    );
    let identity = prepared.mutation_identity();
    let fingerprint = prepared.request_fingerprint();

    let preserved = match second_submission.append_prepared_wal_group(singleton(prepared)) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected)
            if rejected.cause() == worth_store::physical_runtime::PhysicalDurabilityGroupAdmissionDenial::ForeignStore =>
        {
            rejected.into_members().into_vec().pop().unwrap()
        }
        _ => panic!("a foreign Store preparation must be denied before WAL allocation"),
    };
    assert_eq!(preserved.mutation_identity(), identity);
    assert_eq!(preserved.request_fingerprint(), fingerprint);
    assert_untouched_wal(&second_root, &second_submission);

    let first_appended = match first_submission.append_prepared_wal_group(singleton(preserved)) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("the rightful Store must append its preserved preparation"),
    };
    let first_member = first_appended.members()[0].mutation();
    let second_prepared = prepare(
        &second_submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([81; 32]),
    );
    assert_eq!(
        first_member.mutation_identity().operation_identity(),
        second_prepared.mutation_identity().operation_identity(),
        "both independent runtimes begin at the same local mutation ordinal"
    );
    let second_appended =
        match second_submission.append_prepared_wal_group(singleton(second_prepared)) {
            worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
            _ => panic!("the second Store must append its own preparation"),
        };
    let second_member = second_appended.members()[0].mutation();
    assert_ne!(
        first_member.reserved().member_basis().member_identity(),
        second_member.reserved().member_basis().member_identity(),
        "equal local ordinals under different Store/runtime identities cannot collide"
    );
    first.close();
    second.close();
}

#[test]
fn stale_runtime_preparation_is_denied_without_effect_and_preserves_exact_identity() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let initial_media = media(&store_root);
    let initial_policy = durability_with_group_limit(&initial_media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let initial = success(initial_media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, initial_policy),
    ));
    let prepared = prepare(
        &initial.certification_record_submission(),
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([82; 32]),
    );
    let identity = prepared.mutation_identity();
    let fingerprint = prepared.request_fingerprint();
    initial.close();

    let reopened_media = media(&store_root);
    let reopened_policy =
        durability_with_group_limit(&reopened_media, std::num::NonZeroU32::new(32).unwrap());
    let (format, _, access) = configuration();
    let reopened = success(reopened_media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
        format,
        access,
        reopened_policy,
    )));
    assert_eq!(identity.store_identity(), reopened.store_identity());
    assert_ne!(identity.runtime_identity(), reopened.runtime_identity());
    let submission = reopened.certification_record_submission();

    let preserved = match submission.append_prepared_wal_group(singleton(prepared)) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected)
            if rejected.cause() == worth_store::physical_runtime::PhysicalDurabilityGroupAdmissionDenial::ForeignRuntime =>
        {
            rejected.into_members().into_vec().pop().unwrap()
        }
        _ => panic!("a stale runtime preparation must be denied before WAL allocation"),
    };
    assert_eq!(preserved.mutation_identity(), identity);
    assert_eq!(preserved.request_fingerprint(), fingerprint);
    assert_untouched_wal(&store_root, &submission);
    reopened.close();
}

#[test]
fn released_submission_reports_lifecycle_loss_without_fabricating_wal_inspection() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let media = media(&store_root);
    let policy = durability_with_group_limit(&media, std::num::NonZeroU32::new(32).unwrap());
    let (format, placement, access) = configuration();
    let serving = success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    let submission = serving.certification_record_submission();
    let prepared = prepare(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([86; 32]),
    );
    let identity = prepared.mutation_identity();
    let fingerprint = prepared.request_fingerprint();
    serving.close();

    let preserved = match submission.append_prepared_wal_group(singleton(prepared)) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotAdmitted {
            members,
            cause: worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause::RuntimeReleased,
        } => members.into_vec().pop().unwrap(),
        _ => panic!("released publication authority must not claim WAL inspection is required"),
    };
    assert_eq!(preserved.mutation_identity(), identity);
    assert_eq!(preserved.request_fingerprint(), fingerprint);
    assert!(submission.wal_observation().is_none());
}
