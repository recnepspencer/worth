use super::*;

#[test]
fn wrong_host_cannot_update_a_retained_binding_epoch() {
    use crate::mounting::presentation::presented_surface_witness_for_certification as witness;
    use worth_ui_host_contract::*;
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let host_surface = UiHostSurfaceIdentity::mint_unbound().unwrap();
    let first = UiHostObservationPresentationBasis::new(
        host_surface,
        frame,
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let mut retained = UiRetainedPresentationBinding {
        surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        displayed: witness(first).displayed_basis(),
    };
    let before = retained;
    let wrong = UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        frame,
        binding,
        UiHostPresentationEpoch::issued_by_host(2),
    );
    assert_eq!(
        retained.update_epoch(witness(wrong).displayed_basis()),
        Err(UiPresentedFrameBasisDenial::BindingNotPresented)
    );
    assert_eq!(retained, before);
    let right =
        UiHostObservationPresentationBasis::new(host_surface, frame, binding, wrong.epoch());
    retained
        .update_epoch(witness(right).displayed_basis())
        .unwrap();
    assert_eq!(retained.displayed.basis(), right);
}
