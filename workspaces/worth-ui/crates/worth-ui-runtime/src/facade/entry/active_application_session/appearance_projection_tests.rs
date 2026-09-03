use crate::runtime::tests::appearance_component_session_test_support as support;

#[test]
fn why_appearance_reads_the_production_resolve_and_mount_receipt() {
    let role = support::validation_background_role(support::APPEARANCE_TOKEN);
    let mut session = theme_session(&role);
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
    session.advance_mounted_identity_frame().unwrap();

    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let value = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex("#405060").unwrap(),
    );
    let change = super::super::UiNativeThemeTokenValueChange::new(token, value).unwrap();
    session.admit_application_theme_values(&[change]).unwrap();
    let consumers_selected = session
        .complete_application_theme_values_source()
        .canonical_consumers()
        .len() as u32;
    assert!(consumers_selected > 0);

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
    assert_eq!(
        explanation.invalidation_cause(),
        worth_ui_inspection::UiAppearanceInspectionInvalidationCause::MountedMechanicalOutputChanged
    );
    assert_eq!(
        explanation.mounted_mechanic(),
        worth_ui_inspection::UiAppearanceInspectionMountedMechanic::Changed
    );
    assert_eq!(
        explanation.physical_suppression(),
        worth_ui_inspection::UiAppearanceInspectionPhysicalSuppression::NotSuppressed
    );
    assert_eq!(explanation.cost().consumers_selected(), consumers_selected);
    assert_eq!(
        explanation.value(),
        worth_ui_inspection::UiAppearanceInspectionValue::Resolved(
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                17, 34, 51, 255,
            ])),
        )
    );

    let _ = session.shutdown();
}

fn theme_session(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiActiveApplicationSession {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::NativePaint,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
            worth_ui_host_contract::WorthUiHostCapability::DpiObservation,
            worth_ui_host_contract::WorthUiHostCapability::PortalAnchorObservation,
        ]),
    );
    support::appearance_component_builder(role)
        .register_appearance_theme_bundle(theme_bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(support::appearance_fixture(role))
        .freeze()
        .map(|application| {
            host.push_native_display_presented();
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("appearance capability fixture should prepare")
        .launch()
        .expect("appearance capability fixture should launch")
}

fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [crate::capability::UiThemeSlotDeclaration::new(
            token.clone(),
            crate::capability::ThemeTokenFamily::surface(),
            worth_ui_dsl::UiThemeValueKind::Color,
            crate::capability::ThemeTokenSource::application(),
            crate::capability::UiThemeSlotDisclosure::Public,
            crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )],
    )
    .unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.production").unwrap(),
        1,
        &catalog,
        [(
            token,
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                17, 34, 51, 255,
            ])),
        )],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(catalog, vec![definition]).unwrap()
}
