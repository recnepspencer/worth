pub use worth_proof::TransitionOutcome;
pub use {
    worth_foundational::attach_proof_bearing_profiled_commit_receipt,
    worth_foundational::plan_foundational_profile_materialization_with_elision,
    worth_foundational::request_foundational_profile_set,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::ProofBearingArtifactTarget, worth_foundational::RetentionDeliveryProfile,
    worth_foundational::SupportPostureProfile,
};

pub use {
    crate::certification::ordinary_context::transitions::fixtures::committed::accepted_verdict,
    crate::certification::ordinary_context::transitions::fixtures::committed::committed_authority,
    crate::certification::ordinary_context::transitions::fixtures::committed::ordinary_commit_input,
};
pub use {
    crate::certification::ordinary_context::transitions::fixtures::receipt::commit_id,
    crate::certification::ordinary_context::transitions::fixtures::receipt::receipt_authority,
    crate::certification::ordinary_context::transitions::fixtures::receipt::receipt_identity,
};
pub use {
    crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::assert_equivalent,
    crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::ready_receipt,
    crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::ready_receipt_ref,
};

pub fn profile() -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Forensic,
        support_posture: worth_foundational::SupportPostureProfile::CertificationReady,
        compatibility_posture:
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
        certification_posture: worth_foundational::CertificationPostureProfile::ProductionCertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("coherent profile")
}
