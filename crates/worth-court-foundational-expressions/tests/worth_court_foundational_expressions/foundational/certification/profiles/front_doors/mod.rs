use worth_foundational::profiles;

#[test]
fn profile_front_doors_expose_compose_progress_attach_materialize_and_strengthen() {
    let requested = profiles()
        .set()
        .diagnostic_richness(worth_foundational::DiagnosticRichnessProfile::Forensic)
        .support_posture(worth_foundational::SupportPostureProfile::CertificationReady)
        .compatibility_posture(
            worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        )
        .admission_readiness(worth_foundational::AdmissionReadinessProfile::ProductionGateReady)
        .retention_delivery(worth_foundational::RetentionDeliveryProfile::Durable)
        .certification_posture(worth_foundational::CertificationPostureProfile::ProductionCertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .request()
        .expect("requested profile");

    let admitted = match profiles().progression().admit_same(requested) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted profile, got {other:?}"),
    };

    let proof_bearing = match profiles().attach().to_proof_bearing_artifact(
        admitted,
        crate::certification::ordinary_context::profiles::support::profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::CertificationReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
            worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
            worth_foundational::RetentionDeliveryProfile::Durable,
            worth_foundational::CertificationPostureProfile::ProductionCertified,
        ),
        Some(worth_foundational::FoundationalProfileNarrowingRecord::new(
            worth_foundational::FoundationalProfileNarrowingKind::RichnessReduced,
            "proof-bearing consumers may narrow descriptive richness",
        )),
        "proof payload",
    ) {
        worth_proof::TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected proof-bearing attachment, got {other:?}"),
    };

    let plan = profiles()
        .materialization()
        .for_proof_bearing_artifact(&proof_bearing)
        .selected(&[worth_foundational::FoundationalDescriptiveSurface::Provenance])
        .expect("selected proof-bearing materialization plan");
    let evidence_backed = match profiles().certification().evidence_backed(proof_bearing) {
        worth_proof::TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected evidence-backed strengthening"),
    };
    let production = match profiles()
        .certification()
        .production_certified(evidence_backed)
    {
        worth_proof::TransitionOutcome::Success(certified) => certified,
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
