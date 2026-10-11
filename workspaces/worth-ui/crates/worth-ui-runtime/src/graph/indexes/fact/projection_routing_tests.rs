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
use crate::fact_contract::{UiProducedFact, UiQueryChangedFact};
use crate::graph::UiGraphFactConsumerIdentity;

const COMPONENT: &str = "platform.pulse.projected_status";
const PROJECTION: &str = "platform.pulse.status";

#[test]
fn projection_fact_selects_only_its_declared_content_consumer() {
    let mut workspace = worth_ui_query_binding::certification::collection_projection_workspace();
    let domain = workspace.worth_ui().expect("Worth UI domain installed");
    let registration = projection_registration(&domain);
    let app = projection_app(registration.clone());
    let binding = match registration.admit(&workspace) {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        UiCollectionProjectionBindingAdmission::Stopped(stop) => {
            panic!("real projection binding must admit: {stop:?}")
        }
    };
    let opened = match binding.open(
        UiCollectionProjectionBudget::new(2, 2, 0, 1024).unwrap(),
        &mut workspace,
    ) {
        UiCollectionProjectionOpenOutcome::Opened(opened) => opened,
        UiCollectionProjectionOpenOutcome::Stopped(stop) => {
            panic!("real collection projection must open: {stop:?}")
        }
    };
    let (live, fact) = opened.into_parts();
    let query = UiProducedFact::Query(UiQueryChangedFact::from_projection_observation(
        worth_ui_query_binding::UiProjectionObservation::Collection(fact.into_observation()),
    ));
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();

    let receipt = index
        .lookup(index.basis(), &query)
        .expect("projection content should resolve through the exact index");
    let node = authority
        .graph_snapshot()
        .nodes()
        .iter()
        .find(|node| {
            node.declaration_identity().authored_semantic_name() == format!("component:{COMPONENT}")
        })
        .expect("projected component graph node");
    let slot = authority
        .graph_snapshot()
        .mount_eligibility_slot_for_node(node.graph_node_identity())
        .expect("projected component mount slot");

    assert_eq!(receipt.entries().len(), 2);
    assert!(receipt.entries().iter().all(|entry| {
        entry
            .affected_aspect()
            .is_some_and(|aspect| aspect.canonical_label() == "content.text")
    }));
    assert!(receipt.entries().iter().any(|entry| {
        entry.consumer() == UiGraphFactConsumerIdentity::GraphNode(node.graph_node_identity())
    }));
    assert!(receipt.entries().iter().any(|entry| {
        entry.consumer()
            == UiGraphFactConsumerIdentity::MountEligibilitySlot(slot.mount_eligibility_identity())
    }));
    assert_eq!(receipt.cost().index_probes(), 1);
    assert_eq!(receipt.cost().selected_consumers(), 2);

    match live.close(&mut workspace) {
        worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome::Closed(receipt) => {
            assert!(receipt.owner_terminal());
        }
        worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome::Stopped(stop) => {
            panic!("projection close must succeed: {:?}", stop.query_error())
        }
    }
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
