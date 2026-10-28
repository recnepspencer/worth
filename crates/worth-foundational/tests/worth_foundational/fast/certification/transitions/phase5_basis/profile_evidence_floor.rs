use worth_foundational::{
    attach_proof_bearing_profiled_commit_receipt,
    plan_foundational_profile_materialization_with_elision, request_foundational_profile_set,
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, FoundationalProfileSet, FoundationalProfileSetInput,
    ProofBearingArtifactTarget, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::super::fixtures::committed::{
    accepted_verdict, committed_authority, ordinary_commit_input,
};
use super::super::fixtures::receipt::{commit_id, receipt_authority, receipt_identity};
use super::canonical_basis::{assert_equivalent, ready_receipt, ready_receipt_ref};

#[test]
fn profile_attachment_and_reduced_richness_do_not_weaken_receipt_evidence_floor() {
    let profile = profile();
    let requested = request_foundational_profile_set(profile);
    let admitted = match worth_foundational::admit_requested_foundational_profile(
        requested,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        _ => panic!("expected admitted profile"),
    };
    let receipt = accepted_verdict("mesh-update")
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority")
        .issue_receipt(receipt_identity(77), commit_id(66), receipt_authority())
        .expect("receipt");
    let original_basis = ready_receipt(
        accepted_verdict("mesh-update")
            .commit_with(ordinary_commit_input(), committed_authority())
            .expect("committed authority")
            .issue_receipt(receipt_identity(77), commit_id(66), receipt_authority())
            .expect("receipt"),
    );

    let profiled = match attach_proof_bearing_profiled_commit_receipt(
        admitted,
        profile,
        None,
        receipt,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(profiled) => profiled,
        _ => panic!("expected profiled receipt"),
    };
    let attached_basis = ready_receipt_ref(profiled.payload().payload());
    assert_equivalent(original_basis, attached_basis);

    let plan =
        plan_foundational_profile_materialization_with_elision::<ProofBearingArtifactTarget>(
            profiled.payload().profile(),
            worth_foundational::FoundationalDescriptiveElisionProfile::OperationalSummary,
        )
        .expect("continuous profile should carry its default disposition");
    assert!(plan
        .decision_for(worth_foundational::FoundationalDescriptiveSurface::Provenance)
        .expect("proof-bearing provenance decision")
        .is_available());
    assert_eq!(profiled.payload().payload().commit_id(), commit_id(66));
    assert_eq!(
        profiled.payload().payload().receipt_identity(),
        receipt_identity(77)
    );
}

fn profile() -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Forensic,
        support_posture: SupportPostureProfile::CertificationReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: AdmissionReadinessProfile::ProductionGateReady,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::ProductionCertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("coherent profile")
}
