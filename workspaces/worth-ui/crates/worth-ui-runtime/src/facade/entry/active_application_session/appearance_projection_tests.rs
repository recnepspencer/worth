use crate::runtime::tests::appearance_component_session_test_support as support;

#[path = "appearance_projection_test_support.rs"]
mod test_support;
use test_support::theme_session;

#[test]
fn why_appearance_reads_the_production_resolve_and_mount_receipt() {
    let role = support::validation_background_role(support::APPEARANCE_TOKEN);
    let (mut session, host) = theme_session(&role);
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
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
    let graph_node = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.appearance_role_attachment().is_some())
        .expect("the production fixture has one appearance consumer")
        .graph_node_identity();
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
    let initial_observation = support::attached_appearance_candidate_submission(
        &session,
        "appearance-production-initial",
        "workspace.component.active_session_current",
    );
    let mut initial_turn = session.begin_observation_turn().unwrap();
    initial_turn.admit_source(initial_observation).unwrap();
    let initial_admitted = initial_turn.seal().unwrap();
    session.classify_observations(initial_admitted).unwrap();
    assert!(session.has_appearance_owner_snapshot_for_test());
    session.advance_mounted_identity_frame().unwrap();
    host.push_native_display_presented();
    test_support::publish_frame(&mut session, 0);

    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let value = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex("#405060").unwrap(),
    );
    let change = super::super::UiNativeThemeTokenValueChange::new(token, value).unwrap();
    session.admit_application_theme_values(&[change]).unwrap();
    assert!(session
        .presentation
        .appearance_invalidation_batch()
        .is_some_and(|batch| batch.selected_count() > 0));

    let outcome = session
        .execute_mounted_frame(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            1,
            |_| {},
        )
        .unwrap_or_else(|_| {
            panic!("the ordinary production frame should resolve and mount appearance")
        });
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let static_paint_colors = host.last_filled_rect_colors();
    assert!(!static_paint_colors.is_empty());
    assert!(static_paint_colors
        .iter()
        .all(|color| color.channels() == [17, 34, 51, 255]));

    let world = worth_ui_inspection::UiAppearanceInspectionWorld::new(
        session.session_identity().as_u64(),
        session
            .active_generation_identity()
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        surface.diagnostic_value(),
    );
    let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
        world,
        graph_node.digest(),
        worth_ui_dsl::UiAppearanceAspect::Background,
    );
    let explanation = match session.why_appearance(query) {
        worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) => explanation,
        outcome => panic!("production appearance receipt was not inspected: {outcome:?}"),
    };
    assert!(explanation.mounted_mechanical_output_changed());
    assert_eq!(
        explanation.mounted_mechanic(),
        worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Changed
    );
    assert_eq!(
        explanation.physical_suppression(),
        worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotSuppressed
    );
    assert!(explanation.cost().consumers_selected() > 0);
    assert_eq!(explanation.cost().theme_slots_compared(), 1);
    assert_eq!(
        explanation.value(),
        worth_ui_inspection::UiAppearanceInspectionValue::Resolved(
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                64, 80, 96, 255,
            ])),
        )
    );

    let _ = session.shutdown();
}

#[test]
fn first_appearance_attempt_denial_is_retained_by_why_appearance() {
    let role = support::validation_background_role_with_axis(
        support::APPEARANCE_TOKEN,
        worth_ui_dsl::UiAppearanceStateAxis::Hover,
    );
    let (mut session, _host) = theme_session(&role);
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
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
    let graph_node = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.appearance_role_attachment().is_some())
        .expect("the production fixture has one appearance consumer")
        .graph_node_identity();
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
        .expect("mounted allocation should commit before the denied appearance attempt");
    assert!(allocation_receipt
        .committed()
        .receipts()
        .iter()
        .any(|receipt| receipt.identity().graph_node_identity() == graph_node));
    let initial_observation = support::appearance_candidate_submission(
        &session,
        "appearance-production-denial",
        Some(&role),
    );
    let mut initial_turn = session.begin_observation_turn().unwrap();
    initial_turn.admit_source(initial_observation).unwrap();
    let initial_admitted = initial_turn.seal().unwrap();
    session.classify_observations(initial_admitted).unwrap();
    assert!(session.has_appearance_owner_snapshot_for_test());
    session.advance_mounted_identity_frame().unwrap();

    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let value = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex("#405060").unwrap(),
    );
    let change = super::super::UiNativeThemeTokenValueChange::new(token, value).unwrap();
    session.admit_application_theme_values(&[change]).unwrap();
    assert!(session
        .presentation
        .appearance_invalidation_batch()
        .is_some_and(|batch| batch.selected_count() > 0));

    let outcome = session
        .execute_mounted_frame(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            1,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("the production frame should retain the appearance denial"));
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));

    let world = worth_ui_inspection::UiAppearanceInspectionWorld::new(
        session.session_identity().as_u64(),
        session
            .active_generation_identity()
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        surface.diagnostic_value(),
    );
    let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
        world,
        graph_node.digest(),
        worth_ui_dsl::UiAppearanceAspect::Background,
    );
    let explanation = match session.why_appearance(query) {
        worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) => explanation,
        outcome => panic!("first appearance denial was not inspected: {outcome:?}"),
    };
    assert!(!explanation.input_evidence_changed());
    assert!(!explanation.semantic_projection_changed());
    assert!(!explanation.resolved_aspect_value_changed());
    assert!(!explanation.mounted_mechanical_output_changed());
    assert!(!explanation.equal_output_suppressed());
    assert!(explanation.denied_before_effects());
    assert_eq!(
        explanation.value(),
        worth_ui_inspection::UiAppearanceInspectionValue::Missing
    );
    assert_eq!(
        explanation.mounted_mechanic(),
        worth_ui_inspection::UiAppearanceInspectionMountedMechanic::NotAttempted
    );
    assert_eq!(
        explanation.physical_suppression(),
        worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotAttempted
    );
    assert_eq!(
        explanation.denial_posture(),
        Some(worth_ui_inspection::UiAppearanceInspectionDenialPosture::Basis)
    );
    assert_eq!(explanation.cost().theme_slots_compared(), 0);

    let _ = session.shutdown();
}
