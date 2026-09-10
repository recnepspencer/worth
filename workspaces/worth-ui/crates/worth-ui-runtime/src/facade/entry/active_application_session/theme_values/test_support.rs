use crate::runtime::tests::appearance_component_session_test_support as component_support;

pub(super) fn session_with_bundle(
    bundle: crate::capability::FrozenAppearanceThemeCapabilities,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let observer = host.clone();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let session = component_support::legacy_static_paint_appearance_component_builder(role)
        .register_appearance_theme_bundle(bundle)
        .unwrap()
        .with_rust_authored_declaration_fixture(component_support::appearance_fixture(role))
        .freeze()
        .map(|application| {
            host.push_native_display_presented();
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("theme-value session should prepare")
        .launch()
        .expect("theme-value session should launch");
    (session, observer)
}

pub(super) fn mounted_session(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
    worth_ui_host_contract::UiSemanticSurfaceIdentity,
    crate::graph::UiGraphNodeIdentity,
) {
    let (mut session, host) = session_with_bundle(theme_bundle(), role);
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
        .expect("theme-value fixture has an appearance consumer")
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
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(&mut session, surface);
    let baseline = publish_frame(&mut session, 0);
    assert_eq!(baseline.cost_report().adapter().presented_surfaces(), 1);
    assert_eq!(host.presentation_calls(), 1);
    let baseline_colors = host.last_filled_rect_colors();
    assert!(!baseline_colors.is_empty());
    assert!(baseline_colors
        .iter()
        .all(|color| color.channels() == [17, 34, 51, 255]));
    let observation = component_support::attached_appearance_candidate_submission(
        &session,
        "theme-value-truth-initial",
        "workspace.component.active_session_current",
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(observation).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
    assert!(session.has_appearance_owner_snapshot_for_test());
    host.push_native_display_settled_without_effects();
    let initial_appearance = publish_frame(&mut session, 1);
    assert_eq!(
        initial_appearance
            .cost_report()
            .adapter()
            .presented_surfaces(),
        0
    );
    assert_eq!(host.presentation_calls(), 2);
    assert!(host.last_filled_rect_colors().is_empty());
    (session, host, surface, graph_node)
}

pub(super) fn publish_frame(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    now: u64,
) -> crate::mounting::UiMountedFramePublicationReceipt {
    let outcome = session.execute_mounted_frame(
        crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
        |_| {},
    );
    match outcome {
        Ok(crate::mounting::UiMountedFrameOutcome::Published(receipt)) => return receipt,
        Ok(crate::mounting::UiMountedFrameOutcome::Unchanged(_)) => {
            panic!("theme-value support frame {now} was unexpectedly unchanged")
        }
        Ok(crate::mounting::UiMountedFrameOutcome::Reconciled(_)) => {
            panic!("theme-value support frame {now} was unexpectedly reconciled")
        }
        Ok(crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(frame)) => {
            let denial = frame
                .rejections()
                .first()
                .map(|rejection| format!("{:?}", rejection.denial()))
                .unwrap_or_else(|| "no surface rejection".to_owned());
            panic!("theme-value support frame {now} was rejected before effects: {denial}")
        }
        Ok(crate::mounting::UiMountedFrameOutcome::InFlight(_)) => {
            panic!("theme-value support frame {now} remained in flight")
        }
        Ok(crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame)) => {
            panic!(
                "theme-value support frame {now} was indeterminate: report={:?}, cost={:?}",
                frame.report(),
                frame.cost_report()
            )
        }
        Ok(crate::mounting::UiMountedFrameOutcome::Superseded(_)) => {
            panic!("theme-value support frame {now} was superseded")
        }
        Ok(crate::mounting::UiMountedFrameOutcome::RetentionDenied(_)) => {
            panic!("theme-value support frame {now} was retention denied")
        }
        Ok(crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_)) => {
            panic!("theme-value support frame {now} was admission denied")
        }
        Ok(crate::mounting::UiMountedFrameOutcome::CompletionDenied(_)) => {
            panic!("theme-value support frame {now} was completion denied")
        }
        Err(_) => panic!("theme-value support frame {now} stopped before publication"),
    }
}

pub(super) fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token = theme_token();
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
    let identity =
        crate::capability::UiThemeDefinitionIdentity::new("theme.truth.initial").unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        [(token, typed_color([68, 85, 102, 255]))],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition])
        .unwrap()
}

pub(super) fn theme_token() -> crate::capability::ThemeTokenId {
    crate::capability::ThemeTokenId::new(component_support::APPEARANCE_TOKEN).unwrap()
}

pub(super) fn legacy_color(hex: &str) -> crate::capability::ThemeTokenValue {
    crate::capability::ThemeTokenValue::color(crate::capability::ThemeColorValue::hex(hex).unwrap())
}

pub(super) fn typed_color(channels: [u8; 4]) -> worth_ui_dsl::UiThemeValue {
    worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(channels))
}

pub(super) fn theme_change(
    token: crate::capability::ThemeTokenId,
    revision: u64,
    value: crate::capability::ThemeTokenValue,
) -> crate::facade::entry::UiNativeThemeTokenValueChange {
    crate::facade::entry::UiNativeThemeTokenValueChange::successor(token, revision, value).unwrap()
}
