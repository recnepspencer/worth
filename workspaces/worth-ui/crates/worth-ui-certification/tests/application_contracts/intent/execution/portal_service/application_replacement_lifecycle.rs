use worth_ui::facade::{
    app::{
        WorthUiApplicationCutoverDenial, WorthUiMountedApplicationReplacementOutcome,
        WorthUiMountedReplacementPreparationOutcome, WorthUiPortalExitRetentionPendingKind,
    },
    intent::{
        UiIntentConsequencePublicationOutcome, UiIntentDefinition,
        UiIntentExecutionDispatchOutcome, UiIntentRuntimeServiceDestination,
    },
    rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest},
    source::{WorthUiSourceIngressExt, WorthUiSourceProvider, WorthUiWatcherEvent},
};
use worth_ui_runtime::certification_support::{presented_completion, ScriptedPresentationHost};
use worth_ui_runtime::facade::execution::WorthUiFrameBoundary;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationOutcome, UiMountedFrameRequest,
    UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiFrameworkTurnCertificationExt, WorthUiPortalRuntimeCertificationExt,
    WorthUiServiceProposalCertificationExt,
};

use super::motion_sampling::scripted_motion_host;
use super::only_transition;
use crate::intent::admission::phase3::world::AdmissionWorld;
use crate::intent::operability::{
    build_open_portal_application_with_host, portal_owner_removal_input, OperabilityFacts,
    PrimaryIntent,
};

#[test]
fn replacement_keeps_portal_installation_while_removing_one_exact_portal_owner() {
    let host = scripted_motion_host();
    for _ in 0..12 {
        host.push_presented();
    }
    let (application, facts) = build_open_portal_application_with_host(host.clone());
    let candidate_facts = facts.clone();
    let mut world =
        AdmissionWorld::launch_application_with_target(application, facts, 1, 2, [18, 20]);
    open_portal(&mut world);
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .visible_portals(),
        1
    );
    let presentation_calls = host.presentation_calls();

    let (pending, catalog) = prepare_owner_removal(&mut world, &candidate_facts);
    let boundary = safe_boundary(&mut world);
    let replacement = match world
        .session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("the candidate keeps Portal service ownership while removing one owner")
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("the removed owner changes application meaning")
        }
    };
    host.push_presented();
    let activation = match replacement.present(UiPresentationDeadline::at_tick(60), 2) {
        WorthUiMountedApplicationReplacementOutcome::Published { application, .. } => application,
        _ => panic!("the owner-removal successor must publish"),
    };

    assert_eq!(
        activation.active_generation(),
        world.session.generation_identity()
    );
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .active_portals(),
        0,
        "the exact missing owner removes its Portal row"
    );
    assert_eq!(
        world
            .session
            .runtime_service_resource_census()
            .portal_exit_retentions(),
        0
    );
    assert_eq!(
        host.presentation_calls(),
        presentation_calls + 1,
        "only the required mounted replacement frame reaches the host"
    );

    let shutdown = world.session.shutdown();
    assert!(shutdown.runtime_service_resource_census().is_empty());
}

#[test]
fn replacement_settles_retryable_portal_exit_before_removing_portal_authority() {
    let (mut world, candidate_facts, host, _) =
        world_with_retained_exit(|host| host.push_rejected());
    assert_eq!(
        world
            .session
            .progress_portal_exit_terminal_for_certification(113),
        worth_ui_test_support::UiPortalExitTerminalCertificationOutcome::Retry
    );
    super::exit_retention::assert_retention_census(&world.session);

    let calls_before_replacement = host.presentation_calls();
    publish_owner_removal(&mut world, &candidate_facts, &host);
    assert_eq!(
        world
            .session
            .inspect_portal_runtime_for_certification()
            .active_portals(),
        0
    );
    assert_eq!(
        world
            .session
            .runtime_service_resource_census()
            .portal_exit_retentions(),
        0
    );
    assert_eq!(host.presentation_calls(), calls_before_replacement + 1);
    super::exit_retention::assert_retention_census(&world.session);

    let shutdown = world.session.shutdown();
    assert!(shutdown.runtime_service_resource_census().is_empty());
}

#[test]
fn replacement_denies_in_flight_portal_exit_before_removing_portal_authority() {
    let (mut world, candidate_facts, host, boundary) = world_with_retained_exit(|host| {
        host.push_in_flight(
            vec![presented_completion()],
            UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
        );
    });
    settle_to_physical_pending(&mut world);
    assert_replacement_denied_with_pending(
        world,
        &candidate_facts,
        &host,
        boundary,
        WorthUiPortalExitRetentionPendingKind::InFlight,
    );
}

#[test]
fn replacement_denies_indeterminate_portal_exit_before_removing_portal_authority() {
    let (mut world, candidate_facts, host, boundary) = world_with_retained_exit(|host| {
        host.push_presentation(UiHostSurfacePresentationOutcome::PresentationIndeterminate);
    });
    settle_to_physical_pending(&mut world);
    assert_replacement_denied_with_pending(
        world,
        &candidate_facts,
        &host,
        boundary,
        WorthUiPortalExitRetentionPendingKind::Indeterminate,
    );
}

pub(super) fn open_portal(world: &mut AdmissionWorld) {
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let admitted = world.admit_exact_definition(0, definition);
    assert!(matches!(
        world
            .session
            .dispatch_admitted_intent(admitted, super::super::execution_deadline(20)),
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
    ));
    let handle = only_transition(world)
        .into_consequence()
        .expect("the completed intent retains its Portal consequence");
    assert!(matches!(
        world.session.publish_intent_consequences(
            handle,
            UiRebindExecutionPolicy::ordinary(),
            UiRebindExecutionRequest::new(40),
        ),
        UiIntentConsequencePublicationOutcome::Published(_)
    ));
}

pub(super) fn prepare_owner_removal(
    world: &mut AdmissionWorld,
    facts: &OperabilityFacts,
) -> (
    worth_ui::facade::app::WorthUiPendingApplicationCutover,
    worth_ui::facade::graph::UiAdmittedAllocationCatalogDelta,
) {
    let provider_name = "portal-owner-removal";
    let provider = WorthUiSourceProvider::rust_authored(provider_name)
        .with_rust_authored_input(portal_owner_removal_input(facts));
    let mut ingress = world.session.source_event_ingress(provider).start();
    let settled = ingress
        .ingest([WorthUiWatcherEvent::provider_revision(provider_name)])
        .expect("the owner-removal candidate settles through source ingress");
    let submission = settled
        .attempt_candidate_for_certification(world.session.capabilities())
        .expect("the owner-removal candidate lowers through production semantics");
    let prepared = world
        .session
        .prepare_replacement(submission)
        .expect("the owner-removal candidate prepares");
    let graph = world.session.graph();
    let removed_roots = graph
        .allocation_planning_node_identities()
        .filter(|node| {
            graph.lookup().graph_node(*node).is_some_and(|record| {
                record
                    .value()
                    .declaration_identity()
                    .authored_semantic_name()
                    == "component:visual.identity.component.paint_and_hit"
            })
        })
        .collect();
    let catalog = prepared
        .admit_candidate_allocation_catalog_delta(Vec::new(), removed_roots)
        .expect("the candidate admits exact active allocation-root removals");
    let lowered = world
        .session
        .lower_prepared_replacement(*prepared)
        .expect("the owner-removal candidate lowers");
    let pending = world
        .session
        .stage_prepared_replacement(lowered)
        .expect("the owner-removal candidate stages");
    (pending, catalog)
}

pub(super) fn safe_boundary(world: &mut AdmissionWorld) -> WorthUiFrameBoundary {
    world
        .session
        .execute_framework_turn(|_| {})
        .expect("no host presentation attempt is active")
        .into_completion()
        .into_execution()
        .expect("the empty framework turn completes")
        .into_activation_boundary()
}

fn world_with_retained_exit(
    script: impl FnOnce(&ScriptedPresentationHost),
) -> (
    AdmissionWorld,
    OperabilityFacts,
    ScriptedPresentationHost,
    WorthUiFrameBoundary,
) {
    let host = scripted_motion_host();
    for _ in 0..5 {
        host.push_presented();
    }
    let (application, facts) = build_open_portal_application_with_host(host.clone());
    let candidate_facts = facts.clone();
    let mut world =
        AdmissionWorld::launch_application_with_target(application, facts, 1, 2, [18, 20]);
    open_portal(&mut world);
    let boundary = safe_boundary(&mut world);
    script(&host);
    super::exit_retention::terminalize_portal_exit_motion(&mut world);
    (world, candidate_facts, host, boundary)
}

fn settle_to_physical_pending(world: &mut AdmissionWorld) {
    assert_eq!(
        world
            .session
            .progress_portal_exit_terminal_for_certification(113),
        worth_ui_test_support::UiPortalExitTerminalCertificationOutcome::AwaitingPhysical
    );
    super::exit_retention::assert_retention_census(&world.session);
}

fn assert_replacement_denied_with_pending(
    mut world: AdmissionWorld,
    candidate_facts: &OperabilityFacts,
    host: &ScriptedPresentationHost,
    boundary: WorthUiFrameBoundary,
    expected: WorthUiPortalExitRetentionPendingKind,
) {
    let (pending, catalog) = prepare_owner_removal(&mut world, candidate_facts);
    let portal_before = world.session.inspect_portal_runtime_for_certification();
    let proposals_before = world.session.inspect_service_proposals_for_certification();
    let calls_before = host.presentation_calls();
    match world.session.prepare_mounted_replacement(
        pending,
        catalog,
        boundary,
        None,
        UiMountedFrameRequest::all_bound_surfaces(),
    ) {
        Err(WorthUiApplicationCutoverDenial::PortalExitRetentionPending { kind, retry }) => {
            assert_eq!(kind, expected);
            drop(retry);
        }
        Err(other) => panic!("unexpected replacement denial: {other:?}"),
        Ok(_) => panic!("physical portal exit retention must deny replacement"),
    }
    assert_eq!(
        world.session.inspect_portal_runtime_for_certification(),
        portal_before,
        "denial leaves Portal rows and receipts untouched"
    );
    assert_eq!(
        world.session.inspect_service_proposals_for_certification(),
        proposals_before,
        "typed replacement denial preserves the exact proposal correlation"
    );
    assert_eq!(host.presentation_calls(), calls_before);
    super::exit_retention::assert_retention_census(&world.session);
    let shutdown = world.session.shutdown();
    assert!(shutdown.runtime_service_resource_census().is_empty());
}

fn publish_owner_removal(
    world: &mut AdmissionWorld,
    candidate_facts: &OperabilityFacts,
    host: &ScriptedPresentationHost,
) {
    let (pending, catalog) = prepare_owner_removal(world, candidate_facts);
    let boundary = safe_boundary(world);
    let replacement = match world
        .session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("retryable Portal retention is removable before replacement")
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("the removed owner changes application meaning")
        }
    };
    host.push_presented();
    assert!(matches!(
        replacement.present(UiPresentationDeadline::at_tick(60), 2),
        WorthUiMountedApplicationReplacementOutcome::Published { .. }
    ));
}
