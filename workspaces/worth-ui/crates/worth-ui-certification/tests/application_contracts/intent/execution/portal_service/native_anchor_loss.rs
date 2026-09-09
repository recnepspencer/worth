use worth_ui::facade::{
    app::{
        WorthUiNativeIntentTransition, WorthUiNativeManagedIntentConsequencePublicationOutcome,
        WorthUiNativeManagedRebindProgress,
    },
    intent::{
        UiIntentDefinition, UiIntentExecutionAdvanceOutcome, UiIntentRuntimeServiceDestination,
    },
};
use worth_ui_runtime::{
    facade::mounted::{UiHostSurfaceCancellationOutcome, UiMountedFrameOutcome},
    native_platform::{UiNativeApplicationPhysicalProgress, UiNativeComponentPresenceChange},
};

use super::super::{execution_deadline, execution_reading};
use super::native_recovery::native_activation_drain;
use crate::intent::operability::{build_open_portal_application_with_host, PrimaryIntent};

const PORTAL_ANCHOR: &str = "component:visual.identity.component.paint_and_hit";

#[test]
fn anchor_loss_defers_native_unmount_until_portal_dismissal_is_physically_committed() {
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
        .launch_native_surface()
        .expect("the production native composition root launches");
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    assert!(matches!(
        shell
            .present_frame(10, 1)
            .unwrap_or_else(|_| panic!("the initial native frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));

    let presentation = super::native_duplicate_dismissal::current_presentation(&shell);
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        native_activation_drain(shell.host_session_identity().as_u64(), presentation),
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
            panic!("native portal provider advance stopped: {stop:?}")
        }
    };
    host.push_native_display_presented();
    assert!(matches!(
        shell
            .begin_managed_native_intent_consequence_publication(
                transition.into_consequence().unwrap(),
                40,
            )
            .expect("the managed Portal consequence belongs to this native session"),
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Published(_)
    ));
    assert_eq!(
        shell.component_is_present_for_certification(PORTAL_ANCHOR),
        Some(true)
    );
    assert_eq!(
        shell
            .inspect_portal_runtime_for_certification()
            .visible_portals(),
        1
    );

    let removal = UiNativeComponentPresenceChange::new(PORTAL_ANCHOR, false)
        .expect("the real authored Portal anchor accepts a presence successor");
    host.push_in_flight(
        vec![
            worth_ui_runtime::certification_support::ScriptedSurfaceCompletion::Presented(
                worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
                    worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
                    worth_ui_runtime::certification_support::scripted_presentation_epoch(),
                    worth_ui_host_contract::UiMountedCompletedEffects::new(vec![
                        worth_ui_host_contract::UiMountedEffectFamily::NativePaint,
                    ]),
                    Default::default(),
                ),
            ),
        ],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    assert!(
        shell
            .apply_component_presence_for_certification(std::slice::from_ref(&removal))
            .expect("anchor removal enters the production presence transaction"),
        "the mounted anchor must be retained while its Portal dismissal is in flight"
    );
    assert_eq!(host.native_in_flight_count(), 1);
    assert!(
        !shell
            .inspect_service_proposals_for_certification()
            .is_zero(),
        "the in-flight dismissal retains its typed multi-family proposal"
    );
    assert_eq!(
        shell.component_is_present_for_certification(PORTAL_ANCHOR),
        Some(true),
        "native unmount cannot race ahead of the Portal dismissal"
    );
    let before_completion = shell.inspect_portal_runtime_for_certification();
    assert_eq!(before_completion.visible_portals(), 1);
    assert_eq!(before_completion.committed_requests(), 1);

    let progress = UiNativeApplicationPhysicalProgress::from_certification(
        worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
            worth_ui_host_native::UiNativePhysicalProgressClass::Presentation,
            None,
            false,
        ),
    );
    assert!(matches!(
        shell
            .progress_managed_rebind(&progress)
            .expect("the exact physical completion remains session-bound"),
        WorthUiNativeManagedRebindProgress::PortalDismissed(_)
    ));
    assert_eq!(host.native_in_flight_count(), 0);
    assert_eq!(
        shell.component_is_present_for_certification(PORTAL_ANCHOR),
        Some(true),
        "physical Portal commit and component unmount remain separate ordered phases"
    );
    assert!(!shell
        .resume_component_presence_for_certification(41)
        .expect("the retained component transaction resumes after Portal commit"));
    assert_eq!(
        shell.component_is_present_for_certification(PORTAL_ANCHOR),
        Some(false)
    );
    assert!(!shell
        .apply_component_presence_for_certification(std::slice::from_ref(&removal))
        .expect("the exact program retry consumes the completed presence transaction"));

    let dismissed = shell.inspect_portal_runtime_for_certification();
    assert_eq!(dismissed.visible_portals(), 0);
    assert_eq!(dismissed.closing_portals(), 1);
    assert_eq!(dismissed.committed_requests(), 2);
    assert!(shell
        .inspect_service_proposals_for_certification()
        .is_zero());
    let shutdown = shell.shutdown();
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert!(shutdown.runtime_service_resources_empty());
}
