use {
    worth_foundational::foundational_profile_progression_authority,
    worth_foundational::materialize_admitted_foundational_profile,
    worth_foundational::plan_foundational_profile_materialization,
    worth_foundational::plan_foundational_profile_materialization_with_elision,
    worth_foundational::plan_selected_foundational_profile_materialization,
    worth_foundational::plan_selected_foundational_profile_materialization_with_disposition,
    worth_foundational::profiles, worth_foundational::ProofBearingArtifactTarget,
};

use {
    crate::certification::ordinary_context::profiles::support::admit_same_profile,
    crate::certification::ordinary_context::profiles::support::profile,
};

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
        worth_proof::TransitionOutcome::Success(artifact) => *artifact.payload(),
        other => panic!("expected materialized profile, got {other:?}"),
    };

    let plan = plan_selected_foundational_profile_materialization_with_disposition::<
        ProofBearingArtifactTarget,
    >(
        &materialized,
        &[worth_foundational::FoundationalDescriptiveSurface::Provenance],
        worth_foundational::FoundationalObservationDisposition::Inactive,
    )
    .expect("selected surface should remain plannable while inactive");
    assert_eq!(
        plan.decision_for(worth_foundational::FoundationalDescriptiveSurface::Provenance)
            .expect("provenance decision")
            .absence_cause(),
        Some(worth_foundational::FoundationalSurfaceAbsenceCause::ObservationNotActivated)
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
        worth_proof::TransitionOutcome::Success(artifact) => *artifact.payload(),
        other => panic!("expected materialized profile, got {other:?}"),
    };

    assert_eq!(
        plan_foundational_profile_materialization::<ProofBearingArtifactTarget>(&materialized),
        Err(worth_foundational::FoundationalMaterializationPlanningDenial::ObservationDispositionRequired)
    );
    assert_eq!(
        plan_foundational_profile_materialization_with_elision::<ProofBearingArtifactTarget>(
            &materialized,
            worth_foundational::FoundationalDescriptiveElisionProfile::OperationalSummary,
        ),
        Err(worth_foundational::FoundationalMaterializationPlanningDenial::ObservationDispositionRequired)
    );
    assert_eq!(
        plan_selected_foundational_profile_materialization::<ProofBearingArtifactTarget>(
            &materialized,
            &[worth_foundational::FoundationalDescriptiveSurface::Provenance],
        ),
        Err(worth_foundational::FoundationalMaterializationPlanningDenial::ObservationDispositionRequired)
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
        worth_proof::TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected boundary attachment, got {other:?}"),
    };
    let selected = [worth_foundational::FoundationalDescriptiveSurface::Provenance];

    assert_eq!(
        profiles()
            .materialization()
            .for_boundary_artifact(&artifact)
            .selected(&selected),
        Err(worth_foundational::FoundationalMaterializationPlanningDenial::ObservationDispositionRequired)
    );
    let inactive = profiles()
        .materialization()
        .for_boundary_artifact(&artifact)
        .selected_with_disposition(
            &selected,
            worth_foundational::FoundationalObservationDisposition::Inactive,
        )
        .expect("explicit inactive disposition should remain available");
    assert_eq!(
        inactive.observation_disposition(),
        worth_foundational::FoundationalObservationDisposition::Inactive
    );
}

#[test]
fn explicit_activation_is_distinct_from_continuous_and_inactive() {
    let activated = worth_foundational::FoundationalObservationDisposition::ExplicitlyActivated {
        scope: worth_foundational::FoundationalObservationActivationScope::Operation,
        session: worth_foundational::BoundaryHandle::new(10),
        observed_epoch: worth_foundational::BoundaryEpoch::new(4),
    };
    assert!(activated.is_active());
    assert_eq!(
        activated.scope(),
        Some(worth_foundational::FoundationalObservationActivationScope::Operation)
    );
    assert!(worth_foundational::FoundationalObservationDisposition::Continuous.is_active());
    assert!(!worth_foundational::FoundationalObservationDisposition::Inactive.is_active());
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
        worth_proof::TransitionOutcome::Success(artifact) => *artifact.payload(),
        other => panic!("expected materialized profile, got {other:?}"),
    };
    let disposition = worth_foundational::FoundationalObservationDisposition::ExplicitlyActivated {
        scope: worth_foundational::FoundationalObservationActivationScope::ManagedSession,
        session: worth_foundational::BoundaryHandle::new(11),
        observed_epoch: worth_foundational::BoundaryEpoch::new(5),
    };
    let plan = plan_selected_foundational_profile_materialization_with_disposition::<
        ProofBearingArtifactTarget,
    >(
        &materialized,
        &[worth_foundational::FoundationalDescriptiveSurface::Provenance],
        disposition,
    )
    .expect("activated selected surface should plan");
    assert_eq!(plan.observation_disposition(), disposition);
    assert!(plan
        .decision_for(worth_foundational::FoundationalDescriptiveSurface::Provenance)
        .expect("provenance decision")
        .is_available());
}
