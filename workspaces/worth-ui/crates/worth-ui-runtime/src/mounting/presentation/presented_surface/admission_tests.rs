//! A host acknowledgement becomes displayed truth only when it answers work this
//! runtime issued under its live lease: the exact attempt, requirement, and frame,
//! in the issued mode, with the issued effects.

use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationCostReport, UiHostPresentationEpoch,
    UiHostSurfaceIdentity, UiHostSurfacePresentationMode, UiMountedCompletedEffects,
    UiMountedFrameConsumptionView, UiMountedFrameIdentity, UiMountedPresentationAttemptIdentity,
    UiMountedSurfaceBindingRequirement, UiMountedSurfacePresentationCompletion,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

use super::certification_witness::{
    acknowledged_for_certification, UiAcknowledgedCertificationWork, CERTIFICATION_EFFECTS,
};
use super::{UiIssuedSurfacePresentation, UiPresentedSurfaceWitness};

fn basis() -> UiHostObservationPresentationBasis {
    UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        UiHostPresentationEpoch::issued_by_host(3),
    )
}

#[expect(
    clippy::disallowed_methods,
    reason = "the test acknowledges its issued work the way a host does"
)]
fn acknowledged(
    basis: UiHostObservationPresentationBasis,
    mode: UiHostSurfacePresentationMode,
) -> UiAcknowledgedCertificationWork {
    acknowledged_for_certification(
        UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        basis,
        |view: &UiMountedFrameConsumptionView<'_>| {
            view.acknowledge_presented(
                mode,
                basis.epoch(),
                UiMountedCompletedEffects::new(CERTIFICATION_EFFECTS.to_vec()),
                UiHostPresentationCostReport::default(),
            )
        },
    )
}

fn issued(
    attempt: UiMountedPresentationAttemptIdentity,
    requirement: UiMountedSurfaceBindingRequirement,
    frame: UiMountedFrameIdentity,
) -> UiIssuedSurfacePresentation<'static> {
    UiIssuedSurfacePresentation {
        attempt,
        requirement,
        frame,
        expected_effects: &CERTIFICATION_EFFECTS,
    }
}

fn admits(
    completion: UiMountedSurfacePresentationCompletion,
    authority: &crate::host::adapter::UiHostAdapterSessionAuthority,
    issued: UiIssuedSurfacePresentation<'_>,
) -> bool {
    UiPresentedSurfaceWitness::admit(completion, authority, issued).is_ok()
}

#[test]
fn the_acknowledgement_of_the_exact_issued_work_is_displayed_truth() {
    let basis = basis();
    let witness = acknowledged(basis, UiHostSurfacePresentationMode::RecordOnly)
        .admit(basis.frame(), &CERTIFICATION_EFFECTS)
        .unwrap_or_else(|_| panic!("the issuing session admits its own work"));
    assert_eq!(witness.displayed_basis().basis(), basis);
}

#[test]
fn another_sessions_live_lease_cannot_admit_an_acknowledgement() {
    let basis = basis();
    let ours = acknowledged(basis, UiHostSurfacePresentationMode::RecordOnly);
    let other = acknowledged(basis, UiHostSurfacePresentationMode::RecordOnly);
    assert!(!admits(
        ours.completion,
        &other.authority,
        issued(ours.attempt, ours.requirement, basis.frame())
    ));
}

#[test]
fn a_released_lease_admits_nothing_it_issued_even_after_a_new_claim() {
    let basis = basis();
    let UiAcknowledgedCertificationWork {
        authority,
        lease,
        attempt,
        requirement,
        completion,
    } = acknowledged(basis, UiHostSurfacePresentationMode::RecordOnly);
    drop(lease);
    let _successor = authority
        .claim_mounted_presentation_lease()
        .expect("a released lease can be claimed again");
    assert!(!admits(
        completion,
        &authority,
        issued(attempt, requirement, basis.frame())
    ));
}

#[test]
fn an_acknowledgement_answers_only_its_own_attempt_requirement_and_frame() {
    let basis = basis();
    let other_attempt = UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let other_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    for mismatch in 0..3 {
        let UiAcknowledgedCertificationWork {
            authority,
            lease: _lease,
            attempt,
            requirement,
            completion,
        } = acknowledged(basis, UiHostSurfacePresentationMode::RecordOnly);
        let other_requirement = UiMountedSurfaceBindingRequirement::new(
            requirement.semantic_surface(),
            requirement.host_surface(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            requirement.capability_generation(),
            requirement.capability_profile_digest(),
            requirement.presentation_mode(),
        );
        let admitted = match mismatch {
            0 => admits(
                completion,
                &authority,
                issued(other_attempt, requirement, basis.frame()),
            ),
            1 => admits(
                completion,
                &authority,
                issued(attempt, other_requirement, basis.frame()),
            ),
            _ => admits(
                completion,
                &authority,
                issued(attempt, requirement, other_frame),
            ),
        };
        assert!(
            !admitted,
            "mismatch {mismatch} must not become displayed truth"
        );
    }
}

#[test]
fn an_acknowledgement_must_meet_the_issued_mode_and_effects() {
    let basis = basis();
    assert!(
        acknowledged(basis, UiHostSurfacePresentationMode::NativeDisplay)
            .admit(basis.frame(), &CERTIFICATION_EFFECTS)
            .is_err(),
        "record-only work cannot be acknowledged as a native display"
    );
    assert!(
        acknowledged(basis, UiHostSurfacePresentationMode::RecordOnly)
            .admit(basis.frame(), &[])
            .is_err(),
        "a host cannot claim effects the work never issued"
    );
}
