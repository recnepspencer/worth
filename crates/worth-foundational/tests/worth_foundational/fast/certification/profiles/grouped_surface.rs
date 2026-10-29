use worth_foundational::{
    profiles_api::{
        common_path,
        lower_lane::{attachment, certification, composition, materialization, progression},
        stronger_lane::readiness as stronger_readiness,
    },
    FoundationalDescriptiveSurface,
};
use worth_proof::TransitionOutcome;

#[test]
fn grouped_profile_surface_exposes_common_lower_and_stronger_lanes() {
    let requested = common_path::profiles()
        .set()
        .diagnostic_richness(composition::DiagnosticRichnessProfile::Forensic)
        .support_posture(composition::SupportPostureProfile::CertificationReady)
        .compatibility_posture(composition::CompatibilityPostureProfile::CompatibilityRequired)
        .admission_readiness(composition::AdmissionReadinessProfile::ProductionGateReady)
        .retention_delivery(composition::RetentionDeliveryProfile::Durable)
        .certification_posture(composition::CertificationPostureProfile::ProductionCertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .request()
        .expect("requested profile");

    let admitted = match common_path::profiles().progression().admit_same(requested) {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted profile, got {other:?}"),
    };
    let proof_bearing = match common_path::profiles().attach().to_proof_bearing_artifact(
        admitted,
        super::support::profile(
            composition::DiagnosticRichnessProfile::Standard,
            composition::SupportPostureProfile::CertificationReady,
            composition::CompatibilityPostureProfile::CompatibilityRequired,
            composition::AdmissionReadinessProfile::ProductionGateReady,
            composition::RetentionDeliveryProfile::Durable,
            composition::CertificationPostureProfile::ProductionCertified,
        ),
        Some(progression::FoundationalProfileNarrowingRecord::new(
            progression::FoundationalProfileNarrowingKind::RichnessReduced,
            "common lane narrows descriptive richness before materialization",
        )),
        "grouped payload",
    ) {
        TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected common-path attachment, got {other:?}"),
    };
    let common_plan = common_path::profiles()
        .materialization()
        .for_proof_bearing_artifact(&proof_bearing)
        .selected(&[FoundationalDescriptiveSurface::Provenance])
        .expect("common-path materialization plan");

    let lower_requested = progression::request_foundational_profile_set(super::support::profile(
        composition::DiagnosticRichnessProfile::Forensic,
        composition::SupportPostureProfile::CertificationReady,
        composition::CompatibilityPostureProfile::CompatibilityRequired,
        composition::AdmissionReadinessProfile::ProductionGateReady,
        composition::RetentionDeliveryProfile::Durable,
        composition::CertificationPostureProfile::ProductionCertified,
    ));
    let lower_admitted = match progression::admit_requested_foundational_profile(
        lower_requested,
        super::support::profile(
            composition::DiagnosticRichnessProfile::Forensic,
            composition::SupportPostureProfile::CertificationReady,
            composition::CompatibilityPostureProfile::CompatibilityRequired,
            composition::AdmissionReadinessProfile::ProductionGateReady,
            composition::RetentionDeliveryProfile::Durable,
            composition::CertificationPostureProfile::ProductionCertified,
        ),
        None,
        progression::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected lower-lane admitted profile, got {other:?}"),
    };
    let lower_attachment = match attachment::attach_proof_bearing_profiled_artifact(
        lower_admitted,
        super::support::profile(
            composition::DiagnosticRichnessProfile::Standard,
            composition::SupportPostureProfile::CertificationReady,
            composition::CompatibilityPostureProfile::CompatibilityRequired,
            composition::AdmissionReadinessProfile::ProductionGateReady,
            composition::RetentionDeliveryProfile::Durable,
            composition::CertificationPostureProfile::ProductionCertified,
        ),
        Some(progression::FoundationalProfileNarrowingRecord::new(
            progression::FoundationalProfileNarrowingKind::RichnessReduced,
            "lower lane narrows descriptive richness before proof-bearing attachment",
        )),
        "grouped payload",
        progression::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected lower-lane attachment, got {other:?}"),
    };
    let lower_plan = materialization::plan_selected_foundational_profile_materialization::<
        attachment::ProofBearingArtifactTarget,
    >(
        lower_attachment.payload().profile(),
        &[FoundationalDescriptiveSurface::Provenance],
    )
    .expect("lower-lane materialization plan");
    let lower_evidence_backed = match certification::certify_evidence_backed_proof_bearing_artifact(
        lower_attachment,
        certification::foundational_profile_certification_authority(),
    ) {
        TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected lower-lane evidence-backed strengthening"),
    };
    let lower_production = match certification::certify_production_certified_proof_bearing_artifact(
        lower_evidence_backed,
        certification::foundational_profile_certification_authority(),
    ) {
        TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected lower-lane production-certified strengthening"),
    };
    let report = stronger_readiness::foundational_profile_milestone3_readiness_report();
    let certified =
        stronger_readiness::certify_foundational_profile_milestone3_production_test_readiness();

    assert_eq!(common_plan.cost().requested_surface_count(), 1);
    assert_eq!(lower_plan.cost().requested_surface_count(), 1);
    assert_eq!(lower_production.payload(), &"grouped payload");
    assert!(report.passes_readiness_checklist());
    assert!(std::ptr::eq(
        stronger_readiness::require_foundational_profile_milestone3_production_test_readiness(
            &certified
        ),
        certified.payload()
    ));
}
