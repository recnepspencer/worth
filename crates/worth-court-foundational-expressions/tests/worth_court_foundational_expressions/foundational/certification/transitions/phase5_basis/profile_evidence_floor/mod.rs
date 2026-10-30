#[allow(unused_imports)]
use crate::certification::ordinary_context::transitions::phase5_basis::profile_evidence_floor::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::attach_proof_bearing_profiled_commit_receipt,
    worth_foundational::plan_foundational_profile_materialization_with_elision,
    worth_foundational::request_foundational_profile_set,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::ProofBearingArtifactTarget, worth_foundational::RetentionDeliveryProfile,
    worth_foundational::SupportPostureProfile,
};

use {
    crate::certification::ordinary_context::transitions::fixtures::committed::accepted_verdict,
    crate::certification::ordinary_context::transitions::fixtures::committed::committed_authority,
    crate::certification::ordinary_context::transitions::fixtures::committed::ordinary_commit_input,
};
use {
    crate::certification::ordinary_context::transitions::fixtures::receipt::commit_id,
    crate::certification::ordinary_context::transitions::fixtures::receipt::receipt_authority,
    crate::certification::ordinary_context::transitions::fixtures::receipt::receipt_identity,
};
use {
    crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::assert_equivalent,
    crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::ready_receipt,
    crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::ready_receipt_ref,
};

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
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
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
        worth_proof::TransitionOutcome::Success(profiled) => profiled,
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
