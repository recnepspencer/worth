use crate::runtime::tests::appearance_component_session_test_support as support;

#[path = "appearance_receipt_role_test_support.rs"]
mod role_support;
use role_support::{
    radius_role, radius_theme_bundle, radius_value_from, single_aspect_role,
    switched_validation_role, theme_bundle, update_radius_at_revision, update_theme, SWITCHED_SLOT,
};

#[path = "appearance_receipt_query_test_support.rs"]
mod query_support;
use query_support::{shutdown, why, why_for};

#[path = "appearance_receipt_frame_test_support.rs"]
mod frame_support;
use frame_support::{publish_frame, publish_validation_class};

#[cfg(test)]
#[path = "appearance_inspection_generation_tests.rs"]
mod appearance_inspection_generation_tests;
#[cfg(test)]
#[path = "appearance_receipt_basis_tests.rs"]
mod basis_tests;
#[cfg(test)]
#[path = "appearance_receipt_binding_tests.rs"]
mod binding_tests;
#[cfg(test)]
#[path = "appearance_receipt_provenance_tests.rs"]
mod provenance_tests;
#[cfg(test)]
#[path = "appearance_receipt_replacement_tests.rs"]
mod replacement_tests;
#[cfg(test)]
#[path = "appearance_receipt_settlement_tests.rs"]
mod settlement_tests;

struct MountedAppearanceFixture {
    session: crate::facade::WorthUiActiveApplicationSession,
    host: crate::certification_support::ScriptedPresentationHost,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    generation_before_allocation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    binding_generation_before_allocation: u64,
}

#[test]
fn real_validation_transition_reports_provenance_changed_byte_equivalent_suppression() {
    let role = switched_validation_role();
    let mut fixture = mounted_fixture(&role, &[SWITCHED_SLOT], false);
    update_theme(&mut fixture.session, "#405060");
    let first_revision = publish_validation_class(
        &mut fixture.session,
        fixture.graph_node,
        crate::runtime::intent::UiValidationAppearanceClass::Valid,
        None,
    );
    refresh_appearance_owner_snapshot(&mut fixture.session, &role, "appearance-receipt-semantic-a");
    publish_frame(&mut fixture.session, 1);
    publish_validation_class(
        &mut fixture.session,
        fixture.graph_node,
        crate::runtime::intent::UiValidationAppearanceClass::Invalid,
        Some(first_revision),
    );
    refresh_appearance_owner_snapshot(&mut fixture.session, &role, "appearance-receipt-semantic-b");
    publish_frame(&mut fixture.session, 2);

    let explanation = why(&fixture);
    assert_eq!(
        explanation.state_classes(),
        &[worth_ui_dsl::UiAppearanceAxisClass::ValidationInvalid]
    );
    assert!(explanation.input_evidence_changed());
    assert!(explanation.semantic_projection_changed());
    assert!(!explanation.resolved_aspect_value_changed());
    assert!(!explanation.mounted_mechanical_output_changed());
    assert!(explanation.equal_output_suppressed());
    assert!(!explanation.denied_before_effects());
    shutdown(fixture.session);
}

#[test]
fn real_validation_transition_reports_equal_output_suppressed() {
    let role = single_aspect_role(
        "test.receipt-equal",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut fixture = mounted_fixture(&role, &[], false);
    update_theme(&mut fixture.session, "#405060");
    let first_revision = publish_validation_class(
        &mut fixture.session,
        fixture.graph_node,
        crate::runtime::intent::UiValidationAppearanceClass::Valid,
        None,
    );
    refresh_appearance_owner_snapshot(&mut fixture.session, &role, "appearance-receipt-equal-a");
    publish_frame(&mut fixture.session, 1);
    publish_validation_class(
        &mut fixture.session,
        fixture.graph_node,
        crate::runtime::intent::UiValidationAppearanceClass::Invalid,
        Some(first_revision),
    );
    refresh_appearance_owner_snapshot(&mut fixture.session, &role, "appearance-receipt-equal-b");
    publish_frame(&mut fixture.session, 2);

    let explanation = why(&fixture);
    assert!(explanation.input_evidence_changed());
    assert!(explanation.semantic_projection_changed());
    assert!(!explanation.resolved_aspect_value_changed());
    assert!(!explanation.mounted_mechanical_output_changed());
    assert!(explanation.equal_output_suppressed());
    assert!(!explanation.denied_before_effects());
    assert_eq!(
        explanation.physical_suppression(),
        worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::Suppressed
    );
    shutdown(fixture.session);
}

#[test]
fn real_theme_transition_reports_resolved_aspect_value_changed() {
    let role = radius_role("test.receipt-resolved");
    let mut fixture = mounted_fixture(&role, &[], true);
    update_radius_at_revision(&mut fixture.session, [i32::MAX - 1; 4], 0);
    publish_frame(&mut fixture.session, 1);
    update_radius_at_revision(&mut fixture.session, [i32::MAX - 2; 4], 1);
    publish_frame(&mut fixture.session, 2);

    let explanation = why_for(&fixture, worth_ui_dsl::UiAppearanceAspect::Radius);
    assert!(!explanation.input_evidence_changed());
    assert!(explanation.semantic_projection_changed());
    assert!(explanation.resolved_aspect_value_changed());
    assert!(!explanation.mounted_mechanical_output_changed());
    assert!(!explanation.equal_output_suppressed());
    assert!(!explanation.denied_before_effects());
    assert_eq!(
        why_for(&fixture, worth_ui_dsl::UiAppearanceAspect::Radius).value(),
        worth_ui_inspection::UiAppearanceInspectionValue::Resolved(radius_value_from([
            i32::MAX - 2,
            i32::MAX - 2,
            i32::MAX - 2,
            i32::MAX - 2,
        ]))
    );
    assert_eq!(
        why_for(&fixture, worth_ui_dsl::UiAppearanceAspect::Radius).mounted_mechanic(),
        worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Unchanged
    );
    shutdown(fixture.session);
}

#[test]
fn real_theme_transition_reports_fully_lowered_radius_mechanical_delta() {
    let role = radius_role("test.receipt-radius-delta");
    let mut fixture = mounted_fixture(&role, &[], true);
    update_radius_at_revision(&mut fixture.session, [1; 4], 0);
    publish_frame(&mut fixture.session, 1);
    update_radius_at_revision(&mut fixture.session, [2; 4], 1);
    publish_frame(&mut fixture.session, 2);

    let explanation = why_for(&fixture, worth_ui_dsl::UiAppearanceAspect::Radius);
    assert!(!explanation.input_evidence_changed());
    assert!(explanation.semantic_projection_changed());
    assert!(explanation.resolved_aspect_value_changed());
    assert!(explanation.mounted_mechanical_output_changed());
    assert!(!explanation.equal_output_suppressed());
    assert!(!explanation.denied_before_effects());
    assert_eq!(
        explanation.mounted_mechanic(),
        worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Changed
    );
    assert_eq!(
        explanation.physical_suppression(),
        worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotSuppressed
    );
    shutdown(fixture.session);
}

fn mounted_fixture(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    extra: &[&str],
    six_axis: bool,
) -> MountedAppearanceFixture {
    let base = if six_axis {
        support::radius_appearance_component_builder_with_legacy_static_paint(role)
    } else {
        support::legacy_static_paint_appearance_component_builder(role)
    };
    let appearance_theme = if six_axis {
        radius_theme_bundle()
    } else {
        theme_bundle(extra, extra.contains(&SWITCHED_SLOT).then_some("#405060"))
    };
    let mut builder = base
        .register_appearance_theme_bundle(appearance_theme)
        .unwrap();
    let extra_initial_color = extra.contains(&SWITCHED_SLOT).then_some("#405060");
    for slot in extra {
        builder = builder.register_theme_token(support::appearance_theme_token_with_color(
            crate::capability::ThemeTokenId::new(*slot).unwrap(),
            extra_initial_color.unwrap_or("#112233"),
        ));
    }
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
    host.push_native_display_presented();
    if six_axis {
        host.push_native_display_presented();
        host.push_native_display_presented();
    } else {
        host.push_native_display_settled_without_effects();
        host.push_native_display_settled_without_effects();
    }
    let fixture_host = host.clone();
    let mut session = builder
        .with_rust_authored_declaration_fixture(support::appearance_fixture(role))
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("receipt fixture should prepare")
        .launch()
        .expect("receipt fixture should launch");
    admit_source(&mut session, role, "appearance-receipt-initial");
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
    let graph_nodes = session
        .graph()
        .node_identities()
        .filter_map(|identity| {
            let lookup = session.graph().lookup().graph_node(identity)?;
            let semantic = lookup
                .value()
                .declaration_identity()
                .authored_semantic_name()
                .to_owned();
            (semantic != "worth_ui.runtime.bootstrap.product_root")
                .then(|| (identity, Box::<str>::from(semantic)))
        })
        .collect::<Vec<_>>();
    for (graph_node, semantic) in graph_nodes {
        session
            .register_application_semantic_text(semantic, graph_node)
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
        .expect("receipt fixture has one appearance consumer")
        .graph_node_identity();
    let capability = session.host_measurement_capability();
    let assumptions = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    let generation_before_allocation = session.active_generation_identity();
    let binding_generation_before_allocation = session
        .presentation
        .active_appearance_theme_binding(surface)
        .expect("appearance surface has a binding before allocation")
        .binding_generation();
    let allocation = session
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
        .unwrap();
    assert!(allocation
        .committed()
        .receipts()
        .iter()
        .any(|receipt| receipt.identity().graph_node_identity() == graph_node));
    refresh_appearance_owner_snapshot(&mut session, role, "appearance-receipt-mounted");
    assert!(session
        .presentation
        .active_appearance_theme_binding(surface)
        .is_some());
    session.advance_mounted_identity_frame().unwrap();
    publish_frame(&mut session, 0);
    MountedAppearanceFixture {
        session,
        host: fixture_host,
        surface,
        graph_node,
        generation_before_allocation,
        binding_generation_before_allocation,
    }
}

fn admit_source(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    source_name: &str,
) {
    let candidate = support::appearance_candidate_submission(session, source_name, Some(role));
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
}

fn refresh_appearance_owner_snapshot(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    source_name: &str,
) {
    let candidate = support::appearance_candidate_submission(session, source_name, Some(role));
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
}
