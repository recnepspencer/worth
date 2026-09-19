use super::super::payload_types::{SelectionIntent, SELECTION_FIELD};
use super::super::world::{
    launch, routed_input, PayloadApplicationFacts, PayloadProjectionRegistration, PayloadWorld,
    DECLARATION,
};
use worth_query::facade::runtime::WorthQueryWorkspace;
use worth_ui::facade::intent::{
    UiIntentDeclaration, UiIntentInputOwnerRevision, UiIntentPayloadSource, UiIntentSelection,
};
use worth_ui::facade::interaction::{UiSelectionCommitStopReason, UiSemanticInteraction};
use worth_ui_dsl::WorthUiIntentInteractionFamily;
use worth_ui_query_binding::{
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionFactReceipt, UiCollectionProjectionOpenOutcome,
    UiCollectionProjectionRegistration, UiLiveCollectionProjection,
    UiLiveCollectionProjectionCloseOutcome, UiPresentProjection, UiProjectionAvailability,
    UiProjectionFieldRequirement, UiProjectionInputFactReference, UiProjectionObservation,
    WorthUiQueryWorkspaceExt,
};

#[test]
fn ia_05_selection_uses_exact_query_identity_and_rejects_the_stale_reorder_revision() {
    let (mut query, entities) =
        worth_ui_query_binding::certification::seeded_collection_projection_workspace_with_item_keys(
            vec![
                ("pulse.alpha".to_owned(), "Alpha".to_owned(), 315_050),
                ("pulse.bravo".to_owned(), "Bravo".to_owned(), 315_051),
            ],
            worth_ui_query_binding::certification::WorthUiCollectionProjectionSeedPosture::Complete,
        );
    let registration = collection_registration(&query);
    let (mut live, snapshot) = open_collection(&registration, &mut query);
    let UiProjectionAvailability::Present(UiPresentProjection::Current(snapshot_value)) =
        snapshot.availability()
    else {
        panic!("the canonical Query snapshot is current before WUI launch")
    };
    assert_eq!(snapshot_value.rows().len(), 2);
    let projection = registration.view().identity().clone();
    let declaration = UiIntentDeclaration::<SelectionIntent>::selection_commit(DECLARATION)
        .unwrap()
        .bind_payload(
            SELECTION_FIELD,
            UiIntentPayloadSource::<UiIntentSelection>::projection(&projection),
        );
    let mut world = launch::<SelectionIntent>(
        routed_input(declaration, WorthUiIntentInteractionFamily::SelectionCommit),
        PayloadProjectionRegistration::Collection(registration.clone()),
        PayloadApplicationFacts::default(),
    );
    let slot = world
        .projection_slot
        .expect("the frozen plan assigns one compact collection slot");
    let bravo_entity_identity = entities[1].evidence_identity();
    let bravo_identity = bravo_entity_identity.terminal_projection_for_reporting();
    let bravo_row = snapshot_value
        .rows()
        .iter()
        .find(|row| row.row().reporting_projection().as_str() == bravo_identity)
        .expect("the Query snapshot contains the independently seeded entity")
        .row()
        .clone();
    let snapshot_input = snapshot.intent_input_transition(slot).apply(None);
    publish_collection(&mut world, snapshot, 314_051);
    let original_option = world
        .interaction
        .session
        .current_projection_option(&projection, &bravo_row)
        .expect("the mounted snapshot exposes the exact current Query option");
    assert_eq!(
        original_option.reporting_projection().as_str(),
        bravo_identity
    );
    assert_selection_payload(&mut world, original_option.clone());

    worth_ui_query_binding::certification::update_projection_identity(
        &mut query,
        entities[1].clone(),
        "pulse.00-bravo",
    );
    let reordered = refresh_collection(&mut live, &mut query);
    assert_eq!(reordered.changes().len(), 2);
    assert!(reordered.changes().iter().all(|change| matches!(
        change,
        worth_ui_query_binding::UiCollectionProjectionChange::Move { .. }
    )));
    let UiProjectionAvailability::Present(UiPresentProjection::Current(reordered_value)) =
        reordered.availability()
    else {
        panic!("the Query reorder remains current")
    };
    assert!(reordered_value.rows().is_empty());
    let reordered_input = reordered
        .intent_input_transition(slot)
        .apply(Some(&snapshot_input));
    let UiProjectionInputFactReference::Collection(reordered_collection) = &reordered_input else {
        unreachable!("a collection patch preserves collection input shape")
    };
    assert_eq!(reordered_collection.row_count(), 2);
    assert_eq!(reordered_collection.transition_work().replaced_rows(), 0);
    assert_eq!(
        reordered_collection.transition_work().change_operations(),
        2
    );
    assert_eq!(reordered_collection.transition_work().key_probes(), 3);
    assert_eq!(reordered_collection.transition_work().node_copies(), 0);
    publish_collection(&mut world, reordered, 314_052);
    let current_option = world
        .interaction
        .session
        .current_projection_option(&projection, &bravo_row)
        .expect("the mounted move patch retains the exact Query row");
    assert_eq!(
        current_option.reporting_projection().as_str(),
        original_option.reporting_projection().as_str()
    );
    assert_ne!(
        current_option.owner_revision(),
        original_option.owner_revision()
    );

    let UiSemanticInteraction::Activate(stale_activation) = super::activation(&mut world, [10, 20])
    else {
        panic!("current target produces activation")
    };
    let stale = world
        .interaction
        .session
        .commit_selection_interaction(stale_activation, original_option)
        .expect_err("the pre-reorder owner revision must stop");
    assert_eq!(
        stale.reason(),
        UiSelectionCommitStopReason::ProjectionRevisionChanged
    );
    assert_selection_payload(&mut world, current_option);

    match live.close(&mut query) {
        UiLiveCollectionProjectionCloseOutcome::Closed(closed) => {
            assert!(closed.owner_terminal())
        }
        UiLiveCollectionProjectionCloseOutcome::Stopped(stop) => {
            panic!("the exact Query owner closes: {:?}", stop.query_error())
        }
    }
    let _ = world.interaction.session.shutdown();
}

fn assert_selection_payload(
    world: &mut PayloadWorld,
    option: worth_ui_query_binding::UiProjectionOptionReference,
) {
    let expected_option = option.clone();
    let UiSemanticInteraction::Activate(activation) = super::activation(world, [10, 20]) else {
        panic!("current target produces activation")
    };
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
        .expect("the exact current option becomes a selection interaction");
    assert_eq!(selection.option(), &expected_option);
    let route = super::product_route(
        &mut world.interaction,
        UiSemanticInteraction::SelectionCommit(selection),
    );
    let prepared = world
        .interaction
        .session
        .prepare_intent_payload(route)
        .expect("current option identity reaches the typed payload");
    assert_eq!(prepared.input_basis().cost().declared_fields(), 1);
    assert_eq!(prepared.input_basis().cost().query_inputs_read(), 1);
    let [UiIntentInputOwnerRevision::Query(revision)] = prepared.input_basis().owner_revisions()
    else {
        panic!("the selected option retains one exact Query owner revision")
    };
    assert_eq!(revision.field(), SELECTION_FIELD.descriptor());
    assert_eq!(revision.revision(), expected_option.owner_revision());
}

pub(super) fn collection_registration(
    query: &WorthQueryWorkspace,
) -> UiCollectionProjectionRegistration {
    let domain = query.worth_ui().expect("Worth UI Query domain installed");
    UiCollectionProjectionRegistration::text(
        domain
            .projection_view("platform.pulse.status")
            .expect("collection fixture view is installed"),
        UiProjectionFieldRequirement::declared("identity.id").unwrap(),
        [UiProjectionFieldRequirement::declared("status").unwrap()],
        false,
        false,
    )
    .expect("identity-preserving collection registration")
    .with_unsigned64_application_item_key_field(UiProjectionFieldRequirement::collection_item_key())
}

pub(super) fn open_collection(
    registration: &UiCollectionProjectionRegistration,
    query: &mut WorthQueryWorkspace,
) -> (
    UiLiveCollectionProjection,
    UiCollectionProjectionFactReceipt,
) {
    let binding = match registration.clone().admit(query) {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        admission => panic!("real collection binding admits: {admission:?}"),
    };
    let budget = UiCollectionProjectionBudget::new(2, 2, 0, 1024).unwrap();
    let UiCollectionProjectionOpenOutcome::Opened(opened) = binding.open(budget, query) else {
        panic!("real collection projection opens")
    };
    opened.into_parts()
}

fn refresh_collection(
    live: &mut UiLiveCollectionProjection,
    query: &mut WorthQueryWorkspace,
) -> UiCollectionProjectionFactReceipt {
    match live.refresh(query).expect("real Query refresh completes") {
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::Applied(receipt) => {
            receipt.into_fact()
        }
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::NoSemanticDelivery => {
            panic!("reorder changes the collection revision")
        }
    }
}

pub(super) fn publish_collection(
    world: &mut PayloadWorld,
    fact: UiCollectionProjectionFactReceipt,
    request: u64,
) {
    super::publish_projection(
        world,
        UiProjectionObservation::Collection(fact.into_observation()),
        request,
    );
    world.interaction.publish_successor();
}
