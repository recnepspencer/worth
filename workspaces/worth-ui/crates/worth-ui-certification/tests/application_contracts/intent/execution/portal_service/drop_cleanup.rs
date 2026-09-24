use worth_ui::facade::{
    intent::{
        UiIntentConsequencePublicationOutcome, UiIntentDefinition,
        UiIntentExecutionDispatchOutcome, UiIntentRuntimeServiceDestination,
    },
    rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest},
};
use worth_ui_runtime::certification_support::ScriptedPresentationHost;
use worth_ui_runtime::certification_support::ScriptedPresentationOutcome;
use worth_ui_test_support::{
    UiPortalDismissalCertificationOutcome, WorthUiFocusRuntimeCertificationExt,
    WorthUiPortalRuntimeCertificationExt, WorthUiServiceProposalCertificationExt,
};

use super::super::execution_deadline;
use crate::intent::{
    admission::phase3::world::AdmissionWorld,
    operability::{build_open_portal_application_with_host, PrimaryIntent},
};

#[test]
fn dropping_indeterminate_intent_consequence_releases_exact_service_proposals() {
    let host = indeterminate_host();
    host.push_presented();
    host.push_presentation(ScriptedPresentationOutcome::PresentationIndeterminate);
    let (application, facts) = build_open_portal_application_with_host(host);
    let mut world = AdmissionWorld::launch_application_on_declared_surface(
        application,
        facts,
        "visual.identity.surface.main",
        2,
        [18, 20],
    );
    let handle = completed_portal_consequence(&mut world);

    let recovery = match world.session.publish_intent_consequences(
        handle,
        UiRebindExecutionPolicy::ordinary(),
        UiRebindExecutionRequest::new(40),
    ) {
        UiIntentConsequencePublicationOutcome::Indeterminate(recovery) => recovery,
        _ => panic!("scripted open must retain indeterminate recovery"),
    };
    assert!(!recovery
        .inspect_retained_service_family_state_for_certification()
        .2
        .is_zero());
    drop(recovery);

    assert!(world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero());
    assert_eq!(
        world
            .session
            .inspect_focus_runtime_for_certification()
            .pending_portal_transitions(),
        0
    );
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .active_portals(),
        0
    );
    let shutdown = world.session.shutdown();
    assert_eq!(shutdown.portal_abandoned_indeterminate_records(), 0);
    assert_eq!(shutdown.intent_execution().active_after(), 0);
    assert_eq!(shutdown.intent_admission().active_after(), 0);
}

#[test]
fn dropping_indeterminate_portal_dismissal_releases_exact_service_proposals() {
    let host = indeterminate_host();
    host.push_presented();
    host.push_presented();
    host.push_presentation(ScriptedPresentationOutcome::PresentationIndeterminate);
    let (application, facts) = build_open_portal_application_with_host(host);
    let mut world = AdmissionWorld::launch_application_on_declared_surface(
        application,
        facts,
        "visual.identity.surface.main",
        2,
        [18, 20],
    );
    let handle = completed_portal_consequence(&mut world);
    assert!(matches!(
        world.session.publish_intent_consequences(
            handle,
            UiRebindExecutionPolicy::ordinary(),
            UiRebindExecutionRequest::new(40),
        ),
        UiIntentConsequencePublicationOutcome::Published(_)
    ));
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .visible_portals(),
        1
    );

    assert_eq!(
        world
            .session
            .publish_escape_portal_dismissal_for_certification(41),
        UiPortalDismissalCertificationOutcome::Indeterminate
    );
    assert!(world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero());
    assert_eq!(
        world
            .session
            .inspect_focus_runtime_for_certification()
            .pending_portal_transitions(),
        0
    );
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .visible_portals(),
        1,
        "disposing uncertain dismissal cannot commit false Portal closure"
    );
    let shutdown = world.session.shutdown();
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert_eq!(shutdown.portal_abandoned_indeterminate_records(), 0);
    assert_eq!(shutdown.motion_terminated_active_tracks(), 1);
    assert!(shutdown.motion_final_census_is_zero());
}

fn completed_portal_consequence(
    world: &mut AdmissionWorld,
) -> worth_ui::facade::intent::UiIntentConsequenceHandle {
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
    super::only_transition(world)
        .into_consequence()
        .expect("completed portal intent retains its consequence")
}

fn indeterminate_host() -> ScriptedPresentationHost {
    let host = ScriptedPresentationHost::default();
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::MountedFrameRecording,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
        ]),
    );
    host
}
