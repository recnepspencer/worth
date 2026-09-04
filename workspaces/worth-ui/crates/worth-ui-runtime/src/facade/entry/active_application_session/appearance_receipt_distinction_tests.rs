use crate::runtime::tests::appearance_component_session_test_support as support;

#[path = "appearance_receipt_role_test_support.rs"]
mod role_support;
use role_support::{
    single_aspect_role, switched_validation_role, theme_bundle, update_theme,
    update_theme_at_revision, SWITCHED_SLOT,
};

#[path = "appearance_receipt_query_test_support.rs"]
mod query_support;
use query_support::{shutdown, why, why_for};

#[cfg(test)]
#[path = "appearance_receipt_basis_tests.rs"]
mod basis_tests;

struct MountedAppearanceFixture {
    session: crate::facade::WorthUiActiveApplicationSession,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
}

#[test]
fn real_source_turn_reports_input_evidence_changed() {
    let role = single_aspect_role(
        "test.receipt-input",
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
    refresh_appearance_owner_snapshot(&mut fixture.session, &role, "appearance-receipt-input-a");
    publish_frame(&mut fixture.session, 1);
    let second_revision = publish_validation_class(
        &mut fixture.session,
        fixture.graph_node,
        crate::runtime::intent::UiValidationAppearanceClass::Valid,
        Some(first_revision),
    );
    assert_eq!(second_revision, first_revision + 1);
    refresh_appearance_owner_snapshot(&mut fixture.session, &role, "appearance-receipt-input-b");
    publish_frame(&mut fixture.session, 2);

    let explanation = why(&fixture);
    assert_eq!(explanation.denial_posture(), None,);
    assert_eq!(
        explanation.state_classes(),
        &[worth_ui_dsl::UiAppearanceAxisClass::ValidationValid]
    );
    assert_eq!(
        explanation.invalidation_cause(),
        worth_ui_inspection::UiAppearanceInspectionInvalidationCause::InputEvidenceChanged
    );
    shutdown(fixture.session);
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
    assert_eq!(
        explanation.invalidation_cause(),
        worth_ui_inspection::UiAppearanceInspectionInvalidationCause::EqualOutputSuppressed
    );
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
    assert_eq!(
        explanation.invalidation_cause(),
        worth_ui_inspection::UiAppearanceInspectionInvalidationCause::EqualOutputSuppressed
    );
    assert_eq!(
        explanation.physical_suppression(),
        worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::Suppressed
    );
    shutdown(fixture.session);
}

#[test]
fn real_theme_transition_reports_resolved_aspect_value_changed() {
    let role = single_aspect_role(
        "test.receipt-resolved",
        worth_ui_dsl::UiAppearanceAspect::Foreground,
        support::APPEARANCE_TOKEN,
    );
    let mut fixture = mounted_fixture(&role, &[], true);
    update_theme(&mut fixture.session, "#304050");
    publish_frame(&mut fixture.session, 1);
    update_theme_at_revision(&mut fixture.session, "#405060", 1);
    publish_frame(&mut fixture.session, 2);

    assert_eq!(
        why_for(&fixture, worth_ui_dsl::UiAppearanceAspect::Foreground).invalidation_cause(),
        worth_ui_inspection::UiAppearanceInspectionInvalidationCause::ResolvedAspectValueChanged
    );
    shutdown(fixture.session);
}

fn mounted_fixture(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    extra: &[&str],
    six_axis: bool,
) -> MountedAppearanceFixture {
    let base = if six_axis {
        support::single_aspect_appearance_component_builder(
            role,
            worth_ui_dsl::UiAppearanceAspect::Foreground,
        )
    } else {
        support::legacy_static_paint_appearance_component_builder(role)
    };
    let mut builder = base
        .register_appearance_theme_bundle(theme_bundle(
            extra,
            extra.contains(&SWITCHED_SLOT).then_some("#405060"),
        ))
        .unwrap();
    let extra_initial_color = extra.contains(&SWITCHED_SLOT).then_some("#405060");
    for slot in extra {
        builder = builder.register_theme_token(support::appearance_theme_token_with_color(
            crate::capability::ThemeTokenId::new(*slot).unwrap(),
            extra_initial_color.unwrap_or("#112233"),
        ));
    }
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::NativePaint,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
            worth_ui_host_contract::WorthUiHostCapability::DpiObservation,
            worth_ui_host_contract::WorthUiHostCapability::PortalAnchorObservation,
        ]),
    );
    host.push_native_display_presented();
    if six_axis {
        host.push_native_display_presented();
        host.push_native_display_presented();
    } else {
        host.push_native_display_settled_without_effects();
        host.push_native_display_settled_without_effects();
    }
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
    let themes = session
        .capabilities()
        .appearance_themes()
        .expect("receipt fixture has an appearance theme bundle");
    let definition = themes
        .definitions()
        .first()
        .expect("receipt fixture has a theme definition");
    let host_profile = worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        "appearance-receipt-test-host",
        1,
        worth_ui_host_contract::UiHostAppearanceMechanicFamily::ALL,
        Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
    )
    .unwrap();
    let capability =
        crate::runtime::appearance::UiThemeCapabilityAdmission::from_frozen_capabilities(
            themes,
            definition.identity(),
            session.capabilities().appearance_roles(),
            &host_profile,
        )
        .unwrap()
        .issue(
            [role.role().clone()],
            surface,
            session.active_generation_identity(),
        )
        .unwrap();
    session
        .presentation
        .install_initial_appearance_theme_binding(capability)
        .unwrap();
    session.advance_mounted_identity_frame().unwrap();
    publish_frame(&mut session, 0);
    MountedAppearanceFixture {
        session,
        surface,
        graph_node,
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

fn publish_validation_class(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    graph_node: crate::graph::UiGraphNodeIdentity,
    class: crate::runtime::intent::UiValidationAppearanceClass,
    expected_revision: Option<u64>,
) -> u64 {
    let identity = session.inspect_mounted_identity();
    let row = identity
        .mounted_instances()
        .iter()
        .find(|row| row.graph_node_identity() == graph_node)
        .expect("validation transition has a mounted appearance instance");
    let instance = row.identity();
    let receipt = session
        .inspect_mounted_identity()
        .frame_receipts()
        .iter()
        .find(|row| row.mounted_instance_identity() == instance)
        .expect("validation transition has a current node receipt")
        .node_receipt_identity();
    let target = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        session, graph_node, instance, receipt,
    )
    .unwrap();
    session
        .intent_application_facts
        .publish_validation_appearance_fact(target, expected_revision, class)
        .unwrap();
    session
        .intent_application_facts
        .validation_appearance_snapshot()
        .and_then(|snapshot| snapshot.fact_basis_for(graph_node, instance))
        .expect("validation publication should retain its fact revision")
        .1
}

fn publish_frame(session: &mut crate::facade::WorthUiActiveApplicationSession, now: u64) {
    let outcome = session
        .execute_mounted_frame(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            now,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("receipt frame should publish"));
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
}
