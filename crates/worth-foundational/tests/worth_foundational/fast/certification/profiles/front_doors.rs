use worth_foundational::{
    profiles, AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, FoundationalDescriptiveSurface, FoundationalProfileNarrowingKind,
    FoundationalProfileNarrowingRecord, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

#[test]
fn profile_front_doors_expose_compose_progress_attach_materialize_and_strengthen() {
    let requested = profiles()
        .set()
        .diagnostic_richness(DiagnosticRichnessProfile::Forensic)
        .support_posture(SupportPostureProfile::CertificationReady)
        .compatibility_posture(CompatibilityPostureProfile::CompatibilityRequired)
        .admission_readiness(AdmissionReadinessProfile::ProductionGateReady)
        .retention_delivery(RetentionDeliveryProfile::Durable)
        .certification_posture(CertificationPostureProfile::ProductionCertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .request()
        .expect("requested profile");

    let admitted = match profiles().progression().admit_same(requested) {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted profile, got {other:?}"),
    };

    let proof_bearing = match profiles().attach().to_proof_bearing_artifact(
        admitted,
        super::support::profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::CertificationReady,
            CompatibilityPostureProfile::CompatibilityRequired,
            AdmissionReadinessProfile::ProductionGateReady,
            RetentionDeliveryProfile::Durable,
            CertificationPostureProfile::ProductionCertified,
        ),
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            "proof-bearing consumers may narrow descriptive richness",
        )),
        "proof payload",
    ) {
        TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected proof-bearing attachment, got {other:?}"),
    };

    let plan = profiles()
        .materialization()
        .for_proof_bearing_artifact(&proof_bearing)
        .selected(&[FoundationalDescriptiveSurface::Provenance])
        .expect("selected proof-bearing materialization plan");
    let evidence_backed = match profiles().certification().evidence_backed(proof_bearing) {
        TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected evidence-backed strengthening"),
    };
    let production = match profiles()
        .certification()
        .production_certified(evidence_backed)
    {
        TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected production-certified strengthening"),
    };
    let bridged = profiles()
        .certification()
        .bridge_production_certified(production);
    let readmitted = profiles()
        .certification()
        .readmit_production_certified(bridged);

    assert_eq!(plan.cost().requested_surface_count(), 1);
    assert_eq!(readmitted.payload(), &"proof payload");
}
