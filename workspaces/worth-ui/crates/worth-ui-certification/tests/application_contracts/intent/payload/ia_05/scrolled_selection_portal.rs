//! A Selection Portal opened from a Query row a wheel scrolled past the top of
//! its region.
use worth_query::facade::runtime::WorthQueryWorkspace;
use worth_ui::facade::intent::{
    UiIntentAdmissionDecision, UiIntentDeclaration, UiIntentDefinition,
    UiIntentExecutionAdvanceOutcome, UiIntentExecutionDispatchOutcome, UiIntentPayloadSource,
    UiIntentRuntimeServiceDestination, UiIntentSelection,
};
use worth_ui::facade::interaction::{
    UiActivateInteraction, UiHostInteractionIngressOutcome, UiSemanticInteraction,
};
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_query_binding::{
    UiLiveCollectionProjection, UiPresentProjection, UiProjectionAvailability,
    UiProjectionOptionReference, WorthUiQueryViewIdentity,
};
use worth_ui_runtime::facade::mounted::UiMountedFrameRequest;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiServiceStateCertificationExt,
};

use super::super::payload_types::{SelectionIntent, SELECTION_FIELD};
use super::super::world::{
    launch_scroll_portal, routed_scroll_selection_input, PayloadApplicationFacts,
    PayloadProjectionRegistration, PayloadWorld, DECLARATION,
};
use super::selection_identity::{collection_registration, open_collection};

/// The application key of the one Query row the Selection Portal selects.
pub(super) const SELECTED_KEY: u64 = 315_051;

pub(super) struct SelectionPortalWorld {
    pub(super) world: PayloadWorld,
    pub(super) query: WorthQueryWorkspace,
    pub(super) live: UiLiveCollectionProjection,
    pub(super) projection: WorthUiQueryViewIdentity,
    pub(super) option: UiProjectionOptionReference,
}

/// The activated row, with the offset the wheel left its region at and the
/// offset a Nearest reveal of the row settles it at.
pub(super) struct ScrolledActivation {
    pub(super) activation: UiActivateInteraction,
    pub(super) scrolled_offset: i64,
    pub(super) expected_reveal_offset: i64,
}

/// Launch the Selection Portal application over one current Query row and
/// publish the frame projecting it.
pub(super) fn launch_selection_portal(recorder: WorthUiHeadlessRecorder) -> SelectionPortalWorld {
    let (mut query, _) =
        worth_ui_query_binding::certification::seeded_collection_projection_workspace_with_item_keys(
            vec![("pulse.alpha".to_owned(), "Alpha".to_owned(), SELECTED_KEY)],
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
    let mut world = launch_scroll_portal::<SelectionIntent>(
        routed_scroll_selection_input(selection_declaration(&projection), false),
        PayloadProjectionRegistration::Collection(registration),
        PayloadApplicationFacts::default(),
        recorder,
    );
    super::publish_projection(
        &mut world,
        worth_ui_query_binding::UiProjectionObservation::Collection(snapshot.into_observation()),
        SELECTED_KEY,
    );
    let frame = world
        .interaction
        .session
        .prepare_application_presentation_frame(UiMountedFrameRequest::all_bound_surfaces())
        .expect("Query publication reuses the complete Selection Portal geometry");
    world.interaction.publish_prepared_successor(frame);
    let option = world
        .interaction
        .session
        .current_projection_option(&projection, &row)
        .expect("the current selection row maps to one exact option");
    SelectionPortalWorld {
        world,
        query,
        live,
        projection,
        option,
    }
}

/// Wheel the row past the top of its region and activate it.
pub(super) fn activate_scrolled_row(world: &mut PayloadWorld) -> ScrolledActivation {
    // The wheel offset commits when the frame carrying it is accepted, and
    // activation targets that frame.
    assert!(matches!(
        world.interaction.scroll([20, 20], -1_000_000_000),
        UiHostInteractionIngressOutcome::Applied(_)
    ));
    let frame = world
        .interaction
        .session
        .prepare_application_presentation_frame(UiMountedFrameRequest::all_bound_surfaces())
        .expect("the direct wheel frame prepares over the current geometry");
    world.interaction.publish_prepared_successor(frame);
    let UiSemanticInteraction::Activate(activation) = super::activation(world, [20, 20]) else {
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
    let scrolled = world
        .interaction
        .session
        .inspect_scroll_runtime_for_certification();
    assert_eq!(scrolled.owner_geometry().len(), 1);
    let scrolled_offset = scrolled.owner_geometry()[0].block_offset_subpixels();
    assert!(
        scrolled_offset > expected_reveal_offset,
        "the proof requires a nonzero predecessor offset beyond the clip-relative target: expected={expected_reveal_offset}, geometry={:?}",
        scrolled.owner_geometry(),
    );
    ScrolledActivation {
        activation,
        scrolled_offset,
        expected_reveal_offset,
    }
}

/// Commit `option` through `activation` and run the declared Selection Portal
/// intent to its mounted consequence.
pub(super) fn dispatch_selection_portal(
    world: &mut PayloadWorld,
    activation: UiActivateInteraction,
    option: UiProjectionOptionReference,
) -> worth_ui::facade::intent::UiIntentConsequenceHandle {
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
    completed_transition(world)
        .into_consequence()
        .expect("selection portal execution retains its mounted consequence")
}

pub(super) fn selection_declaration(
    projection: &WorthUiQueryViewIdentity,
) -> UiIntentDeclaration<SelectionIntent> {
    UiIntentDeclaration::<SelectionIntent>::selection_commit(DECLARATION)
        .unwrap()
        .bind_payload(
            SELECTION_FIELD,
            UiIntentPayloadSource::<UiIntentSelection>::projection(projection),
        )
}

fn completed_transition(
    world: &mut PayloadWorld,
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
