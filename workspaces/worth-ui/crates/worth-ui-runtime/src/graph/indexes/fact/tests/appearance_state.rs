use worth_ui_dsl::UiAppearanceStateAxis;

#[test]
fn state_selection_uses_the_existing_consumed_fact_index_basis() {
    let app = super::static_paint_app();
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();
    let graph_node =
        super::graph_node_for(authority.graph_snapshot(), super::STATIC_PAINT_COMPONENT);
    let selection = index
        .select_appearance_state_consumers(
            index.basis(),
            UiAppearanceStateAxis::Validation,
            graph_node,
        )
        .expect("validation state consumers should use the admitted fact index");

    assert_eq!(selection.basis(), index.basis());
    assert_eq!(selection.axis(), UiAppearanceStateAxis::Validation);
    assert_eq!(selection.cost().index_probes(), 1);
    assert_eq!(selection.cost().selected_consumers(), 1);
    assert_eq!(selection.cost().unrelated_neighborhoods_touched(), 0);
    assert_eq!(selection.consumers().len(), 1);
    assert_eq!(selection.consumers()[0].graph_node(), graph_node);
    let unrelated = super::graph_node_for(authority.graph_snapshot(), super::STATIC_PAINT_PEER);
    let unrelated_selection = index
        .select_appearance_state_consumers(
            index.basis(),
            UiAppearanceStateAxis::Validation,
            unrelated,
        )
        .expect("an exact graph-node scope should produce an empty local selection");
    assert!(unrelated_selection.consumers().is_empty());
    assert_eq!(unrelated_selection.cost().selected_consumers(), 0);

    let foreign = super::foreign_indexed_app();
    let foreign_basis = foreign.prepared_authority().consumed_fact_index().basis();
    assert_eq!(
        index.select_appearance_state_consumers(
            foreign_basis,
            UiAppearanceStateAxis::Validation,
            graph_node
        ),
        Err(crate::graph::UiGraphFactLookupDenial::BasisMismatch {
            index_basis: index.basis(),
            requested_basis: foreign_basis,
        })
    );
}
