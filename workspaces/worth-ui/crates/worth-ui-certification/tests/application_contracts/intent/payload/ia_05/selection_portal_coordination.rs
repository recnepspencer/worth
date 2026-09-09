use worth_ui::facade::intent::{
    UiIntentAdmissionDecision, UiIntentConsequencePublicationOutcome, UiIntentDeclaration,
    UiIntentDefinition, UiIntentExecutionAdvanceOutcome, UiIntentExecutionDispatchOutcome,
    UiIntentPayloadSource, UiIntentRuntimeServiceDestination, UiIntentSelection,
};
use worth_ui::facade::interaction::{UiHostInteractionIngressOutcome, UiSemanticInteraction};
use worth_ui::facade::source::{
    WorthUiSourceIngressExt, WorthUiSourceProvider, WorthUiWatcherEvent,
};
use worth_ui_certification::scenario::application_authority_closure::candidate_catalog::admit_candidate_catalog;
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_query_binding::{
    UiLiveCollectionProjectionCloseOutcome, UiPresentProjection, UiProjectionAvailability,
};
use worth_ui_runtime::facade::measurement_exchange::UiViewportExtentObservation;
use worth_ui_runtime::facade::mounted::{UiMountedFrameRequest, UiPresentationDeadline};
use worth_ui_test_support::{
    WorthUiFocusRuntimeCertificationExt, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedIdentityCertificationExt, WorthUiServiceStateCertificationExt,
};

use super::super::payload_types::{SelectionIntent, SELECTION_FIELD};
use super::super::world::{
    launch_scroll_portal, routed_scroll_selection_input, PayloadApplicationFacts,
    PayloadProjectionRegistration, DECLARATION,
};
use super::selection_identity::{collection_registration, open_collection, publish_collection};

#[test]
fn declared_selection_portal_rejects_atomically_then_commits_selection_and_focus_reveal() {
    let (mut query, _) =
        worth_ui_query_binding::certification::seeded_collection_projection_workspace_with_item_keys(
            vec![("pulse.alpha".to_owned(), "Alpha".to_owned(), 315_051)],
            worth_ui_query_binding::certification::WorthUiCollectionProjectionSeedPosture::Complete,
        );
    let registration = collection_registration(&query);
    let (live, snapshot) = open_collection(&registration, &mut query);
    let UiProjectionAvailability::Present(UiPresentProjection::Current(snapshot_value)) =
        snapshot.availability()
    else {
        panic!("the selection portal starts with one current Query row")
    };
    let projection = registration.view().identity().clone();
    let row = snapshot_value.rows()[0].row().clone();
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::new(8, 1, 16_384),
        UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let replacement_input = routed_scroll_selection_input(selection_declaration(&projection), true);
    let mut world = launch_scroll_portal::<SelectionIntent>(
        routed_scroll_selection_input(selection_declaration(&projection), false),
        PayloadProjectionRegistration::Collection(registration),
        PayloadApplicationFacts::default(),
        recorder.clone(),
    );
    publish_collection(&mut world, snapshot, 315_051);
    let option = world
        .interaction
        .session
        .current_projection_option(&projection, &row)
        .expect("the current selection row maps to one exact option");
    let UiSemanticInteraction::Activate(activation) = super::activation(&mut world, [20, 20])
    else {
        panic!("the current selection portal target activates")
    };
    let target = activation.target().mounted_instance();
    let target_geometry = world
        .interaction
        .hit_rows
        .iter()
        .find(|row| row.mounted_instance() == target)
        .expect("the activated target retains presented geometry");
    let scale =
        worth_ui::facade::observation_report::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32;
    let expected_reveal_offset =
        ((target_geometry.bounds().y() - target_geometry.clip_bounds().y()) * scale)
            .round()
            .max(0.0) as i64;
    assert!(matches!(
        world.interaction.scroll([20, 20], -1_000_000_000),
        UiHostInteractionIngressOutcome::Applied(_)
    ));
    let scrolled = world
        .interaction
        .session
        .inspect_scroll_runtime_for_certification();
    assert_eq!(scrolled.owner_geometry().len(), 1);
    assert!(
        scrolled.owner_geometry()[0].block_offset_subpixels() > expected_reveal_offset,
        "the proof requires a nonzero predecessor offset beyond the clip-relative target: expected={expected_reveal_offset}, geometry={:?}",
        scrolled.owner_geometry(),
    );
    let target_receipt = activation.target().node_receipt();
    world
        .interaction
        .session
        .bind_selection_item(target_receipt, target_receipt, option.clone())
        .expect("the declared single-item owner binds its current option");
    let selection = world
        .interaction
        .session
        .commit_selection_interaction(activation, option)
        .expect("the current option becomes a selection interaction");
    let route = super::product_route(
        &mut world.interaction,
        UiSemanticInteraction::SelectionCommit(selection),
    );
    let payload = world
        .interaction
        .session
        .prepare_intent_payload(route)
        .expect("the declared selection payload prepares");
    let operability = world
        .interaction
        .session
        .evaluate_intent_operability(payload);
    let definition = UiIntentDefinition::<SelectionIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let UiIntentAdmissionDecision::Admitted(admitted) = world
        .interaction
        .session
        .admit_intent(definition, operability)
    else {
        panic!("the declared selection portal admits")
    };
    assert!(matches!(
        world
            .interaction
            .session
            .dispatch_admitted_intent(admitted, crate::intent::execution::execution_deadline(20),),
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
    ));
    let consequence = completed_transition(&mut world)
        .into_consequence()
        .expect("selection portal execution retains its mounted consequence");
    let selection_before = world
        .interaction
        .session
        .inspect_selection_runtime_for_certification();
    let scroll_before = world
        .interaction
        .session
        .inspect_scroll_runtime_for_certification();
    let focus_before = world
        .interaction
        .session
        .inspect_focus_runtime_for_certification();

    let recovery = match world.interaction.session.publish_intent_consequences(
        consequence,
        worth_ui::facade::rebind::UiRebindExecutionPolicy::ordinary(),
        worth_ui::facade::rebind::UiRebindExecutionRequest::new(315_052),
    ) {
        UiIntentConsequencePublicationOutcome::Stopped(stop) => stop.into_recovery(),
        _ => panic!("the full recorder must reject before effects"),
    };
    assert_eq!(
        world
            .interaction
            .session
            .inspect_selection_runtime_for_certification(),
        selection_before
    );
    assert_eq!(
        world
            .interaction
            .session
            .inspect_scroll_runtime_for_certification(),
        scroll_before
    );
    assert_eq!(
        world
            .interaction
            .session
            .inspect_focus_runtime_for_certification(),
        focus_before
    );
    assert_eq!(recorder.drain_transcripts().len(), 1);

    assert!(matches!(
        world.interaction.session.retry_intent_consequences(
            recovery,
            worth_ui::facade::rebind::UiRebindExecutionPolicy::ordinary(),
            worth_ui::facade::rebind::UiRebindExecutionRequest::new(315_053),
        ),
        UiIntentConsequencePublicationOutcome::Published(_)
    ));
    let selected = world
        .interaction
        .session
        .inspect_selection_runtime_for_certification();
    assert_eq!((selected.owners(), selected.selected_keys()), (1, 1));
    assert_eq!(selected.available_catalog_owners(), 1);
    assert_eq!((selected.requests(), selected.keys_visited()), (1, 1));
    assert_eq!(selected.catalog_keys_reconciled(), 1);
    assert_eq!(
        selected.selected_application_item_keys(),
        &[core::num::NonZeroU64::new(315_051).unwrap()],
    );
    let revealed = world
        .interaction
        .session
        .inspect_scroll_runtime_for_certification();
    assert_eq!((revealed.owners(), revealed.admitted_requests()), (1, 2));
    assert_eq!(revealed.owners_visited(), 2);
    assert_eq!(
        revealed.owner_geometry()[0].block_offset_subpixels(),
        expected_reveal_offset,
        "Nearest reveal settles in owner content space, independent of the predecessor offset"
    );
    let focused = world
        .interaction
        .session
        .inspect_focus_runtime_for_certification();
    assert_eq!(focused.revision(), focus_before.revision() + 1);
    assert!(focused.current_participant().is_some());

    let _ = recorder.drain_transcripts();
    publish_mounted_replacement(&mut world, replacement_input);
    let replaced = world
        .interaction
        .session
        .inspect_selection_runtime_for_certification();
    assert_eq!((replaced.owners(), replaced.selected_keys()), (1, 1));
    assert_eq!(
        replaced.available_catalog_owners(),
        1,
        "the published successor installs its reconciled current catalog"
    );
    assert_eq!(
        replaced.selected_application_item_keys(),
        &[core::num::NonZeroU64::new(315_051).unwrap()],
        "production replacement preserves the stable selected application key"
    );

    world.interaction.session.unmount_instance(target).unwrap();
    assert_eq!(
        world
            .interaction
            .session
            .inspect_selection_runtime_for_certification()
            .owners(),
        0,
        "unmount retires the exact Selection owner immediately"
    );
    assert_eq!(
        world
            .interaction
            .session
            .inspect_scroll_runtime_for_certification()
            .owners(),
        0,
        "unmount retires the exact region-owned Scroll state immediately"
    );

    match live.close(&mut query) {
        UiLiveCollectionProjectionCloseOutcome::Closed(closed) => assert!(closed.owner_terminal()),
        UiLiveCollectionProjectionCloseOutcome::Stopped(stop) => {
            panic!(
                "selection portal Query owner closes: {:?}",
                stop.query_error()
            )
        }
    }
    let _ = world.interaction.session.shutdown();
}

fn publish_mounted_replacement(
    world: &mut super::super::world::PayloadWorld,
    input: worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
) {
    const PROVIDER: &str = "phase-315-selection-replacement";
    let provider = WorthUiSourceProvider::rust_authored(PROVIDER).with_rust_authored_input(input);
    let mut ingress = world
        .interaction
        .session
        .source_event_ingress(provider)
        .start();
    let settled = ingress
        .ingest([WorthUiWatcherEvent::provider_revision(PROVIDER)])
        .expect("replacement Rust source settles through production ingress");
    let submission = settled
        .attempt_candidate_for_certification(world.interaction.session.capabilities())
        .expect("selection successor lowers through the production compiler");
    let mut prepared = world
        .interaction
        .session
        .prepare_replacement(submission)
        .expect("selection successor prepares");
    let catalog = admit_candidate_catalog(&world.interaction.session, &mut prepared);
    let lowered = world
        .interaction
        .session
        .lower_prepared_replacement(*prepared)
        .expect("selection successor lowers");
    let pending = world
        .interaction
        .session
        .stage_prepared_replacement(lowered)
        .expect("selection successor stages");
    let boundary = world
        .interaction
        .session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_completion()
        .into_execution()
        .unwrap_or_else(|_| panic!("replacement boundary turn completes"))
        .into_activation_boundary();
    let replacement = match world
        .interaction
        .session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("selection successor prepares one mounted replacement")
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("changed component requires one mounted successor")
        }
    };
    assert!(matches!(
        replacement.present(UiPresentationDeadline::at_tick(315_054), 1),
        WorthUiMountedApplicationReplacementOutcome::Published { .. }
    ));
}

fn selection_declaration(
    projection: &worth_ui_query_binding::WorthUiQueryViewIdentity,
) -> UiIntentDeclaration<SelectionIntent> {
    UiIntentDeclaration::<SelectionIntent>::selection_commit(DECLARATION)
        .unwrap()
        .bind_payload(
            SELECTION_FIELD,
            UiIntentPayloadSource::<UiIntentSelection>::projection(projection),
        )
}

fn completed_transition(
    world: &mut super::super::world::PayloadWorld,
) -> worth_ui::facade::intent::UiIntentExecutionTransition {
    let report = match world
        .interaction
        .session
        .advance_intent_executions(crate::intent::execution::execution_reading(1))
    {
        UiIntentExecutionAdvanceOutcome::Advanced(report) => report,
        UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
            panic!("selection portal execution stopped: {stop:?}")
        }
    };
    let mut transitions = report.into_transitions().into_vec();
    assert_eq!(transitions.len(), 1);
    transitions.pop().unwrap()
}
use worth_ui::facade::app::{
    WorthUiMountedApplicationReplacementOutcome, WorthUiMountedReplacementPreparationOutcome,
};
