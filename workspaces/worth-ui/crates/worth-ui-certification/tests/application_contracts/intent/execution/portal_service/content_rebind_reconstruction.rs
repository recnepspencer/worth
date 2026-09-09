use worth_ui::facade::{
    app::{
        WorthUiNativeIntentTransition, WorthUiNativeManagedIntentConsequencePublicationOutcome,
        WorthUiNativeManagedProjectionRebindOutcome, WorthUiNativeManagedRebindProgress,
    },
    intent::{
        UiIntentDefinition, UiIntentExecutionAdvanceOutcome, UiIntentRuntimeServiceDestination,
    },
    rebind::UiProjectionRebindRequest,
};
use worth_ui_runtime::{
    certification_support::{ScriptedPresentationHost, ScriptedSurfaceCompletion},
    facade::mounted::{UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationDenial},
};

use super::super::{execution_deadline, execution_reading};
use crate::intent::operability::{
    build_open_portal_projection_application_with_host, PrimaryIntent,
};

#[test]
fn native_content_retry_after_reconstruction_preserves_the_open_portal_projection() {
    let plan = worth_ui::facade::query_binding::WorthUiScalarProjectionHostPlan::prepare()
        .expect("the product Query plan prepares");
    let installed = plan
        .install_for_certification()
        .expect("the product Query owner installs for certification");
    let (registration, initial) = installed.into_parts();

    let host = ScriptedPresentationHost::default();
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
    let (application, _) =
        build_open_portal_projection_application_with_host(host.clone(), registration);
    let mut shell = application
        .launch_native_surface()
        .expect("the native portal and Query application launches");
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    let (initial_observation, initial_completion) = initial.into_parts();
    host.push_native_display_presented();
    let initial_receipt = match shell
        .begin_managed_projection_rebind(
            UiProjectionRebindRequest::new(initial_observation).observed_at_tick(1),
        )
        .expect("the initial Query observation enters the native content rebind")
    {
        WorthUiNativeManagedProjectionRebindOutcome::Published(receipt) => receipt,
        WorthUiNativeManagedProjectionRebindOutcome::Pending => {
            panic!("the scripted initial Query publication unexpectedly remained pending")
        }
        WorthUiNativeManagedProjectionRebindOutcome::Stopped(stop) => {
            panic!("initial Query publication stopped: {stop:?}")
        }
    };
    let initial_observation = initial_receipt
        .release_scalar_projection_observation()
        .unwrap_or_else(|_| panic!("the initial content receipt returns its Query observation"));
    let owner = initial_completion
        .admit_publication(initial_observation)
        .expect("the initial fact readmits its exact Query owner");
    let current = owner
        .advance(
            worth_ui::facade::query_binding::WorthUiScalarProjectionSourceRecord::new("ONLINE", 1)
                .expect("the successor source record is valid"),
        )
        .expect("the Query owner advances to a real successor fact");
    let (observation, completion) = current.into_parts();

    let presentation = super::native_duplicate_dismissal::current_presentation(&shell);
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        super::native_recovery::native_activation_drain(
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
            panic!("portal intent execution stopped: {stop:?}")
        }
    };
    host.push_native_display_presented();
    let portal_publication = match shell
        .begin_managed_native_intent_consequence_publication(
            transition.into_consequence().unwrap(),
            40,
        )
        .expect("the portal consequence belongs to the native session")
    {
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Published(receipt) => receipt,
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Pending => {
            panic!("the scripted portal publication unexpectedly remained pending")
        }
        WorthUiNativeManagedIntentConsequencePublicationOutcome::NoConsequences(_) => {
            panic!("the portal consequence was lost")
        }
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Stopped(stop) => {
            panic!("portal publication stopped: {stop:?}")
        }
    };
    assert!(portal_publication.focus_publication().is_some());
    drop(portal_publication);
    let portal_before = shell.inspect_portal_runtime_for_certification();
    let focus_before = shell.inspect_focus_runtime_for_certification();
    assert_eq!(portal_before.visible_portals(), 1);

    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::RejectedBeforeEffects(
            UiHostSurfacePresentationDenial::ReconstructionRequired,
        )],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    match shell
        .begin_managed_projection_rebind(
            UiProjectionRebindRequest::new(observation).observed_at_tick(41),
        )
        .expect("the Query observation enters the managed content rebind")
    {
        WorthUiNativeManagedProjectionRebindOutcome::Pending => {}
        WorthUiNativeManagedProjectionRebindOutcome::Published(_) => panic!(
            "the scripted in-flight Query publication unexpectedly completed; portal counts: {:?}",
            host.requested_portal_overlay_counts(),
        ),
        WorthUiNativeManagedProjectionRebindOutcome::Stopped(stop) => {
            panic!("the Query successor did not produce content work: {stop:?}")
        }
    }

    host.push_native_display_presented();
    host.push_native_display_presented();
    let progress =
        worth_ui_runtime::native_platform::UiNativeApplicationPhysicalProgress::from_certification(
            worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
                worth_ui_host_native::UiNativePhysicalProgressClass::Presentation,
                None,
                false,
            ),
        );
    let receipt = match shell
        .progress_managed_rebind(&progress)
        .expect("the exact native completion remains session-bound")
    {
        WorthUiNativeManagedRebindProgress::Published(receipt) => receipt,
        WorthUiNativeManagedRebindProgress::AwaitingProgress => {
            panic!("immediate reconstruction and retry unexpectedly remained pending")
        }
        WorthUiNativeManagedRebindProgress::IntentConsequencePublished(_) => {
            panic!("the Query retry was misattributed to the earlier portal intent")
        }
        WorthUiNativeManagedRebindProgress::PortalDismissed(_) => {
            panic!("content retry cannot dismiss the portal")
        }
        WorthUiNativeManagedRebindProgress::RecoveryBlocked(denial) => {
            panic!("content reconstruction was blocked: {denial:?}")
        }
        WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(recovery) => {
            panic!("content retry returned the wrong recovery class: {recovery:?}")
        }
        WorthUiNativeManagedRebindProgress::Unrelated => {
            panic!("the matching physical completion was reported as unrelated")
        }
        WorthUiNativeManagedRebindProgress::Stopped(stop) => {
            panic!("content reconstruction retry stopped: {stop:?}")
        }
    };

    assert_eq!(host.reconstruction_portal_overlay_counts(), [1]);
    let requested_portal_counts = host.requested_portal_overlay_counts();
    assert_eq!(requested_portal_counts.last(), Some(&1));
    assert_eq!(
        &requested_portal_counts[requested_portal_counts.len() - 3..],
        [1, 1, 1],
        "the rejected content candidate, predecessor reconstruction, and rebased retry must all carry the open portal; full trace: {requested_portal_counts:?}",
    );
    assert_eq!(
        shell.inspect_portal_runtime_for_certification(),
        portal_before,
        "content reconstruction cannot alter semantic Portal truth",
    );
    let focus_after = shell.inspect_focus_runtime_for_certification();
    assert_eq!(
        focus_after.current_participant(),
        focus_before.current_participant(),
        "content reconstruction preserves the logical focused participant",
    );
    assert_eq!(focus_after.pending_portal_transitions(), 0);
    assert_eq!(
        focus_after.participant_count(),
        focus_before.participant_count(),
    );

    let returned = receipt
        .release_scalar_projection_observation()
        .unwrap_or_else(|_| panic!("the content receipt returns the exact Query observation"));
    let owner = completion
        .admit_publication(returned)
        .expect("the exact published fact readmits its Query owner");
    let source_close = owner.close().expect("the Query owner closes terminally");
    assert!(source_close.owner_terminal());
    assert!(shell
        .inspect_service_proposals_for_certification()
        .is_zero());
    let shutdown = shell.shutdown();
    assert!(shutdown.intent_resources_empty());
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert!(shutdown.host_session_released());
}
