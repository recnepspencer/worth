use worth_proof::NonEmpty;
use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    DataSettledPhysicalMutation, PhysicalCurrentRootAdvanceOutcome, PhysicalDataDispatchOutcome,
    PhysicalDataSettledGroupDenial, PhysicalDataSettlementOutcome, PhysicalDurabilityGroupBasis,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalRootNamespaceDurabilityOutcome, PhysicalRootPublicationPreparationFailureCause,
    PhysicalRootPublicationPreparationOutcome, PhysicalRootReplacementOutcome,
    PhysicalWalGroupAppendOutcome, PhysicalWalGroupBarrierOutcome, RecordAppendBatch,
    SettledRootProjectionMergeDenial,
};
use worth_store_physical_format::{BlobGenerationPublicationV1, PersistedRecordIdentity};

use super::super::super::{
    configuration, durable_publication::settle_single, serving_from_initialization,
};

#[path = "root_projection_carriage/classified_blob_publications.rs"]
mod classified_blob_publications;

#[test]
fn settled_member_carries_its_root_projection_into_the_exact_group_join() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let (basis, settled) = settled_member(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([141; 32]),
        b"root-projection-carriage",
    );

    assert_eq!(settled.prepared_root_source_generation(), 1);
    let joined = match submission.join_data_settled_group(basis, NonEmpty::new(settled, Vec::new()))
    {
        Ok(joined) => joined,
        Err(rejected) => panic!("exact group rejected: {:?}", rejected.cause()),
    };
    assert_eq!(joined.basis(), basis);
    assert_eq!(joined.members().len(), 1);
    assert_eq!(joined.members()[0].prepared_root_source_generation(), 1);
    serving.close();
}

#[test]
fn settled_member_cannot_be_rebound_to_an_unrelated_group() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let (first_basis, first) = settled_member(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([142; 32]),
        b"first-root-projection",
    );
    let (_, second) = settled_member(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([143; 32]),
        b"second-root-projection",
    );

    match submission.join_data_settled_group(first_basis, NonEmpty::new(first, Vec::new())) {
        Ok(_) => {}
        Err(rejected) => panic!("first exact group rejected: {:?}", rejected.cause()),
    }
    let rejected =
        match submission.join_data_settled_group(first_basis, NonEmpty::new(second, Vec::new())) {
            Ok(_) => panic!("a settled member borrowed another group's basis"),
            Err(rejected) => rejected,
        };
    assert_eq!(
        rejected.cause(),
        PhysicalDataSettledGroupDenial::GroupIdentityMismatch
    );
    assert_eq!(rejected.into_members().len(), 1);
    serving.close();
}

#[test]
fn exact_settled_group_advances_only_after_replacement_and_namespace_durability() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let records = root.join("families/records");
    let genesis_current = worth_store_physical_format::DurableRootSelector::decode(
        &std::fs::read(records.join("root-current.selector")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (genesis_current.role(), genesis_current.root_generation()),
        (worth_store_physical_format::RootSelectorRole::Current, 1)
    );
    assert_eq!(genesis_current.linked_selector(), None);
    assert!(
        !records.join("root-previous.selector").exists(),
        "genesis has no predecessor and must not invent a previous selector"
    );
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let (basis, settled) = settled_member(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([144; 32]),
        b"root-publication-prepared",
    );
    let mutation = settled.mutation_identity();
    let data_effect_count = settled.dispatched().effects().len();
    let joined = submission
        .join_data_settled_group(basis, NonEmpty::new(settled, Vec::new()))
        .unwrap_or_else(|rejected| panic!("exact group rejected: {:?}", rejected.cause()));

    let prepared = match submission.prepare_root_publication(joined) {
        PhysicalRootPublicationPreparationOutcome::Prepared(prepared) => prepared,
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            panic!("root preparation did not start: {:?}", failure.cause())
        }
        PhysicalRootPublicationPreparationOutcome::InspectionRequired(failure) => {
            panic!(
                "root preparation became indeterminate: {:?}",
                failure.cause()
            )
        }
    };

    assert_eq!(prepared.group_basis(), basis);
    assert_eq!(prepared.members().len(), 1);
    assert_eq!(prepared.settled_members().len(), 1);
    assert_eq!(prepared.settled_members()[0].mutation_identity(), mutation);
    assert_eq!(
        prepared.settled_members()[0].data_effect_count(),
        data_effect_count
    );
    assert_eq!(prepared.settled_members()[0].persisted_records().len(), 1);
    assert_eq!(prepared.source_root_generation(), 1);
    assert_eq!(prepared.candidate_root_generation(), 2);
    assert!(prepared.candidate_artifacts().len() >= 2);
    assert_eq!(
        prepared.candidate_synchronization_count(),
        prepared.candidate_artifacts().len(),
        "C5_PREDICATE:publication-durability: every root candidate artifact must be synchronized"
    );

    let replaced = match submission.replace_prepared_root(prepared) {
        PhysicalRootReplacementOutcome::Replaced(replaced) => replaced,
        PhysicalRootReplacementOutcome::NotStarted(failure) => {
            panic!("root replacement did not start: {:?}", failure.cause())
        }
        PhysicalRootReplacementOutcome::InspectionRequired(failure) => {
            panic!(
                "root replacement became indeterminate: {:?} {:?}",
                failure.effect_fate(),
                failure.recovery_disposition()
            )
        }
    };
    assert_eq!(replaced.source_root_generation(), 1);
    assert_eq!(replaced.candidate_root_generation(), 2);
    assert_eq!(replaced.settled_members()[0].mutation_identity(), mutation);
    assert!(replaced.replacement_effect_identity().is_some());

    let durable = match submission.synchronize_replaced_root_namespace(replaced) {
        PhysicalRootNamespaceDurabilityOutcome::Durable(durable) => durable,
        PhysicalRootNamespaceDurabilityOutcome::NotStarted(failure) => {
            panic!(
                "namespace synchronization did not start: {:?}",
                failure.cause()
            )
        }
        PhysicalRootNamespaceDurabilityOutcome::InspectionRequired(failure) => {
            panic!(
                "namespace synchronization became indeterminate: {:?} {:?}",
                failure.effect_fate(),
                failure.recovery_disposition()
            )
        }
    };
    assert_eq!(durable.source_root_generation(), 1);
    assert_eq!(durable.current_root_generation(), 2);
    assert_eq!(durable.settled_members()[0].mutation_identity(), mutation);
    assert!(durable.replacement_effect_identity().is_some());
    assert!(durable.namespace_effect_identity().is_some());

    let completed = match submission.advance_namespace_durable_root(durable) {
        PhysicalCurrentRootAdvanceOutcome::Advanced(completed) => completed,
        PhysicalCurrentRootAdvanceOutcome::InspectionRequired(failure) => {
            panic!("current-root advance rejected: {:?}", failure.cause())
        }
    };
    assert_eq!(completed.current_root().generation(), 2);
    assert_eq!(completed.settled_members().len(), 1);
    assert_eq!(completed.settled_members()[0].mutation_identity(), mutation);
    assert_eq!(
        completed.settled_members()[0].data_effect_count(),
        data_effect_count
    );
    assert_eq!(completed.settled_members()[0].persisted_records().len(), 1);
    assert_eq!(completed.retained_root().manifest().generation(), 1);
    assert!(completed.retained_root().supporting_artifacts().contains(
        &worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 }
    ));
    let previous = worth_store_physical_format::DurableRootSelector::decode(
        &std::fs::read(records.join("root-previous.selector")).unwrap(),
    )
    .unwrap();
    let current = worth_store_physical_format::DurableRootSelector::decode(
        &std::fs::read(records.join("root-current.selector")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (previous.role(), previous.root_generation()),
        (worth_store_physical_format::RootSelectorRole::Previous, 1)
    );
    assert_eq!(
        (current.role(), current.root_generation()),
        (worth_store_physical_format::RootSelectorRole::Current, 2)
    );
    assert_eq!(previous.linked_selector(), Some(current.identity()));
    assert_eq!(current.linked_selector(), Some(previous.identity()));
    assert_eq!(
        previous.linked_root_generation(),
        Some(current.root_generation())
    );
    assert_eq!(
        current.linked_root_generation(),
        Some(previous.root_generation())
    );
    assert_eq!(previous.store_identity(), current.store_identity());
    assert_eq!(previous.format(), current.format());
    serving.close();
}

#[test]
fn active_transition_denial_returns_the_exact_settled_group_for_retry() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let (first_basis, first) = settled_member(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([145; 32]),
        b"first-active-root-transition",
    );
    let (second_basis, second) = settled_member(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([146; 32]),
        b"second-denied-root-transition",
    );
    let second_idempotency = second.idempotency_identity();
    let first_joined = submission
        .join_data_settled_group(first_basis, NonEmpty::new(first, Vec::new()))
        .unwrap_or_else(|rejected| panic!("first group rejected: {:?}", rejected.cause()));
    let second_joined = submission
        .join_data_settled_group(second_basis, NonEmpty::new(second, Vec::new()))
        .unwrap_or_else(|rejected| panic!("second group rejected: {:?}", rejected.cause()));
    let first_prepared = match submission.prepare_root_publication(first_joined) {
        PhysicalRootPublicationPreparationOutcome::Prepared(prepared) => prepared,
        _ => panic!("the first transition must become active"),
    };
    let returned = match submission.prepare_root_publication(second_joined) {
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            assert_eq!(
                failure.cause(),
                worth_store::physical_runtime::PhysicalRootPublicationPreparationFailureCause::
                    TransitionDenied(
                        worth_store::physical_runtime::PhysicalRootPublicationTransitionDenial::
                            TransitionActive,
                    )
            );
            failure
                .into_planning_members()
                .expect("fail-before transition denial must return the exact planning group")
        }
        _ => panic!("a concurrent root transition must be denied before effect"),
    };
    assert_eq!(returned.group_basis(), second_basis);
    assert_eq!(returned.settled_members().len(), 1);
    assert_eq!(
        returned.member_identities()[0].idempotency_identity(),
        second_idempotency,
    );

    drop(first_prepared);
    match submission.continue_root_publication_preparation(returned) {
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => assert_eq!(
            failure.cause(),
            worth_store::physical_runtime::PhysicalRootPublicationPreparationFailureCause::
                TransitionDenied(
                    worth_store::physical_runtime::PhysicalRootPublicationTransitionDenial::
                        InspectionRequired,
                )
        ),
        _ => panic!("abandoning an effect-bearing root must seal later preparation"),
    }
    serving.close();
}

#[test]
fn first_candidate_write_no_effect_returns_a_complete_retryable_plan() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let (basis, settled) = settled_member(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([147; 32]),
        b"retryable-root-candidate",
    );
    let mutation = settled.mutation_identity();
    let joined = submission
        .join_data_settled_group(basis, NonEmpty::new(settled, Vec::new()))
        .unwrap_or_else(|rejected| panic!("exact group rejected: {:?}", rejected.cause()));

    serving.certification_reject_next_candidate_retention_before_effect();
    let candidate = match submission.prepare_root_publication(joined) {
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            match failure.cause() {
                worth_store::physical_runtime::PhysicalRootPublicationPreparationFailureCause::
                    CandidateWrite {
                        completed_artifact_count,
                        cause:
                            worth_store::physical_runtime::PhysicalRootCandidateWriteFailureCause::
                                Residency { posture, .. },
                        ..
                    } => {
                        assert_eq!(completed_artifact_count, 0);
                        assert_eq!(
                            posture,
                            worth_store::physical_runtime::
                                PhysicalRootCandidateWriteFailurePosture::ProvenNoEffect
                        );
                    }
                cause => panic!("first candidate denial lost exact no-effect posture: {cause:?}"),
            }
            failure
                .into_candidate_plan()
                .expect("proven-no-effect candidate write must return its linear plan")
        }
        PhysicalRootPublicationPreparationOutcome::Prepared(_) => {
            panic!("the controlled first candidate write unexpectedly completed")
        }
        PhysicalRootPublicationPreparationOutcome::InspectionRequired(failure) => {
            panic!(
                "proven no effect poisoned root health: {:?}",
                failure.cause()
            )
        }
    };
    let prepared = match submission.continue_root_publication_candidate(candidate) {
        PhysicalRootPublicationPreparationOutcome::Prepared(prepared) => prepared,
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            panic!(
                "the restored candidate plan was incomplete: {:?}",
                failure.cause()
            )
        }
        PhysicalRootPublicationPreparationOutcome::InspectionRequired(failure) => {
            panic!(
                "the restored candidate retry became indeterminate: {:?}",
                failure.cause()
            )
        }
    };
    assert_eq!(prepared.group_basis(), basis);
    assert_eq!(prepared.settled_members()[0].mutation_identity(), mutation);
    assert!(prepared.candidate_synchronization_count() >= 2);
    drop(prepared);
    serving.close();
}

fn settled_member(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    idempotency: PhysicalMutationIdempotencyMaterial,
    payload: &'static [u8],
) -> (PhysicalDurabilityGroupBasis, DataSettledPhysicalMutation) {
    let settled = settle_single(
        submission,
        placement,
        idempotency,
        RecordAppendBatch::try_from_iter([payload]).unwrap(),
    );
    (settled.basis, settled.member)
}
