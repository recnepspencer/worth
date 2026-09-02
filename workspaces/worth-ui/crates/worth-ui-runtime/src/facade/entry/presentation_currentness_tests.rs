use crate::certification_support::ScriptedPresentationHost;
use crate::runtime::appearance::{validate_presentation_for_test, UiAppearanceCoherentBasisDenial};
use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host;
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
};

#[test]
fn coherent_basis_rejects_a_noncurrent_surface_or_epoch() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_component_app_with_host(host)
        .launch_native_surface()
        .expect("currentness fixture should launch");
    let outcome = match shell.present_frame(100, 1) {
        Ok(outcome) => outcome,
        Err(_) => panic!("currentness frame should execute"),
    };
    let publication = match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(receipt)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(receipt)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(receipt) => receipt,
        _ => panic!("currentness fixture should publish"),
    };
    let binding = *publication.bindings().first().expect("native binding");
    let host_surface = shell.session.mounted.view().surface_bindings()[0].host_surface_identity();
    let current = UiHostObservationPresentationBasis::new(
        host_surface,
        publication.frame(),
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    assert_eq!(
        validate_presentation_for_test(&shell.session.mounted, Some(current)),
        Ok(())
    );

    let wrong_surface = UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        current.frame(),
        current.binding(),
        current.epoch(),
    );
    assert_eq!(
        validate_presentation_for_test(&shell.session.mounted, Some(wrong_surface)),
        Err(UiAppearanceCoherentBasisDenial::PresentationNotCurrent)
    );
    let wrong_epoch = UiHostObservationPresentationBasis::new(
        current.host_surface(),
        current.frame(),
        current.binding(),
        UiHostPresentationEpoch::issued_by_host(2),
    );
    assert_eq!(
        validate_presentation_for_test(&shell.session.mounted, Some(wrong_epoch)),
        Err(UiAppearanceCoherentBasisDenial::PresentationNotCurrent)
    );
    let _ = shell.shutdown();
}
