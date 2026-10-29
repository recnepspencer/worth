use worth_foundational::{
    admit_requested_foundational_profile_with_resolutions,
    foundational_profile_progression_authority,
    materialize_admitted_foundational_profile_with_resolutions, request_foundational_profile_set,
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalProfileProgressionDenial,
    FoundationalProfileResolutionFamily, FoundationalProfileResolutionLedger,
    FoundationalProfileResolutionLedgerDenial, FoundationalProfileResolutionRecord,
    FoundationalProfileResolutionRelation, FoundationalProfileSet, FoundationalProfileSetInput,
    ObservationActivationProfile, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

fn base_profile(
    objective: ExecutionObjectiveProfile,
    activation: ObservationActivationProfile,
) -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::Uncertified,
        execution_objective: objective,
        observation_activation: activation,
    })
    .expect("phase 2 profile axes compose")
}

fn profile_with_axes(
    profile: FoundationalProfileSet,
    objective: ExecutionObjectiveProfile,
    activation: ObservationActivationProfile,
) -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: profile.diagnostic_richness(),
        support_posture: profile.support_posture(),
        compatibility_posture: profile.compatibility_posture(),
        admission_readiness: profile.admission_readiness(),
        retention_delivery: profile.retention_delivery(),
        certification_posture: profile.certification_posture(),
        execution_objective: objective,
        observation_activation: activation,
    })
    .expect("phase 2 profile axes remain coherent")
}

fn resolution_ledger(objective: bool, activation: bool) -> FoundationalProfileResolutionLedger {
    let mut ledger = FoundationalProfileResolutionLedger::empty();
    if objective {
        ledger
            .insert(FoundationalProfileResolutionRecord::new(
                FoundationalProfileResolutionFamily::ExecutionObjective,
                FoundationalProfileResolutionRelation::ObjectiveSelection,
                "objective selected",
            ))
            .expect("objective family is unique");
    }
    if activation {
        ledger
            .insert(FoundationalProfileResolutionRecord::new(
                FoundationalProfileResolutionFamily::ObservationActivation,
                FoundationalProfileResolutionRelation::ActivationSelection,
                "activation selected",
            ))
            .expect("activation family is unique");
    }
    ledger
}

fn admit_with_resolutions(
    requested: FoundationalProfileSet,
    admitted: FoundationalProfileSet,
    resolutions: FoundationalProfileResolutionLedger,
) -> worth_foundational::AdmittedFoundationalProfileArtifact {
    match admit_requested_foundational_profile_with_resolutions(
        request_foundational_profile_set(requested),
        admitted,
        resolutions,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected admitted profile, got {other:?}"),
    }
}

#[test]
fn resolution_ledger_rejects_duplicates_and_wrong_relations() {
    let mut ledger = FoundationalProfileResolutionLedger::empty();
    let objective = FoundationalProfileResolutionRecord::new(
        FoundationalProfileResolutionFamily::ExecutionObjective,
        FoundationalProfileResolutionRelation::ObjectiveSelection,
        "objective selected",
    );
    ledger.insert(objective).expect("first objective record");
    assert_eq!(
        ledger.insert(objective),
        Err(FoundationalProfileResolutionLedgerDenial::DuplicateFamily(
            FoundationalProfileResolutionFamily::ExecutionObjective,
        ))
    );

    let requested = base_profile(
        ExecutionObjectiveProfile::Balanced,
        ObservationActivationProfile::Continuous,
    );
    let admitted = profile_with_axes(
        requested,
        ExecutionObjectiveProfile::Throughput,
        ObservationActivationProfile::OnDemand,
    );
    let mut wrong_relation = FoundationalProfileResolutionLedger::empty();
    wrong_relation
        .insert(objective)
        .expect("objective family is unique");
    wrong_relation
        .insert(FoundationalProfileResolutionRecord::new(
            FoundationalProfileResolutionFamily::ObservationActivation,
            FoundationalProfileResolutionRelation::ObjectiveSelection,
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
        TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::ResolutionRelationMismatch(
                FoundationalProfileResolutionFamily::ObservationActivation,
            )
        )
    );

    let mut wrong_objective_relation = FoundationalProfileResolutionLedger::empty();
    wrong_objective_relation
        .insert(FoundationalProfileResolutionRecord::new(
            FoundationalProfileResolutionFamily::ExecutionObjective,
            FoundationalProfileResolutionRelation::ActivationSelection,
            "wrong objective relation",
        ))
        .expect("objective family is unique");
    wrong_objective_relation
        .insert(FoundationalProfileResolutionRecord::new(
            FoundationalProfileResolutionFamily::ObservationActivation,
            FoundationalProfileResolutionRelation::ActivationSelection,
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
        TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::ResolutionRelationMismatch(
                FoundationalProfileResolutionFamily::ExecutionObjective,
            )
        )
    );

    let mut omitted = FoundationalProfileResolutionLedger::empty();
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
        TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::ResolutionLedgerDoesNotMatchProfileChange,
        )
    );

    let mut unexpected = FoundationalProfileResolutionLedger::empty();
    unexpected
        .insert(objective)
        .expect("objective family is unique");
    unexpected
        .insert(FoundationalProfileResolutionRecord::new(
            FoundationalProfileResolutionFamily::DiagnosticRichness,
            FoundationalProfileResolutionRelation::Narrowing,
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
        TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::ResolutionLedgerDoesNotMatchProfileChange,
        )
    );
}

#[test]
fn resolution_records_are_canonical_and_survive_materialization() {
    let requested = base_profile(
        ExecutionObjectiveProfile::Balanced,
        ObservationActivationProfile::Continuous,
    );
    let admitted_profile = profile_with_axes(
        requested,
        ExecutionObjectiveProfile::Throughput,
        ObservationActivationProfile::OnDemand,
    );
    let materialized_profile = profile_with_axes(
        admitted_profile,
        ExecutionObjectiveProfile::Throughput,
        ObservationActivationProfile::Continuous,
    );
    let mut reverse = FoundationalProfileResolutionLedger::empty();
    reverse
        .insert(FoundationalProfileResolutionRecord::new(
            FoundationalProfileResolutionFamily::ObservationActivation,
            FoundationalProfileResolutionRelation::ActivationSelection,
            "activation selected",
        ))
        .expect("activation family");
    reverse
        .insert(FoundationalProfileResolutionRecord::new(
            FoundationalProfileResolutionFamily::ExecutionObjective,
            FoundationalProfileResolutionRelation::ObjectiveSelection,
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
            FoundationalProfileResolutionFamily::ExecutionObjective,
            FoundationalProfileResolutionFamily::ObservationActivation,
        ]
    );

    let materialized = match materialize_admitted_foundational_profile_with_resolutions(
        admitted.clone(),
        materialized_profile,
        resolution_ledger(false, true),
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(artifact) => artifact,
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
            .get(FoundationalProfileResolutionFamily::ObservationActivation)
            .expect("activation materialization record")
            .relation(),
        FoundationalProfileResolutionRelation::ActivationSelection
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
        .get(FoundationalProfileResolutionFamily::ExecutionObjective)
        .is_none());

    let omitted = match materialize_admitted_foundational_profile_with_resolutions(
        admitted,
        materialized_profile,
        FoundationalProfileResolutionLedger::empty(),
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected omitted materialization resolution denial, got {other:?}"),
    };
    assert_eq!(
        omitted,
        FoundationalProfileProgressionDenial::ResolutionLedgerDoesNotMatchProfileChange
    );
}
