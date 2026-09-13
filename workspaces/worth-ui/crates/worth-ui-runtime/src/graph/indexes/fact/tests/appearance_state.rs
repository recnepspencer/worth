use worth_ui_dsl::UiAppearanceStateAxis;

#[test]
fn state_demand_membership_uses_the_attached_nodes_consumed_fact_index() {
    let app = super::static_paint_app();
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();
    let attached = super::graph_node_for(authority.graph_snapshot(), super::STATIC_PAINT_COMPONENT);
    let unattached = super::graph_node_for(authority.graph_snapshot(), super::STATIC_PAINT_PEER);
    assert!(index.consumes_appearance_state(UiAppearanceStateAxis::Validation, attached));
    assert!(!index.consumes_appearance_state(UiAppearanceStateAxis::Hover, attached));
    assert!(!index.consumes_appearance_state(UiAppearanceStateAxis::Validation, unattached));
}
