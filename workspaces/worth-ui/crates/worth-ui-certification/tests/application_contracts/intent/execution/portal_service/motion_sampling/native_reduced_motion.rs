use worth_ui::facade::{
    app::{WorthUiNativeIntentTransition, WorthUiNativeManagedIntentConsequencePublicationOutcome},
    intent::{
        UiIntentDefinition, UiIntentExecutionAdvanceOutcome, UiIntentRuntimeServiceDestination,
    },
};
use worth_ui_runtime::facade::mounted::UiMountedFrameOutcome;

use crate::intent::operability::{build_open_portal_application_with_host, PrimaryIntent};

#[test]
fn native_reduced_motion_admission_snaps_real_portal_entrance_and_stops_sampling() {
    let host = native_portal_motion_host();
    host.push_native_display_presented();
    let (application, _) = build_open_portal_application_with_host(host.clone());
    let mut shell = application
        .launch_native_surface()
        .expect("the production native composition root launches");
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    assert!(matches!(
        shell
            .present_frame(10, 1)
            .unwrap_or_else(|_| panic!("the initial native frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));

    let presentation = super::super::native_duplicate_dismissal::current_presentation(&shell);
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        super::super::native_recovery::native_activation_drain(
            shell.host_session_identity().as_u64(),
            presentation,
        ),
        super::super::super::execution_deadline(20),
    );
    match ingress.transitions() {
        [WorthUiNativeIntentTransition::AttemptPrepared(_)] => {}
        [WorthUiNativeIntentTransition::ConfirmationRequired(_)] => {
            panic!("native portal activation unexpectedly required confirmation")
        }
        [WorthUiNativeIntentTransition::Stopped(_)] => {
            panic!("native portal activation stopped before dispatch")
        }
        transitions => panic!(
            "native portal activation produced {} transitions",
            transitions.len()
        ),
    }
    let transition =
        match shell.advance_native_intent_executions(super::super::super::execution_reading(1)) {
            UiIntentExecutionAdvanceOutcome::Advanced(report) => {
                report.into_transitions().into_vec().pop().unwrap()
            }
            UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
                panic!("native portal provider advance stopped: {stop:?}")
            }
        };
    host.push_native_display_presented();
    assert!(matches!(
        shell
            .begin_managed_native_intent_consequence_publication(
                transition
                    .into_consequence()
                    .expect("the completed portal intent retains its consequence"),
                40,
            )
            .expect("the portal consequence belongs to this native session"),
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Published(_)
    ));
    let installed = shell.inspect_motion_presentation_for_certification();
    assert_eq!(installed.active_tracks(), 1);
    assert_eq!(installed.retained_samples(), 1);

    host.push_native_display_presented();
    assert!(
        shell
            .admit_reduced_motion_tick_for_certification(1)
            .expect("native reduced-motion admission samples the real portal group"),
        "a decorative portal entrance must become inactive in the same reduced-motion turn"
    );

    let snapped = shell.inspect_motion_presentation_for_certification();
    assert_eq!(snapped.active_tracks(), 0);
    assert_eq!(snapped.last_tick(), Some(1));
    assert_eq!(snapped.opacity_units(), Some(u16::MAX));
    assert!(snapped.hit_test_visible().unwrap_or(false));
}

fn native_portal_motion_host() -> worth_ui_runtime::certification_support::ScriptedPresentationHost
{
    use worth_ui_host_contract::{WorthUiHostCapability, WorthUiHostCapabilityReport};

    let host = worth_ui_runtime::certification_support::ScriptedPresentationHost::default();
    host.set_capabilities(WorthUiHostCapabilityReport::available(vec![
        WorthUiHostCapability::NativePaint,
        WorthUiHostCapability::ViewportObservation,
        WorthUiHostCapability::DpiObservation,
        WorthUiHostCapability::FontMetrics,
        WorthUiHostCapability::TextIntrinsicMeasurement,
        WorthUiHostCapability::TextBaselineMeasurement,
        WorthUiHostCapability::PortalAnchorObservation,
        WorthUiHostCapability::SemanticFocusPlacement,
    ]));
    host
}
