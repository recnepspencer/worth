use worth_proof::TransitionOutcome;
use {
    worth_foundational::profiles_api::common_path,
    worth_foundational::profiles_api::lower_lane::attachment,
    worth_foundational::profiles_api::lower_lane::certification,
    worth_foundational::profiles_api::lower_lane::composition,
    worth_foundational::profiles_api::lower_lane::materialization,
    worth_foundational::profiles_api::lower_lane::progression,
    worth_foundational::profiles_api::stronger_lane::readiness as stronger_readiness,
    worth_foundational::FoundationalDescriptiveSurface,
};

#[test]
fn grouped_profile_surface_exposes_common_lower_and_stronger_lanes() {
    let requested = worth_foundational::profiles_api::common_path::profiles()
        .set()
        .diagnostic_richness(worth_foundational::profiles_api::lower_lane::composition::DiagnosticRichnessProfile::Forensic)
        .support_posture(worth_foundational::profiles_api::lower_lane::composition::SupportPostureProfile::CertificationReady)
        .compatibility_posture(worth_foundational::profiles_api::lower_lane::composition::CompatibilityPostureProfile::CompatibilityRequired)
        .admission_readiness(worth_foundational::profiles_api::lower_lane::composition::AdmissionReadinessProfile::ProductionGateReady)
        .retention_delivery(worth_foundational::profiles_api::lower_lane::composition::RetentionDeliveryProfile::Durable)
        .certification_posture(worth_foundational::profiles_api::lower_lane::composition::CertificationPostureProfile::ProductionCertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .request()
        .expect("requested profile");

    let admitted = match worth_foundational::profiles_api::common_path::profiles()
        .progression()
        .admit_same(requested)
    {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted profile, got {other:?}"),
    };
    let proof_bearing = match worth_foundational::profiles_api::common_path::profiles().attach().to_proof_bearing_artifact(
        admitted,
        crate::certification::ordinary_context::profiles::support::profile(
            worth_foundational::profiles_api::lower_lane::composition::DiagnosticRichnessProfile::Standard,
            worth_foundational::profiles_api::lower_lane::composition::SupportPostureProfile::CertificationReady,
            worth_foundational::profiles_api::lower_lane::composition::CompatibilityPostureProfile::CompatibilityRequired,
            worth_foundational::profiles_api::lower_lane::composition::AdmissionReadinessProfile::ProductionGateReady,
            worth_foundational::profiles_api::lower_lane::composition::RetentionDeliveryProfile::Durable,
            worth_foundational::profiles_api::lower_lane::composition::CertificationPostureProfile::ProductionCertified,
        ),
        Some(worth_foundational::profiles_api::lower_lane::progression::FoundationalProfileNarrowingRecord::new(
            worth_foundational::profiles_api::lower_lane::progression::FoundationalProfileNarrowingKind::RichnessReduced,
            "common lane narrows descriptive richness before materialization",
        )),
        "grouped payload",
    ) {
        worth_proof::TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected common-path attachment, got {other:?}"),
    };
    let common_plan = worth_foundational::profiles_api::common_path::profiles()
        .materialization()
        .for_proof_bearing_artifact(&proof_bearing)
        .selected(&[worth_foundational::FoundationalDescriptiveSurface::Provenance])
        .expect("common-path materialization plan");

    let lower_requested = worth_foundational::profiles_api::lower_lane::progression::request_foundational_profile_set(crate::certification::ordinary_context::profiles::support::profile(
        worth_foundational::profiles_api::lower_lane::composition::DiagnosticRichnessProfile::Forensic,
        worth_foundational::profiles_api::lower_lane::composition::SupportPostureProfile::CertificationReady,
        worth_foundational::profiles_api::lower_lane::composition::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::profiles_api::lower_lane::composition::AdmissionReadinessProfile::ProductionGateReady,
        worth_foundational::profiles_api::lower_lane::composition::RetentionDeliveryProfile::Durable,
        worth_foundational::profiles_api::lower_lane::composition::CertificationPostureProfile::ProductionCertified,
    ));
    let lower_admitted = match worth_foundational::profiles_api::lower_lane::progression::admit_requested_foundational_profile(
        lower_requested,
        crate::certification::ordinary_context::profiles::support::profile(
            worth_foundational::profiles_api::lower_lane::composition::DiagnosticRichnessProfile::Forensic,
            worth_foundational::profiles_api::lower_lane::composition::SupportPostureProfile::CertificationReady,
            worth_foundational::profiles_api::lower_lane::composition::CompatibilityPostureProfile::CompatibilityRequired,
            worth_foundational::profiles_api::lower_lane::composition::AdmissionReadinessProfile::ProductionGateReady,
            worth_foundational::profiles_api::lower_lane::composition::RetentionDeliveryProfile::Durable,
            worth_foundational::profiles_api::lower_lane::composition::CertificationPostureProfile::ProductionCertified,
        ),
        None,
        worth_foundational::profiles_api::lower_lane::progression::foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected lower-lane admitted profile, got {other:?}"),
    };
    let lower_attachment = match worth_foundational::profiles_api::lower_lane::attachment::attach_proof_bearing_profiled_artifact(
        lower_admitted,
        crate::certification::ordinary_context::profiles::support::profile(
            worth_foundational::profiles_api::lower_lane::composition::DiagnosticRichnessProfile::Standard,
            worth_foundational::profiles_api::lower_lane::composition::SupportPostureProfile::CertificationReady,
            worth_foundational::profiles_api::lower_lane::composition::CompatibilityPostureProfile::CompatibilityRequired,
            worth_foundational::profiles_api::lower_lane::composition::AdmissionReadinessProfile::ProductionGateReady,
            worth_foundational::profiles_api::lower_lane::composition::RetentionDeliveryProfile::Durable,
            worth_foundational::profiles_api::lower_lane::composition::CertificationPostureProfile::ProductionCertified,
        ),
        Some(worth_foundational::profiles_api::lower_lane::progression::FoundationalProfileNarrowingRecord::new(
            worth_foundational::profiles_api::lower_lane::progression::FoundationalProfileNarrowingKind::RichnessReduced,
            "lower lane narrows descriptive richness before proof-bearing attachment",
        )),
        "grouped payload",
        worth_foundational::profiles_api::lower_lane::progression::foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected lower-lane attachment, got {other:?}"),
    };
    let lower_plan = worth_foundational::profiles_api::lower_lane::materialization::plan_selected_foundational_profile_materialization::<
        worth_foundational::profiles_api::lower_lane::attachment::ProofBearingArtifactTarget,
    >(
        lower_attachment.payload().profile(),
        &[worth_foundational::FoundationalDescriptiveSurface::Provenance],
    )
    .expect("lower-lane materialization plan");
    let lower_evidence_backed = match worth_foundational::profiles_api::lower_lane::certification::certify_evidence_backed_proof_bearing_artifact(
        lower_attachment,
        worth_foundational::profiles_api::lower_lane::certification::foundational_profile_certification_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected lower-lane evidence-backed strengthening"),
    };
    let lower_production = match worth_foundational::profiles_api::lower_lane::certification::certify_production_certified_proof_bearing_artifact(
        lower_evidence_backed,
        worth_foundational::profiles_api::lower_lane::certification::foundational_profile_certification_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected lower-lane production-certified strengthening"),
    };
    let report = worth_foundational::profiles_api::stronger_lane::readiness::foundational_profile_milestone3_readiness_report();
    let certified =
        worth_foundational::profiles_api::stronger_lane::readiness::certify_foundational_profile_milestone3_production_test_readiness();

    assert_eq!(common_plan.cost().requested_surface_count(), 1);
    assert_eq!(lower_plan.cost().requested_surface_count(), 1);
    assert_eq!(lower_production.payload(), &"grouped payload");
    assert!(report.passes_readiness_checklist());
    assert!(std::ptr::eq(
        worth_foundational::profiles_api::stronger_lane::readiness::require_foundational_profile_milestone3_production_test_readiness(
            &certified
        ),
        certified.payload()
    ));
}
