use worth_ui_dsl::{
    WorthUiArtifactInputBodyAtom, WorthUiProjectionCollectionPolicy,
    WorthUiProjectionCollectionSelection, WorthUiProjectionLifecycle,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
use worth_ui_query_binding::{
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionOpenOutcome, UiCollectionProjectionRegistration,
    UiProjectionFieldRequirement, WorthUiQueryWorkspaceExt,
};

use crate::capability::{
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentStateOwnership,
};
use crate::runtime::observation::UiChangeClassificationOutcome;

const COMPONENT: &str = "platform.pulse.projected_status";
const PROJECTION: &str = "platform.pulse.status";

#[test]
fn real_collection_snapshot_compiles_distinct_keyed_mounted_rows() {
    let (mut workspace, entities) =
        worth_ui_query_binding::certification::seeded_collection_projection_workspace(
            vec![
                ("pulse.alpha".to_owned(), "Alpha".to_owned()),
                ("pulse.bravo".to_owned(), "Bravo".to_owned()),
            ],
            worth_ui_query_binding::certification::WorthUiCollectionProjectionSeedPosture::Complete,
        );
    let domain = workspace.worth_ui().expect("Worth UI domain installed");
    let registration = projection_registration(&domain);
    let mut session = projection_app(registration.clone())
        .launch()
        .expect("projection application launches");
    let binding = match registration.admit(&workspace) {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        UiCollectionProjectionBindingAdmission::Stopped(stop) => {
            panic!("real collection binding must admit: {stop:?}")
        }
    };
    let opened = match workspace
        .with_advancement_mut(|phase, workspace| {
            binding.open(
                &phase,
                UiCollectionProjectionBudget::new(2, 2, 0, 1024).unwrap(),
                workspace,
            )
        })
        .expect("fixture workspace host call opens")
    {
        UiCollectionProjectionOpenOutcome::Opened(opened) => opened,
        UiCollectionProjectionOpenOutcome::Stopped(stop) => {
            panic!("real collection projection must open: {stop:?}")
        }
    };
    let (mut live, fact) = opened.into_parts();
    let plan = projection_plan(
        &mut session,
        worth_ui_query_binding::UiProjectionObservation::Collection(fact.into_observation()),
    );
    let content = plan.content();
    let graph_node = content
        .graph_nodes()
        .next()
        .expect("the exact projection content consumer is indexed");
    let crate::mounting::UiMountedSemanticTextContent::Collection(collection) =
        content.get(graph_node).expect("collection content exists")
    else {
        panic!("shape-specific collection fact cannot collapse to scalar content")
    };
    let crate::mounting::UiMountedCollectionTextDirective::Replace(rows) = collection.value()
    else {
        panic!("an initial collection fact must compile as a snapshot replacement")
    };

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].selected_values()[0].as_ref(), "Alpha");
    assert_eq!(rows[1].selected_values()[0].as_ref(), "Bravo");
    assert_eq!(
        rows.iter()
            .map(|row| row.identity().query_reference().query_identity().clone())
            .collect::<Vec<_>>(),
        entities
            .iter()
            .map(|entity| entity.evidence_identity())
            .collect::<Vec<_>>()
    );
    assert_eq!(collection.posture().as_ref(), "CURRENT · COMPLETE");

    drop(plan);
    worth_ui_query_binding::certification::update_projection_status(
        &mut workspace,
        entities[1].clone(),
        "Bravo updated",
    );
    let refreshed = match workspace
        .with_execution_request_mut(|execution, workspace| live.refresh(execution, workspace))
        .expect("fixture workspace host call opens")
        .unwrap()
    {
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::Applied(receipt) => receipt,
        worth_ui_query_binding::UiCollectionProjectionRefreshOutcome::NoSemanticDelivery => {
            panic!("the Query update must deliver its exact patch")
        }
    };
    let patch_plan = projection_plan(
        &mut session,
        worth_ui_query_binding::UiProjectionObservation::Collection(
            refreshed.into_fact().into_observation(),
        ),
    );
    let patch_content = patch_plan.content();
    let patch_node = patch_content.graph_nodes().next().unwrap();
    let crate::mounting::UiMountedSemanticTextContent::Collection(collection) =
        patch_content.get(patch_node).unwrap()
    else {
        panic!("collection patch remains shape-specific")
    };
    let crate::mounting::UiMountedCollectionTextDirective::Patch(changes) = collection.value()
    else {
        panic!("an applied Query delivery must remain an indexed patch")
    };
    assert!(matches!(
        changes.as_ref(),
        [crate::mounting::UiMountedCollectionTextChange::Update(row)]
            if row.identity().query_reference().query_identity()
                == &entities[1].evidence_identity()
                && row.selected_values()[0].as_ref() == "Bravo updated"
    ));

    drop(patch_plan);
    match live.close(&mut workspace) {
        worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome::Closed(_) => {}
        worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome::Stopped(stop) => {
            panic!("live collection closes: {:?}", stop.query_error())
        }
    }
    let _ = session.shutdown();
}

fn projection_plan(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    observation: worth_ui_query_binding::UiProjectionObservation,
) -> super::super::UiRebindPlan {
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_projection_query(observation).unwrap();
    let admitted = turn.seal().unwrap();
    let changed = match session.classify_observations(admitted).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("a real projection observation changes mounted content"),
    };
    let lifecycle = session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    session
        .compile_rebind_plan(lifecycle, super::super::UiRebindExecutionPolicy::ordinary())
        .unwrap()
}

fn projection_app(registration: UiCollectionProjectionRegistration) -> crate::facade::WorthUiApp {
    let module = WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_component_body_atoms(
            COMPONENT,
            [
                WorthUiArtifactInputBodyAtom::Identifier("content".to_owned()),
                WorthUiArtifactInputBodyAtom::Identifier("projection".to_owned()),
                WorthUiArtifactInputBodyAtom::Identifier(PROJECTION.to_owned()),
            ],
        )
        .try_with_query_collection_text(
            PROJECTION,
            PROJECTION,
            "identity.id",
            WorthUiProjectionCollectionSelection::new(
                ["status"],
                WorthUiProjectionLifecycle::Live,
                WorthUiProjectionCollectionPolicy::new(false, false),
            ),
        )
        .unwrap();
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(ComponentDescriptor::new(
            ComponentId::new(COMPONENT).unwrap(),
            ComponentPropSchema::named("platform.pulse.projected_status.props"),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        ))
        .register_collection_projection(registration)
        .expect("product projection registration")
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("projection content application")
}

fn projection_registration(
    domain: &worth_ui_query_binding::WorthUiInstalledQueryDomain,
) -> UiCollectionProjectionRegistration {
    UiCollectionProjectionRegistration::text(
        domain.projection_view(PROJECTION).unwrap(),
        UiProjectionFieldRequirement::declared("identity.id").unwrap(),
        [UiProjectionFieldRequirement::declared("status").unwrap()],
        false,
        false,
    )
    .unwrap()
}
