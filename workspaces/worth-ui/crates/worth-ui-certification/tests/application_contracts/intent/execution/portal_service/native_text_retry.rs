use super::super::execution_reading;
use super::*;
use worth_ui::facade::app::{
    WorthUiNativeIntentTransition, WorthUiNativeManagedIntentConsequencePublicationOutcome,
    WorthUiNativeManagedRebindProgress,
};
use worth_ui::facade::intent::UiIntentExecutionAdvanceOutcome;
use worth_ui_runtime::facade::mounted::UiMountedFrameOutcome;

#[test]
fn progressed_portal_consequence_retries_the_exact_text_frame_before_commit() {
    let host = worth_ui_runtime::certification_support::ScriptedPresentationHost::default();
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::NativePaint,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
            worth_ui_host_contract::WorthUiHostCapability::DpiObservation,
            worth_ui_host_contract::WorthUiHostCapability::FontMetrics,
            worth_ui_host_contract::WorthUiHostCapability::TextIntrinsicMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::TextBaselineMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::PortalAnchorObservation,
            worth_ui_host_contract::WorthUiHostCapability::SemanticFocusPlacement,
        ]),
    );
    host.push_native_display_presented();
    let (application, _) = build_open_portal_application_with_host(host.clone());
    let mut shell = application
        .launch_native_declared_surface("visual.identity.surface.main")
        .unwrap();
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    assert!(matches!(
        shell
            .present_frame(10, 1)
            .unwrap_or_else(|_| panic!("initial native frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));
    let presentation = native_duplicate_dismissal::current_presentation(&shell);
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        native_activation::native_activation_drain(
            shell.host_session_identity().as_u64(),
            presentation,
        ),
        execution_deadline(20),
    );
    assert!(matches!(
        ingress.transitions(),
        [WorthUiNativeIntentTransition::AttemptPrepared(_)]
    ));
    let transition = match shell.advance_native_intent_executions(execution_reading(1)) {
        UiIntentExecutionAdvanceOutcome::Advanced(report) => {
            report.into_transitions().into_vec().pop().unwrap()
        }
        UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
            panic!("portal advance stopped: {stop:?}")
        }
    };
    host.push_in_flight(
        vec![worth_ui_runtime::certification_support::ScriptedSurfaceCompletion::RejectedBeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred)],
        worth_ui_host_contract::UiHostSurfaceCancellationOutcome::CancelledBeforeEffects);
    assert!(matches!(
        shell
            .begin_managed_native_intent_consequence_publication(
                transition.into_consequence().unwrap(),
                40
            )
            .unwrap(),
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Pending
    ));
    assert!(
        matches!(
            shell.inspect_mounted_frame(
                worth_ui_runtime::facade::mounted::UiMountedInspectionRequest::current()
            ),
            worth_ui_runtime::facade::mounted::UiMountedInspectionReceipt::Omitted(_)
        ),
        "pending host truth must not expose a successor as accepted"
    );
    let requests_before = host.requested_portal_overlay_counts();
    host.push_native_display_presented();
    let progress =
        worth_ui_native_platform::UiNativeApplicationPhysicalProgress::from_certification(
            worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
                worth_ui_host_native::UiNativePhysicalProgressClass::Presentation,
                None,
            ),
        );
    let receipt = match shell.progress_managed_rebind(&progress).unwrap() {
        WorthUiNativeManagedRebindProgress::IntentConsequencePublished(receipt) => receipt,
        WorthUiNativeManagedRebindProgress::Stopped(stop) => {
            panic!("consequence retry stopped: {stop:?}")
        }
        _ => panic!("the progressed text deferral must retry and publish in the same handoff"),
    };
    let requests_after = host.requested_portal_overlay_counts();
    assert_eq!(requests_after.len(), requests_before.len() + 1);
    assert_eq!(
        requests_after.last(),
        requests_before.last(),
        "the retry presents the same prepared Portal participants"
    );
    assert_eq!(host.native_in_flight_count(), 0);
    assert_eq!(host.pending_presentation_count(), 0);
    assert_eq!(
        shell
            .inspect_portal_runtime_for_certification()
            .active_portals(),
        1
    );
    assert!(shell
        .inspect_service_proposals_for_certification()
        .is_zero());
    drop(receipt);
    let shutdown = shell.shutdown();
    assert!(shutdown.intent_resources_empty());
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert_eq!(shutdown.portal_abandoned_indeterminate_records(), 0);
    assert!(shutdown.host_session_released());
}
