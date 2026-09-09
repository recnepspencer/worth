use crate::certification_support::ScriptedPresentationHost;
use crate::runtime::appearance::{validate_presentation_for_test, UiAppearanceCoherentBasisDenial};
use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_viewport_allocation;
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
};

#[test]
fn coherent_basis_rejects_a_noncurrent_surface_or_epoch() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host)
        .launch_native_surface()
        .expect("currentness fixture should launch");
    let surface = shell.session.mounted.view().surface_bindings()[0].semantic_surface_identity();
    crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut shell.session,
        surface,
        1,
        &[],
    );
    let outcome = match shell.present_frame(100, 1) {
        Ok(outcome) => outcome,
        Err(crate::facade::entry::WorthUiMountedFrameExecutionStop::Preparation(denial)) => {
            panic!("currentness frame preparation: {denial:?}")
        }
        Err(crate::facade::entry::WorthUiMountedFrameExecutionStop::PublicationLease(denial)) => {
            panic!("currentness frame publication lease: {denial:?}")
        }
        Err(crate::facade::entry::WorthUiMountedFrameExecutionStop::HostMeasurement(denial)) => {
            panic!("currentness frame host measurement: {denial:?}")
        }
        Err(crate::facade::entry::WorthUiMountedFrameExecutionStop::HostMeasurementTransition(
            denial,
        )) => panic!("currentness frame host measurement transition: {denial:?}"),
        Err(crate::facade::entry::WorthUiMountedFrameExecutionStop::OccurrenceGeometry(denial)) => {
            panic!("currentness frame occurrence geometry: {denial:?}")
        }
        Err(crate::facade::entry::WorthUiMountedFrameExecutionStop::FrameworkTransition(_)) => {
            panic!("currentness frame framework transition")
        }
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
        validate_presentation_for_test(&shell.session.mounted, surface, Some(current)),
        Ok(())
    );

    let wrong_surface = UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        current.frame(),
        current.binding(),
        current.epoch(),
    );
    assert_eq!(
        validate_presentation_for_test(&shell.session.mounted, surface, Some(wrong_surface)),
        Err(UiAppearanceCoherentBasisDenial::PresentationNotCurrent)
    );
    let wrong_epoch = UiHostObservationPresentationBasis::new(
        current.host_surface(),
        current.frame(),
        current.binding(),
        UiHostPresentationEpoch::issued_by_host(2),
    );
    assert_eq!(
        validate_presentation_for_test(&shell.session.mounted, surface, Some(wrong_epoch)),
        Err(UiAppearanceCoherentBasisDenial::PresentationNotCurrent)
    );
    let foreign_surface = shell.session.create_semantic_surface().unwrap();
    assert_eq!(
        validate_presentation_for_test(&shell.session.mounted, foreign_surface, Some(current)),
        Err(UiAppearanceCoherentBasisDenial::PresentationNotCurrent)
    );
    let _ = shell.shutdown();
}
