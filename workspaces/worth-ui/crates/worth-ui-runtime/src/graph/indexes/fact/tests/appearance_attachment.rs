use std::collections::BTreeSet;

#[test]
fn appearance_role_is_explicit_per_node_not_a_component_default() {
    let app = super::static_paint_app();
    let snapshot = app.prepared_authority().graph_snapshot();
    let attached = super::graph_node_named(snapshot, super::STATIC_PAINT_COMPONENT);
    let peer = super::graph_node_named(snapshot, super::STATIC_PAINT_PEER);

    assert!(attached.appearance_role_attachment().is_some());
    assert!(peer.appearance_role_attachment().is_none());
    assert_eq!(attached.component_reference(), peer.component_reference());
    assert!(app
        .prepared_authority()
        .consumed_fact_index()
        .has_appearance_consumers());
}

#[test]
fn unattached_node_does_not_infer_appearance_demand_from_its_component() {
    let app = crate::declaration::appearance_fact_index_test_support::unattached_static_paint_app(
        "unattached-static-paint",
        super::STATIC_PAINT_COMPONENT,
        super::STATIC_PAINT_TOKEN,
    );

    assert!(super::graph_node_named(
        app.prepared_authority().graph_snapshot(),
        super::STATIC_PAINT_COMPONENT,
    )
    .appearance_role_attachment()
    .is_none());
    assert!(!app
        .prepared_authority()
        .consumed_fact_index()
        .has_appearance_consumers());
}

#[test]
fn appearance_slot_selection_uses_the_existing_fact_index_without_host_work() {
    let app = super::static_paint_app();
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();
    let slot = worth_ui_dsl::UiThemeSlotIdentity::new(super::STATIC_PAINT_TOKEN).unwrap();
    let node = super::graph_node_named(authority.graph_snapshot(), super::STATIC_PAINT_COMPONENT)
        .graph_node_identity();

    let slot_batch = index
        .select_appearance_slot_consumers(index.basis(), slot.as_str(), slot.as_str())
        .expect("declared theme slot should resolve");

    let attached =
        super::graph_node_named(authority.graph_snapshot(), super::STATIC_PAINT_COMPONENT);
    let peer = super::graph_node_named(authority.graph_snapshot(), super::STATIC_PAINT_PEER);
    assert_eq!(attached.component_reference(), peer.component_reference());
    assert!(attached.appearance_role_attachment().is_some());
    assert!(peer.appearance_role_attachment().is_none());
    assert_eq!(
        slot_batch.consumers(),
        [node],
        "sharing a component type cannot create an unauthored appearance consumer"
    );
    let fact = crate::fact_contract::UiProducedFact::AuthoredSource(
        crate::fact_contract::UiAuthoredChangedFact::new(
            crate::fact_contract::UiAuthoredFactSelector::node(super::STATIC_PAINT_TOKEN),
            crate::fact_contract::UiAuthoredFactKind::SemanticsChanged,
        ),
    );
    let receipt = index
        .lookup_retained(&fact)
        .expect("the existing authored-fact index should answer slot demand");
    assert!(receipt.entries().iter().any(|entry| {
        entry.consumer() == crate::graph::UiGraphFactConsumerIdentity::GraphNode(node)
    }));
}

#[test]
fn role_slot_fact_lookup_selects_only_attached_nodes() {
    let app = role_only_app();
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();
    let attached =
        super::graph_node_named(authority.graph_snapshot(), super::STATIC_PAINT_COMPONENT)
            .graph_node_identity();
    let peer = super::graph_node_named(authority.graph_snapshot(), super::STATIC_PAINT_PEER)
        .graph_node_identity();
    let fact = crate::fact_contract::UiProducedFact::AuthoredSource(
        crate::fact_contract::UiAuthoredChangedFact::new(
            crate::fact_contract::UiAuthoredFactSelector::node(super::STATIC_PAINT_TOKEN),
            crate::fact_contract::UiAuthoredFactKind::SemanticsChanged,
        ),
    );

    let receipt = index
        .lookup_retained(&fact)
        .expect("role slot should be represented by the authored-fact index");
    assert_eq!(receipt.entries().len(), 2);
    assert_eq!(
        receipt
            .entries()
            .iter()
            .filter_map(|entry| match entry.consumer() {
                crate::graph::UiGraphFactConsumerIdentity::GraphNode(node) => Some(node),
                crate::graph::UiGraphFactConsumerIdentity::MountEligibilitySlot(_) => None,
            })
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([attached])
    );
    assert!(receipt
        .entries()
        .iter()
        .all(|entry| entry.consumption_relation().is_appearance_role_slot()));
    assert!(receipt
        .entries()
        .iter()
        .all(|entry| entry.affected_aspect().is_none()));

    let slot = worth_ui_dsl::UiThemeSlotIdentity::new(super::STATIC_PAINT_TOKEN).unwrap();
    let batch = index
        .select_appearance_slot_consumers(index.basis(), slot.as_str(), slot.as_str())
        .expect("declared theme slot should resolve");
    assert_eq!(batch.consumers(), [attached]);
    assert_ne!(attached, peer);
}

#[test]
fn canonical_slot_selection_exposes_unknown_authored_fact_denial() {
    let app = role_only_app();
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();
    let slot = worth_ui_dsl::UiThemeSlotIdentity::new(super::STATIC_PAINT_TOKEN).unwrap();
    let denial =
        index.select_appearance_slot_consumers(index.basis(), slot.as_str(), "theme.pulse.missing");

    assert_eq!(
        denial,
        Err(
            crate::graph::UiGraphFactLookupDenial::UnknownAuthoredDeclaration {
                authored_identity: "theme.pulse.missing".into(),
            }
        )
    );
}

#[test]
fn canonical_slot_selection_rejects_a_foreign_requested_basis() {
    let app = role_only_app();
    let foreign = super::foreign_indexed_app();
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();
    let slot = worth_ui_dsl::UiThemeSlotIdentity::new(super::STATIC_PAINT_TOKEN).unwrap();
    let requested_basis = foreign.prepared_authority().consumed_fact_index().basis();

    assert_eq!(
        index.select_appearance_slot_consumers(requested_basis, slot.as_str(), slot.as_str()),
        Err(crate::graph::UiGraphFactLookupDenial::BasisMismatch {
            index_basis: index.basis(),
            requested_basis,
        })
    );
}

fn role_only_app() -> crate::facade::WorthUiApp {
    crate::declaration::appearance_fact_index_test_support::role_only_fact_index_app(
        "role-only-fact-index",
        super::STATIC_PAINT_TOKEN,
        super::STATIC_PAINT_COMPONENT,
        super::STATIC_PAINT_PEER,
    )
}
