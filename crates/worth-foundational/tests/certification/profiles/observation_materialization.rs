use worth_foundational::{
    foundational_profile_progression_authority, materialize_admitted_foundational_profile,
    plan_foundational_profile_materialization,
    plan_foundational_profile_materialization_with_elision,
    plan_selected_foundational_profile_materialization,
    plan_selected_foundational_profile_materialization_with_disposition, profiles,
    FoundationalDescriptiveElisionProfile, FoundationalDescriptiveSurface,
    FoundationalMaterializationPlanningDenial, FoundationalObservationDisposition,
    FoundationalSurfaceAbsenceCause, ProofBearingArtifactTarget,
};
use worth_proof::TransitionOutcome;

use super::support::{admit_same_profile, profile};

#[test]
fn inactive_observation_is_an_explicit_materialization_absence() {
    let materialized = profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::ProductionCertified,
    );
    let admitted = admit_same_profile(materialized);
    let materialized = match materialize_admitted_foundational_profile(
        admitted,
        materialized,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(artifact) => *artifact.payload(),
        other => panic!("expected materialized profile, got {other:?}"),
    };

    let plan = plan_selected_foundational_profile_materialization_with_disposition::<
        ProofBearingArtifactTarget,
    >(
        &materialized,
        &[FoundationalDescriptiveSurface::Provenance],
        FoundationalObservationDisposition::Inactive,
    )
    .expect("selected surface should remain plannable while inactive");
    assert_eq!(
        plan.decision_for(FoundationalDescriptiveSurface::Provenance)
            .expect("provenance decision")
            .absence_cause(),
        Some(FoundationalSurfaceAbsenceCause::ObservationNotActivated)
    );
}

#[test]
fn profile_only_materialization_requires_actual_disposition_for_on_demand() {
    let on_demand = worth_foundational::FoundationalProfileSet::new(
        worth_foundational::FoundationalProfileSetInput {
            diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Forensic,
            support_posture: worth_foundational::SupportPostureProfile::CertificationReady,
            compatibility_posture:
                worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
            admission_readiness: worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
            retention_delivery: worth_foundational::RetentionDeliveryProfile::Durable,
            certification_posture:
                worth_foundational::CertificationPostureProfile::ProductionCertified,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Throughput,
            observation_activation: worth_foundational::ObservationActivationProfile::OnDemand,
        },
    )
    .expect("on-demand profile should compose");
    let admitted = admit_same_profile(on_demand);
    let materialized = match materialize_admitted_foundational_profile(
        admitted,
        on_demand,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(artifact) => *artifact.payload(),
        other => panic!("expected materialized profile, got {other:?}"),
    };

    assert_eq!(
        plan_foundational_profile_materialization::<ProofBearingArtifactTarget>(&materialized),
        Err(FoundationalMaterializationPlanningDenial::ObservationDispositionRequired)
    );
    assert_eq!(
        plan_foundational_profile_materialization_with_elision::<ProofBearingArtifactTarget>(
            &materialized,
            FoundationalDescriptiveElisionProfile::OperationalSummary,
        ),
        Err(FoundationalMaterializationPlanningDenial::ObservationDispositionRequired)
    );
    assert_eq!(
        plan_selected_foundational_profile_materialization::<ProofBearingArtifactTarget>(
            &materialized,
            &[FoundationalDescriptiveSurface::Provenance],
        ),
        Err(FoundationalMaterializationPlanningDenial::ObservationDispositionRequired)
    );
}

#[test]
fn profile_materialization_front_door_preserves_disposition_boundary() {
    let on_demand = worth_foundational::FoundationalProfileSet::new(
        worth_foundational::FoundationalProfileSetInput {
            diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Forensic,
            support_posture: worth_foundational::SupportPostureProfile::CertificationReady,
            compatibility_posture:
                worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
            admission_readiness: worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
            retention_delivery: worth_foundational::RetentionDeliveryProfile::Durable,
            certification_posture:
                worth_foundational::CertificationPostureProfile::ProductionCertified,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Throughput,
            observation_activation: worth_foundational::ObservationActivationProfile::OnDemand,
        },
    )
    .expect("on-demand profile should compose");
    let admitted = admit_same_profile(on_demand);
    let artifact = match profiles().attach().to_boundary_artifact(
        admitted,
        on_demand,
        None,
        "boundary payload",
    ) {
        TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected boundary attachment, got {other:?}"),
    };
    let selected = [FoundationalDescriptiveSurface::Provenance];

    assert_eq!(
        profiles()
            .materialization()
            .for_boundary_artifact(&artifact)
            .selected(&selected),
        Err(FoundationalMaterializationPlanningDenial::ObservationDispositionRequired)
    );
    let inactive = profiles()
        .materialization()
        .for_boundary_artifact(&artifact)
        .selected_with_disposition(&selected, FoundationalObservationDisposition::Inactive)
        .expect("explicit inactive disposition should remain available");
    assert_eq!(
        inactive.observation_disposition(),
        FoundationalObservationDisposition::Inactive
    );
}

#[test]
fn explicit_activation_is_distinct_from_continuous_and_inactive() {
    let activated = FoundationalObservationDisposition::ExplicitlyActivated {
        scope: worth_foundational::FoundationalObservationActivationScope::Operation,
        session: worth_foundational::BoundaryHandle::new(10),
        observed_epoch: worth_foundational::BoundaryEpoch::new(4),
    };
    assert!(activated.is_active());
    assert_eq!(
        activated.scope(),
        Some(worth_foundational::FoundationalObservationActivationScope::Operation)
    );
    assert!(FoundationalObservationDisposition::Continuous.is_active());
    assert!(!FoundationalObservationDisposition::Inactive.is_active());
}

#[test]
fn explicit_activation_survives_materialization_and_allows_selected_surface() {
    let profile = profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::ProductionCertified,
    );
    let admitted = admit_same_profile(profile);
    let materialized = match materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(artifact) => *artifact.payload(),
        other => panic!("expected materialized profile, got {other:?}"),
    };
    let disposition = FoundationalObservationDisposition::ExplicitlyActivated {
        scope: worth_foundational::FoundationalObservationActivationScope::ManagedSession,
        session: worth_foundational::BoundaryHandle::new(11),
        observed_epoch: worth_foundational::BoundaryEpoch::new(5),
    };
    let plan = plan_selected_foundational_profile_materialization_with_disposition::<
        ProofBearingArtifactTarget,
    >(
        &materialized,
        &[FoundationalDescriptiveSurface::Provenance],
        disposition,
    )
    .expect("activated selected surface should plan");
    assert_eq!(plan.observation_disposition(), disposition);
    assert!(plan
        .decision_for(FoundationalDescriptiveSurface::Provenance)
        .expect("provenance decision")
        .is_available());
}
