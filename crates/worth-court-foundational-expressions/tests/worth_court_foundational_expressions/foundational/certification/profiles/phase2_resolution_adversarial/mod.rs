#[allow(unused_imports)] use crate::certification::ordinary_context::profiles::phase2_resolution_adversarial::*;
use {worth_foundational::admit_requested_foundational_profile_with_resolutions, worth_foundational::foundational_profile_progression_authority, worth_foundational::materialize_admitted_foundational_profile_with_resolutions, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::ExecutionObjectiveProfile, worth_foundational::FoundationalProfileProgressionDenial, worth_foundational::FoundationalProfileResolutionFamily, worth_foundational::FoundationalProfileResolutionLedger, worth_foundational::FoundationalProfileResolutionLedgerDenial, worth_foundational::FoundationalProfileResolutionRecord, worth_foundational::FoundationalProfileResolutionRelation, worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput, worth_foundational::ObservationActivationProfile, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
use {worth_proof::TransitionOutcome};

#[test]
fn resolution_ledger_rejects_duplicates_and_wrong_relations() {
    let mut ledger = worth_foundational::FoundationalProfileResolutionLedger::empty();
    let objective = worth_foundational::FoundationalProfileResolutionRecord::new(
        worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective,
        worth_foundational::FoundationalProfileResolutionRelation::ObjectiveSelection,
        "objective selected",
    );
    ledger.insert(objective).expect("first objective record");
    assert_eq!(
        ledger.insert(objective),
        Err(worth_foundational::FoundationalProfileResolutionLedgerDenial::DuplicateFamily(
            worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective,
        ))
    );

    let requested = base_profile(
        worth_foundational::ExecutionObjectiveProfile::Balanced,
        worth_foundational::ObservationActivationProfile::Continuous,
    );
    let admitted = profile_with_axes(
        requested,
        worth_foundational::ExecutionObjectiveProfile::Throughput,
        worth_foundational::ObservationActivationProfile::OnDemand,
    );
    let mut wrong_relation = worth_foundational::FoundationalProfileResolutionLedger::empty();
    wrong_relation
        .insert(objective)
        .expect("objective family is unique");
    wrong_relation
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::ObservationActivation,
            worth_foundational::FoundationalProfileResolutionRelation::ObjectiveSelection,
            "wrong relation",
        ))
        .expect("activation family is unique");
    assert_eq!(
        admit_requested_foundational_profile_with_resolutions(
            request_foundational_profile_set(requested),
            admitted,
            wrong_relation,
            foundational_profile_progression_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileProgressionDenial::ResolutionRelationMismatch(
                worth_foundational::FoundationalProfileResolutionFamily::ObservationActivation,
            )
        )
    );

    let mut wrong_objective_relation = worth_foundational::FoundationalProfileResolutionLedger::empty();
    wrong_objective_relation
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective,
            worth_foundational::FoundationalProfileResolutionRelation::ActivationSelection,
            "wrong objective relation",
        ))
        .expect("objective family is unique");
    wrong_objective_relation
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::ObservationActivation,
            worth_foundational::FoundationalProfileResolutionRelation::ActivationSelection,
            "activation selected",
        ))
        .expect("activation family is unique");
    assert_eq!(
        admit_requested_foundational_profile_with_resolutions(
            request_foundational_profile_set(requested),
            admitted,
            wrong_objective_relation,
            foundational_profile_progression_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileProgressionDenial::ResolutionRelationMismatch(
                worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective,
            )
        )
    );

    let mut omitted = worth_foundational::FoundationalProfileResolutionLedger::empty();
    omitted
        .insert(objective)
        .expect("objective family is unique");
    assert_eq!(
        admit_requested_foundational_profile_with_resolutions(
            request_foundational_profile_set(requested),
            admitted,
            omitted,
            foundational_profile_progression_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileProgressionDenial::ResolutionLedgerDoesNotMatchProfileChange,
        )
    );

    let mut unexpected = worth_foundational::FoundationalProfileResolutionLedger::empty();
    unexpected
        .insert(objective)
        .expect("objective family is unique");
    unexpected
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::DiagnosticRichness,
            worth_foundational::FoundationalProfileResolutionRelation::Narrowing,
            "unexpected family",
        ))
        .expect("unexpected family is unique");
    assert_eq!(
        admit_requested_foundational_profile_with_resolutions(
            request_foundational_profile_set(requested),
            admitted,
            unexpected,
            foundational_profile_progression_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileProgressionDenial::ResolutionLedgerDoesNotMatchProfileChange,
        )
    );
}

#[test]
fn resolution_records_are_canonical_and_survive_materialization() {
    let requested = base_profile(
        worth_foundational::ExecutionObjectiveProfile::Balanced,
        worth_foundational::ObservationActivationProfile::Continuous,
    );
    let admitted_profile = profile_with_axes(
        requested,
        worth_foundational::ExecutionObjectiveProfile::Throughput,
        worth_foundational::ObservationActivationProfile::OnDemand,
    );
    let materialized_profile = profile_with_axes(
        admitted_profile,
        worth_foundational::ExecutionObjectiveProfile::Throughput,
        worth_foundational::ObservationActivationProfile::Continuous,
    );
    let mut reverse = worth_foundational::FoundationalProfileResolutionLedger::empty();
    reverse
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::ObservationActivation,
            worth_foundational::FoundationalProfileResolutionRelation::ActivationSelection,
            "activation selected",
        ))
        .expect("activation family");
    reverse
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective,
            worth_foundational::FoundationalProfileResolutionRelation::ObjectiveSelection,
            "objective selected",
        ))
        .expect("objective family");
    let admitted = admit_with_resolutions(requested, admitted_profile, reverse);
    let records: Vec<_> = admitted
        .payload()
        .requested_to_admitted_resolutions()
        .records()
        .map(|record| record.family())
        .collect();
    assert_eq!(
        records,
        vec![
            worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective,
            worth_foundational::FoundationalProfileResolutionFamily::ObservationActivation,
        ]
    );

    let materialized = match materialize_admitted_foundational_profile_with_resolutions(
        admitted.clone(),
        materialized_profile,
        resolution_ledger(false, true),
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected materialized profile, got {other:?}"),
    };
    assert_eq!(
        materialized
            .payload()
            .requested_to_admitted_resolutions()
            .len(),
        2
    );
    assert_eq!(
        materialized
            .payload()
            .admitted_to_materialized_resolutions()
            .get(worth_foundational::FoundationalProfileResolutionFamily::ObservationActivation)
            .expect("activation materialization record")
            .relation(),
        worth_foundational::FoundationalProfileResolutionRelation::ActivationSelection
    );
    assert_eq!(
        materialized
            .payload()
            .admitted_to_materialized_resolutions()
            .len(),
        1
    );
    assert!(materialized
        .payload()
        .admitted_to_materialized_resolutions()
        .get(worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective)
        .is_none());

    let omitted = match materialize_admitted_foundational_profile_with_resolutions(
        admitted,
        materialized_profile,
        worth_foundational::FoundationalProfileResolutionLedger::empty(),
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected omitted materialization resolution denial, got {other:?}"),
    };
    assert_eq!(
        omitted,
        worth_foundational::FoundationalProfileProgressionDenial::ResolutionLedgerDoesNotMatchProfileChange
    );
}
