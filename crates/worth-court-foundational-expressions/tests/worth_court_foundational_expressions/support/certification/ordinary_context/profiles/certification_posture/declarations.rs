pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::admit_requested_foundational_profile,
    worth_foundational::attach_proof_bearing_profiled_artifact,
    worth_foundational::bridge_evidence_backed_proof_bearing_artifact_trust_boundary,
    worth_foundational::bridge_production_certified_proof_bearing_artifact_trust_boundary,
    worth_foundational::certify_evidence_backed_proof_bearing_artifact,
    worth_foundational::certify_production_certified_proof_bearing_artifact,
    worth_foundational::foundational_profile_certification_authority,
    worth_foundational::foundational_profile_certification_proof_lane,
    worth_foundational::foundational_profile_certification_readmission_authority,
    worth_foundational::foundational_profile_progression_authority,
    worth_foundational::readmit_evidence_backed_proof_bearing_artifact_after_boundary,
    worth_foundational::readmit_production_certified_proof_bearing_artifact_after_boundary,
    worth_foundational::request_foundational_profile_set,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalProfileCertificationDenial,
    worth_foundational::FoundationalProfileCertificationProofLane,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

pub(crate) fn profile(
    support_posture: SupportPostureProfile,
    admission_readiness: AdmissionReadinessProfile,
    retention_delivery: RetentionDeliveryProfile,
    certification_posture: CertificationPostureProfile,
) -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Forensic,
        support_posture,
        compatibility_posture:
            worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        admission_readiness,
        retention_delivery,
        certification_posture,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("coherent profile")
}

pub(crate) fn proof_bearing_payload(
    profile: FoundationalProfileSet,
) -> worth_foundational::ProofBearingProfiledArtifact<&'static str> {
    let requested = request_foundational_profile_set(profile);
    let admitted = match admit_requested_foundational_profile(
        requested,
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted profile, got {outcome:?}"),
    };

    match attach_proof_bearing_profiled_artifact(
        admitted,
        profile,
        None,
        "payload",
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(profiled) => profiled,
        outcome => panic!("expected proof-bearing attachment, got {outcome:?}"),
    }
}
