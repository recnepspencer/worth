use crate::facade::entry::WorthUiCertificationApplicationTransition;
use crate::runtime::tests::appearance_component_session_test_support as support;
use crate::runtime::tests::appearance_theme_test_support;

#[test]
fn appearance_inspection_multisurface_real_surfaces_have_independent_current_worlds() {
    let role = support::validation_background_role(support::APPEARANCE_TOKEN);
    let (mut session, host) = mounted_two_surface_session(&role);
    let [first_surface, second_surface] = create_and_mount_surfaces(&mut session, &role);
    publish_initial_frame(&mut session, &host);
    let graph_node = appearance_graph_node(&session);

    let first_world = session.appearance_inspection_world(first_surface);
    let second_world = session.appearance_inspection_world(second_surface);
    assert_ne!(first_world, second_world);
    assert!(matches!(
        session.why_appearance(query(
            first_world,
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        )),
        worth_ui_inspection::UiAppearanceInspectionOutcome::Found(_)
    ));
    assert!(matches!(
        session.why_appearance(query(
            second_world,
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        )),
        worth_ui_inspection::UiAppearanceInspectionOutcome::Found(_)
    ));

    let predecessor_worlds = [first_world, second_world];
    let candidate = support::appearance_candidate_submission(
        &session,
        "appearance-inspection-multisurface-successor",
        Some(&role),
    );
    let mut observation = session.begin_observation_turn().unwrap();
    observation.admit_source(candidate).unwrap();
    let observations = observation.seal().unwrap();
    let evidence = match session.classify_observations(observations).unwrap() {
        crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) => {
            evidence
        }
        _ => panic!("equal authored semantics must produce an evidence-only succession"),
    };
    let plan = session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let prepared = session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(1),
        )
        .unwrap();
    let receipt = match prepared.execute(1) {
        crate::runtime::rebind::UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("the multisurface evidence-only succession must publish"),
    };
    assert_eq!(
        session.why_appearance(query(
            predecessor_worlds[0],
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        )),
        worth_ui_inspection::UiAppearanceInspectionOutcome::WrongWorld
    );
    assert_eq!(
        session.why_appearance(query(
            predecessor_worlds[1],
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        )),
        worth_ui_inspection::UiAppearanceInspectionOutcome::WrongWorld
    );
    refresh_appearance_owner_snapshot(
        &mut session,
        &role,
        "appearance-inspection-multisurface-successor-mounted",
    );
    publish_successor_frame(&mut session, &host);
    let successor_worlds = [
        session.appearance_inspection_world(first_surface),
        session.appearance_inspection_world(second_surface),
    ];
    for successor_world in successor_worlds {
        assert!(matches!(
            session.why_appearance(query(
                successor_world,
                graph_node.digest(),
                worth_ui_dsl::UiAppearanceAspect::Background,
            )),
            worth_ui_inspection::UiAppearanceInspectionOutcome::Found(_)
        ));
    }
    for predecessor_world in predecessor_worlds {
        assert_eq!(
            session.why_appearance(query(
                predecessor_world,
                graph_node.digest(),
                worth_ui_dsl::UiAppearanceAspect::Background,
            )),
            worth_ui_inspection::UiAppearanceInspectionOutcome::WrongWorld
        );
    }
    drop(receipt);

    let unrecorded_surface = session.create_semantic_surface().unwrap();
    let unrecorded_world = session.appearance_inspection_world(unrecorded_surface);
    assert_eq!(
        session.why_appearance(query(
            unrecorded_world,
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        )),
        worth_ui_inspection::UiAppearanceInspectionOutcome::WrongWorld
    );
    assert_eq!(
        session.why_appearance(query(
            successor_worlds[0],
            u64::MAX,
            worth_ui_dsl::UiAppearanceAspect::Background,
        )),
        worth_ui_inspection::UiAppearanceInspectionOutcome::Unavailable
    );
    assert_eq!(
        session.why_appearance(query(
            successor_worlds[0],
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Outline,
        )),
        worth_ui_inspection::UiAppearanceInspectionOutcome::Unavailable
    );

    let mut foreign = support::source_backed_static_paint_consumer_session();
    let foreign_surface = foreign.create_semantic_surface().unwrap();
    let foreign_world = foreign.appearance_inspection_world(foreign_surface);
    assert_eq!(
        session.why_appearance(query(
            foreign_world,
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        )),
        worth_ui_inspection::UiAppearanceInspectionOutcome::WrongWorld
    );
    let _ = foreign.shutdown();
    let _ = session.shutdown();
}

fn mounted_two_surface_session(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let observer = host.clone();
    let mut session = support::appearance_component_builder(role)
        .register_appearance_theme_bundle(appearance_theme_test_support::bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(support::appearance_fixture(role))
        .freeze()
        .map(|application| {
            WorthUiCertificationApplicationTransition::activate_test_host(application, host)
        })
        .expect("the multisurface appearance application should prepare")
        .launch()
        .expect("the multisurface appearance application should launch");
    let candidate = support::appearance_candidate_submission(
        &session,
        "appearance-inspection-multisurface",
        Some(role),
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    let mut session = session;
    session.classify_observations(observations).unwrap();
    (session, observer)
}

fn create_and_mount_surfaces(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> [worth_ui_host_contract::UiSemanticSurfaceIdentity; 2] {
    let surfaces = [
        session.create_semantic_surface().unwrap(),
        session.create_semantic_surface().unwrap(),
    ];
    for surface in surfaces {
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
    }
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
                .then_some((identity, Box::<str>::from(semantic)))
        })
        .collect::<Vec<_>>();
    for (graph_node, semantic) in graph_nodes {
        session
            .register_application_semantic_text(semantic, graph_node)
            .unwrap();
        let mounted_node = session.mounted_graph_node(graph_node).unwrap();
        for surface in surfaces {
            session.mount_instance(mounted_node, surface).unwrap();
        }
    }
    let capability = session.host_measurement_capability();
    let assumptions = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    session
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
        .unwrap_or_else(|_| panic!("the multisurface frame should publish"));
    for surface in surfaces {
        crate::facade::entry::mounted_occurrence_geometry_test_support::
            refresh_nonoverlapping_surface_geometry(session, surface);
    }
    refresh_appearance_owner_snapshot(session, role, "appearance-inspection-multisurface-mounted");
    surfaces
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

fn publish_initial_frame(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
) {
    session.advance_mounted_identity_frame().unwrap();
    host.push_native_display_presented();
    host.push_native_display_presented();
    let outcome = match session.execute_mounted_frame(
        crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        1,
        |_| {},
    ) {
        Ok(outcome) => outcome,
        Err(_) => panic!("two-surface frame execution should succeed"),
    };
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {}
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("two-surface frame was rejected before effects")
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => {
            panic!("two-surface frame remained in flight")
        }
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("two-surface frame presentation was indeterminate")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
            panic!("two-surface frame was superseded")
        }
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(_) => {
            panic!("two-surface frame retention was denied")
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_) => {
            panic!("two-surface frame admission was denied")
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("two-surface frame completion was denied")
        }
    }
}

fn publish_successor_frame(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
) {
    host.push_native_display_settled_without_effects();
    host.push_native_display_settled_without_effects();
    let outcome = match session.execute_mounted_frame(
        crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        2,
        |_| {},
    ) {
        Ok(outcome) => outcome,
        Err(_) => panic!("two-surface successor execution should succeed"),
    };
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {}
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("two-surface successor frame was rejected before effects")
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => {
            panic!("two-surface successor frame remained in flight")
        }
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("two-surface successor presentation was indeterminate")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
            panic!("two-surface successor frame was superseded")
        }
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(_) => {
            panic!("two-surface successor frame retention was denied")
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_) => {
            panic!("two-surface successor frame admission was denied")
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("two-surface successor frame completion was denied")
        }
    }
}

fn appearance_graph_node(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> crate::graph::UiGraphNodeIdentity {
    session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.appearance_role_attachment().is_some())
        .expect("the multisurface fixture has one appearance consumer")
        .graph_node_identity()
}

fn query(
    world: worth_ui_inspection::UiAppearanceInspectionWorld,
    graph_node_digest: u64,
    aspect: worth_ui_dsl::UiAppearanceAspect,
) -> worth_ui_inspection::UiAppearanceInspectionQuery {
    worth_ui_inspection::UiAppearanceInspectionQuery::new(world, graph_node_digest, aspect)
}
