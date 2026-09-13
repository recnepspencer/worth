use worth_ui::facade::{
    intent::{
        UiIntentAdmissionCancellationReason, UiIntentAdmissionSettlementPosture,
        UiIntentConsequencePublicationOutcome, UiIntentDefinition, UiIntentExecutionAdvanceOutcome,
        UiIntentExecutionDispatchOutcome, UiIntentExecutionDispatchStopReason,
        UiIntentExecutionTransitionPosture, UiIntentRuntimeServiceDestination,
    },
    rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest},
};
use worth_ui_host_headless::UiHeadlessRecorderCapacity;
use worth_ui_test_support::{
    UiPortalDismissalCertificationOutcome, WorthUiFocusRuntimeCertificationExt,
    WorthUiPortalRuntimeCertificationExt, WorthUiServiceProposalCertificationExt,
};

use super::{
    admission::phase3::world::AdmissionWorld,
    execution::{execution_deadline, execution_reading},
    operability::{build_open_portal_application, portal_replacement_input, PrimaryIntent},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeServiceSemanticOutcome {
    pub(crate) portal_was_visible: bool,
    pub(crate) focus_was_placed: bool,
    pub(crate) dismissal_closed_only_top: bool,
    pub(crate) focus_restored_to_previous: bool,
    pub(crate) duplicate_was_idempotent: bool,
    pub(crate) proposals_are_zero: bool,
    pub(crate) terminal_resources_are_zero: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeadlessRuntimeServiceEvidence {
    pub(crate) semantic: RuntimeServiceSemanticOutcome,
    pub(crate) hot_rebind_preserved_portal: bool,
    pub(crate) focus_retargeted_to_successor: bool,
    pub(crate) inspection_was_bounded: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeRuntimeServiceEvidence {
    pub(crate) semantic: RuntimeServiceSemanticOutcome,
    pub(crate) indeterminate_effect_retained: bool,
    pub(crate) reconciled_from_exact_host_truth: bool,
    pub(crate) predecessor_was_reconstructed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CancelledRuntimeServiceAdmissionEvidence {
    pub(crate) cancelled_for_application_rebind: bool,
    pub(crate) active_attempts_after: usize,
    pub(crate) portal_truth_unchanged: bool,
    pub(crate) focus_truth_unchanged: bool,
    pub(crate) proposals_are_zero: bool,
    pub(crate) terminal_resources_are_zero: bool,
}

pub(crate) fn run_cancelled_runtime_service_admission_scenario(
) -> CancelledRuntimeServiceAdmissionEvidence {
    let (application, facts, _) =
        build_open_portal_application(UiHeadlessRecorderCapacity::new(8, 8, 16_384));
    let replacement = portal_replacement_input(&facts);
    let mut world = AdmissionWorld::launch_application_on_declared_surface_with_replacement(
        application,
        facts,
        "visual.identity.surface.main",
        2,
        [18, 20],
        replacement,
    );
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let admitted = world.admit_exact_definition(0, definition);
    world.rebind_application();
    let portal_before_dispatch = world.session.inspect_portal_runtime_for_certification();
    let focus_before_dispatch = world.session.inspect_focus_runtime_for_certification();
    let stopped = match world
        .session
        .dispatch_admitted_intent(admitted, execution_deadline(20))
    {
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_) => {
            panic!("a stale admitted runtime-service request opened an execution lane")
        }
        UiIntentExecutionDispatchOutcome::Stopped(stopped) => stopped,
    };
    let cancelled_for_application_rebind = matches!(
        stopped.reason(),
        UiIntentExecutionDispatchStopReason::AdmissionSettled(
            UiIntentAdmissionSettlementPosture::LifecycleCancelled(
                UiIntentAdmissionCancellationReason::ApplicationRebound
            )
        )
    );
    let active_attempts_after = stopped.active_after();
    let portal_truth_unchanged =
        world.session.inspect_portal_runtime_for_certification() == portal_before_dispatch;
    let focus_truth_unchanged =
        world.session.inspect_focus_runtime_for_certification() == focus_before_dispatch;
    let proposals_are_zero = world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero();
    let shutdown = world.session.shutdown();
    let terminal_resources_are_zero = shutdown.runtime_service_resource_census().is_empty()
        && shutdown.intent_resource_census().is_empty()
        && shutdown.mounted_presentation().is_empty()
        && shutdown.rebind().is_empty()
        && matches!(
            shutdown.host_session_release(),
            Some(worth_ui_host_contract::UiHostSessionReleaseOutcome::Released(_))
        );
    CancelledRuntimeServiceAdmissionEvidence {
        cancelled_for_application_rebind,
        active_attempts_after,
        portal_truth_unchanged,
        focus_truth_unchanged,
        proposals_are_zero,
        terminal_resources_are_zero,
    }
}

pub(crate) fn run_headless_runtime_service_scenario() -> HeadlessRuntimeServiceEvidence {
    let (application, facts, recorder) =
        build_open_portal_application(UiHeadlessRecorderCapacity::new(8, 8, 16_384));
    let replacement = portal_replacement_input(&facts);
    let mut world = AdmissionWorld::launch_application_on_declared_surface_with_replacement(
        application,
        facts,
        "visual.identity.surface.main",
        2,
        [18, 20],
        replacement,
    );
    let focus_before = world.session.inspect_focus_runtime_for_certification();
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let admitted = world.admit_exact_definition(0, definition);
    assert!(matches!(
        world
            .session
            .dispatch_admitted_intent(admitted, execution_deadline(20)),
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
    ));
    let report = match world
        .session
        .advance_intent_executions(execution_reading(1))
    {
        UiIntentExecutionAdvanceOutcome::Advanced(report) => report,
        UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
            panic!("headless runtime-service advance stopped: {stop:?}")
        }
    };
    let mut transitions = report.into_transitions().into_vec();
    assert_eq!(transitions.len(), 1);
    let transition = transitions.pop().expect("one completed portal transition");
    assert!(matches!(
        transition.posture(),
        UiIntentExecutionTransitionPosture::Completed { .. }
    ));
    let handle = transition
        .into_consequence()
        .expect("the portal intent retains its consequence");
    assert!(matches!(
        world.session.publish_intent_consequences(
            handle,
            UiRebindExecutionPolicy::ordinary(),
            UiRebindExecutionRequest::new(40),
        ),
        UiIntentConsequencePublicationOutcome::Published(_)
    ));

    let open_portal = world.session.inspect_portal_runtime_for_certification();
    let open_focus = world.session.inspect_focus_runtime_for_certification();
    let first_focus_placement = recorder
        .semantic_focus_placement(world.session.host_session_identity().as_u64())
        .expect("portal opening reaches the production headless focus port");
    recorder.drain_transcripts();

    world.rebind_application();

    let rebound_portal = world.session.inspect_portal_runtime_for_certification();
    let rebound_focus = world.session.inspect_focus_runtime_for_certification();
    let rebound_transcripts = recorder.drain_transcripts();
    let rebound_focus_placement = recorder
        .semantic_focus_placement(world.session.host_session_identity().as_u64())
        .expect("hot rebind retargets focus through the production headless port");
    let hot_rebind_preserved_portal = rebound_portal == open_portal
        && rebound_transcripts.len() == 1
        && rebound_transcripts[0].portal_overlays().len() == 1;
    let focus_retargeted_to_successor = rebound_focus.current_participant()
        == open_focus.current_participant()
        && rebound_focus_placement.request().identity()
            != first_focus_placement.request().identity()
        && rebound_focus_placement.request().target().node_receipt()
            != first_focus_placement.request().target().node_receipt();

    assert_eq!(
        world
            .session
            .publish_outside_portal_dismissal_for_certification(42),
        UiPortalDismissalCertificationOutcome::Published
    );
    let dismissed = world.session.inspect_portal_runtime_for_certification();
    let duplicate = world
        .session
        .publish_escape_portal_dismissal_for_certification(43);
    let after_duplicate = world.session.inspect_portal_runtime_for_certification();
    let restored_focus = world.session.inspect_focus_runtime_for_certification();
    let inspection = world
        .session
        .why_portal_closed()
        .expect("the owner retains a bounded latest close summary");
    let proposals_are_zero = world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero();
    let shutdown = world.session.shutdown();
    let terminal_resources_are_zero = shutdown.runtime_service_resource_census().is_empty()
        && shutdown.intent_resource_census().is_empty()
        && shutdown.mounted_presentation().is_empty()
        && shutdown.rebind().is_empty()
        && matches!(
            shutdown.host_session_release(),
            Some(worth_ui_host_contract::UiHostSessionReleaseOutcome::Released(_))
        );

    HeadlessRuntimeServiceEvidence {
        semantic: RuntimeServiceSemanticOutcome {
            portal_was_visible: open_portal.visible_portals() == 1,
            focus_was_placed: open_focus.current_participant().is_some(),
            dismissal_closed_only_top: dismissed.active_portals() == 1
                && dismissed.open_portals() == 0
                && dismissed.closing_portals() == 1,
            focus_restored_to_previous: restored_focus.current_participant()
                == focus_before.current_participant(),
            // The exact outcome, not a set of tolerated ones. A second dismissal
            // cause reaching an already-closing portal republishes its close and
            // must leave every portal posture count unchanged.
            duplicate_was_idempotent: duplicate == UiPortalDismissalCertificationOutcome::Published
                && after_duplicate.active_portals() == dismissed.active_portals()
                && after_duplicate.open_portals() == dismissed.open_portals()
                && after_duplicate.visible_portals() == dismissed.visible_portals()
                && after_duplicate.closing_portals() == dismissed.closing_portals(),
            proposals_are_zero,
            terminal_resources_are_zero,
        },
        hot_rebind_preserved_portal,
        focus_retargeted_to_successor,
        inspection_was_bounded: inspection.cost().is_bounded_latest_record_lookup(),
    }
}
