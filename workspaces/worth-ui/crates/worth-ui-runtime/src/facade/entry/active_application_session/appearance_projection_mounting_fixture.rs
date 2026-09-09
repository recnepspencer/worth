pub(super) fn mount(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    device_scale_milli: u32,
) -> (
    worth_ui_host_contract::UiSemanticSurfaceIdentity,
    crate::graph::UiGraphNodeIdentity,
) {
    let graph_node = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.appearance_role_attachment().is_some())
        .expect("the production fixture has one appearance consumer")
        .graph_node_identity();
    mount_graph_node(session, device_scale_milli, graph_node)
}

pub(super) fn mount_graph_node(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    device_scale_milli: u32,
    graph_node: crate::graph::UiGraphNodeIdentity,
) -> (
    worth_ui_host_contract::UiSemanticSurfaceIdentity,
    crate::graph::UiGraphNodeIdentity,
) {
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                device_scale_milli,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let graph_nodes = {
        let graph = session.graph();
        graph
            .node_identities()
            .filter_map(|identity| {
                let lookup = graph.lookup().graph_node(identity)?;
                let semantic = lookup
                    .value()
                    .declaration_identity()
                    .authored_semantic_name()
                    .to_owned();
                (semantic != "worth_ui.runtime.bootstrap.product_root")
                    .then(|| (identity, Box::<str>::from(semantic)))
            })
            .collect::<Vec<_>>()
    };
    for (graph_node, authored_semantic_identity) in graph_nodes {
        session
            .register_application_semantic_text(authored_semantic_identity, graph_node)
            .unwrap();
        let mounted_node = session.mounted_graph_node(graph_node).unwrap();
        session.mount_instance(mounted_node, surface).unwrap();
    }
    let capability = session.host_measurement_capability();
    let assumptions = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    let allocation_receipt = session
        .establish_mounted_allocation_catalog(
            1,
            [
                crate::facade::entry::UiMountedAllocationMeasurementRequest::new(
                    worth_ui_host_contract::UiMeasurementEvidenceFamily::ViewportExtent,
                    crate::host::UiHostMeasurementNeed::ViewportExtent(
                        worth_ui_host_contract::UiViewportExtentRequest,
                    ),
                    crate::host::UiHostMeasurementNormalizationContext::viewport_logical_exact(
                        assumptions,
                    ),
                ),
            ],
        )
        .expect("mounted allocation should commit the component allocation");
    let committed_nodes = allocation_receipt
        .committed()
        .receipts()
        .iter()
        .map(|receipt| receipt.identity().graph_node_identity())
        .collect::<Vec<_>>();
    assert!(
        committed_nodes.contains(&graph_node),
        "appearance node {:?} was not admitted; committed allocation nodes: {:?}",
        graph_node,
        committed_nodes
    );
    crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        session,
        surface,
        1,
        &[],
    );
    (surface, graph_node)
}
