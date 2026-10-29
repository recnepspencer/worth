#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::group_commit::*;
use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::PhysicalDurabilityGroupAdmissionDenial, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyKey, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalPreSealCancellationDenial, worth_store::physical_runtime::PhysicalPreSealCancellationOutcome, worth_store::physical_runtime::PhysicalRecordSubmission, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch};
use {worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

#[test]
fn four_request_identity_blender_cancels_one_and_shares_one_barrier_across_three_members() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();

    let cancelled_key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([201; 32]))
        .unwrap();
    let cancelled = prepare(&submission, placement, cancelled_key.clone(), b"cancelled");
    let cancelled_identity = cancelled.mutation_identity();
    let cancelled_fingerprint = cancelled.request_fingerprint();
    let cancellation_request = prepare(&submission, placement, cancelled_key.clone(), b"cancelled");
    let terminal = match submission.cancel_prepared_before_group_seal(cancellation_request) {
        worth_store::physical_runtime::PhysicalPreSealCancellationOutcome::ProvenNoEffect(terminal) => terminal,
        worth_store::physical_runtime::PhysicalPreSealCancellationOutcome::NotCancelled { cause, .. } => {
            panic!("pre-seal cancellation must be terminal: {cause:?}")
        }
    };
    assert_eq!(terminal.mutation_identity(), cancelled_identity);
    assert_eq!(terminal.request_fingerprint(), cancelled_fingerprint);
    assert_eq!(
        terminal.cause(),
        worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal
    );
    assert_eq!(submission.wal_observation().unwrap().appended_frames(), 0);
    match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(cancelled, Vec::new())) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => assert_eq!(
            rejected.cause(),
            worth_store::physical_runtime::PhysicalDurabilityGroupAdmissionDenial::IdempotencyProvenNoEffect
        ),
        _ => panic!("a terminal duplicate must invalidate every older prepared value"),
    }
    assert_eq!(submission.wal_observation().unwrap().appended_frames(), 0);

    let replay = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"cancelled".as_slice()]).unwrap(),
            placement,
            request(cancelled_key.clone()),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::ProvenNoEffect(replay)) => {
            replay
        }
        _ => panic!("same-fingerprint replay must return the retained terminal fact"),
    };
    assert_eq!(replay, terminal);
    assert!(matches!(
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"conflict".as_slice()]).unwrap(),
                placement,
                request(cancelled_key),
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalMutationPreparationDenial::IdempotencyConflict)
    ));
    assert_eq!(submission.wal_observation().unwrap().appended_frames(), 0);

    let first_key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([202; 32]))
        .unwrap();
    let second_key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([203; 32]))
        .unwrap();
    let third_key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([204; 32]))
        .unwrap();
    let first = prepare(&submission, placement, first_key.clone(), b"first");
    let first_duplicate = prepare(&submission, placement, first_key, b"first");
    let second = prepare(&submission, placement, second_key, b"second");
    let third = prepare(&submission, placement, third_key, b"third");

    let appended =
        match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(first, vec![second, third])) {
            worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
            _ => panic!("three runtime-width members must seal as one exact group"),
        };
    assert_eq!(appended.members().len(), 3);
    let append_amplification = appended.amplification_observation();
    assert_eq!(append_amplification.mutations_admitted(), 3);
    assert_eq!(append_amplification.groups_formed(), 1);
    assert_eq!(append_amplification.members_per_group(), 3);
    assert_eq!(append_amplification.wal_frames_appended(), 3);
    assert!(append_amplification.wal_bytes_appended() > 0);
    assert_eq!(append_amplification.root_publications_planned(), 1);
    assert_eq!(append_amplification.shared_root_publications_planned(), 1);
    assert_eq!(submission.wal_observation().unwrap().appended_frames(), 3);
    let basis = appended.basis();
    let mutation_identities = appended
        .members()
        .iter()
        .map(|member| member.mutation().mutation_identity())
        .collect::<Vec<_>>();
    let idempotency_identities = appended
        .members()
        .iter()
        .map(|member| member.mutation().reserved().idempotency_identity())
        .collect::<Vec<_>>();
    for left in 0..3 {
        for right in left + 1..3 {
            assert_ne!(mutation_identities[left], mutation_identities[right]);
            assert_ne!(idempotency_identities[left], idempotency_identities[right]);
        }
    }
    for (index, member) in appended.members().iter().enumerate() {
        assert_eq!(member.binding().group_identity(), basis.identity());
        assert_eq!(member.binding().ordinal().get(), index as u32 + 1);
        assert_eq!(member.binding().member_count().get(), 3);
        assert_eq!(
            member.binding().membership_digest(),
            basis.membership_digest()
        );
    }

    match submission.cancel_prepared_before_group_seal(first_duplicate) {
        worth_store::physical_runtime::PhysicalPreSealCancellationOutcome::NotCancelled {
            cause: worth_store::physical_runtime::PhysicalPreSealCancellationDenial::GroupAlreadySealed,
            ..
        } => {}
        _ => panic!("group sealing must permanently close pre-seal cancellation"),
    }

    let sync_before = serving
        .media_counters()
        .identified_operation_attempts_for(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState);
    let durable = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(durable) => durable,
        _ => panic!("one shared barrier must derive every member proof"),
    };
    let sync_after = serving
        .media_counters()
        .identified_operation_attempts_for(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState);
    assert_eq!(sync_after - sync_before, 1);
    assert_eq!(durable.members().len(), 3);
    assert_eq!(durable.basis(), basis);
    let barrier_amplification = durable.amplification_observation();
    assert_eq!(barrier_amplification.append(), append_amplification);
    assert_eq!(barrier_amplification.wal_barrier_executions(), 1);
    assert_eq!(barrier_amplification.members_proven_wal_durable(), 3);
    let shared = durable.barrier_settlement();
    for member in durable.members() {
        assert_eq!(member.barrier_settlement().group_settlement(), shared);
        assert_eq!(member.group_binding().group_identity(), basis.identity());
        assert_eq!(
            member.mutation_identity(),
            member.barrier_settlement().mutation_identity()
        );
    }
    serving.close();
}
