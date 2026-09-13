use super::{fixture, prepare, set_text, text_contract};
use worth_ui_host_contract::*;

#[path = "appearance_projection_text_succession_tests.rs"]
mod succession;

#[test]
fn graph_replacement_publishes_pending_text_or_retires_it_only_after_acceptance() {
    for (remove, delayed) in [(false, false), (true, false), (false, true), (true, true)] {
        let role = fixture::foreground_role();
        let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(text_contract()));
        let authored = format!(
            "component:{}",
            super::super::super::support::APPEARANCE_NODE_A
        );
        let graph = session
            .graph()
            .snapshot()
            .nodes()
            .iter()
            .find(|node| node.declaration_identity().authored_semantic_name() == authored)
            .unwrap()
            .graph_node_identity();
        let (surface, _) =
            super::super::super::mounting_fixture::mount_only_graph_node(&mut session, graph);
        assert_eq!(session.mounted.view().mounted_instances().len(), 1);
        let second = add_surface(&mut session, graph);
        set_text(&mut session, 0, "AB");
        fixture::close_source(&mut session, &role, "replacement-text-initial");
        let initial = prepare(&mut session);
        fixture::publish(&mut session, &host, initial, 1);
        assert_host_text(&host, surface, Some("AB"));
        assert_host_text(&host, second, Some("AB"));
        let predecessor = session.current_mounted_publication().unwrap().frame();
        set_text(&mut session, 1, "CD");

        let denied = prepare_replacement(
            &mut session,
            &role,
            remove,
            crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![surface]),
        );
        assert!(matches!(denied, Err(crate::facade::entry::WorthUiApplicationCutoverDenial::IncompleteMountedSurfaceScope)));
        drop(denied);
        assert_eq!(
            session.current_mounted_publication().unwrap().frame(),
            predecessor
        );
        for target in [surface, second] {
            assert_host_text(&host, target, Some("AB"));
        }
        let pending = session.presentation.project().unwrap().content();
        let Some(crate::mounting::UiMountedSemanticTextContent::Scalar(row)) = pending.get(graph)
        else {
            panic!("the exact pending component must survive subset denial");
        };
        let crate::mounting::UiMountedSemanticTextValueDirective::Replace(value) = row.value()
        else {
            panic!("the pending successor must retain its replacement value");
        };
        assert_eq!(value.as_ref(), "CD");
        let crate::facade::entry::WorthUiMountedReplacementPreparationOutcome::Prepared(
            replacement,
        ) = prepare_replacement(
            &mut session,
            &role,
            remove,
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
        )
        .unwrap_or_else(|denial| panic!("remove={remove}: {denial:?}"))
        else {
            panic!("changed source prepares a real graph successor");
        };
        host.push_rejected();
        host.push_rejected();
        let crate::facade::entry::WorthUiMountedApplicationReplacementOutcome::RejectedBeforeEffects(rejected) =
            replacement.present(UiPresentationDeadline::at_tick(100), 2)
        else { panic!("host rejects before effects"); };
        assert_host_text(&host, surface, Some("AB"));
        assert_host_text(&host, second, Some("AB"));
        let replacement = rejected.into_replacement();
        for _ in [surface, second] {
            if delayed {
                host.push_in_flight(
                    vec![crate::certification_support::ScriptedSurfaceCompletion::Pending,
                        crate::certification_support::ScriptedSurfaceCompletion::Presented(
                            UiMountedSurfacePresentationCompletion::new(
                                crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                                crate::certification_support::scripted_presentation_epoch(),
                                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                                UiHostPresentationCostReport::default(),
                            ),
                        )],
                    UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
                );
            } else {
                host.push_native_display_presented();
            }
        }
        let outcome = replacement.present(UiPresentationDeadline::at_tick(100), 3);
        let outcome = if delayed {
            let crate::facade::entry::WorthUiMountedApplicationReplacementOutcome::InFlight(
                pending,
            ) = outcome
            else {
                panic!("replacement must retain the asynchronous host attempt");
            };
            let pending = pending.detach();
            assert_eq!(
                session.current_mounted_publication().unwrap().frame(),
                predecessor
            );
            assert_accepted_text(&session, surface, Some("AB"));
            assert_accepted_text(&session, second, Some("AB"));
            assert_host_text(&host, surface, Some("AB"));
            assert_host_text(&host, second, Some("AB"));
            let crate::facade::entry::WorthUiMountedApplicationReplacementOutcome::InFlight(
                pending,
            ) = pending.complete(&mut session, 4)
            else {
                panic!("the first completion poll must remain pending");
            };
            let pending = pending.detach();
            assert_eq!(
                session.current_mounted_publication().unwrap().frame(),
                predecessor
            );
            assert!(!session.presentation.project().unwrap().content().is_empty());
            assert_accepted_text(&session, surface, Some("AB"));
            assert_accepted_text(&session, second, Some("AB"));
            pending.complete(&mut session, 5)
        } else {
            outcome
        };
        let crate::facade::entry::WorthUiMountedApplicationReplacementOutcome::Published {
            mounted,
            ..
        } = outcome
        else {
            panic!("exact prepared replacement retry must publish");
        };
        assert_ne!(mounted.frame(), predecessor);
        for target in [surface, second] {
            assert_host_text(&host, target, (!remove).then_some("CD"));
            assert_eq!(
                host.accepted_text_commands(target).unwrap().0,
                mounted.attempt(),
                "each host completion consumed this exact published attempt"
            );
        }
        assert_accepted_text(&session, surface, (!remove).then_some("CD"));
        assert_accepted_text(&session, second, (!remove).then_some("CD"));
        assert!(
            session.presentation.project().unwrap().content().is_empty(),
            "accepted successor consumes its exact revision or retires the removed binding"
        );
        if remove {
            assert!(
                session
                    .presentation
                    .project_complete()
                    .unwrap()
                    .content()
                    .is_empty(),
                "a removed component must not return as stale authoritative text"
            );
        }
        let _ = session.shutdown();
        assert_eq!(host.pending_presentation_count(), 0);
    }
}

fn assert_accepted_text(
    session: &crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    expected: Option<&str>,
) {
    let accepted = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let projection = session.mounted.current_projection_rc_for_test().unwrap();
    let view = projection.view_for(accepted.binding()).unwrap();
    let values = view
        .semantic_text()
        .rows()
        .iter()
        .filter(|row| row.slot() == UiSemanticTextSlot::Value)
        .map(|row| row.text())
        .collect::<Vec<_>>();
    assert_eq!(values, expected.into_iter().collect::<Vec<_>>());
}

fn assert_host_text(
    host: &crate::certification_support::ScriptedPresentationHost,
    surface: UiSemanticSurfaceIdentity,
    expected: Option<&str>,
) {
    let (_, commands) = host.accepted_text_commands(surface).unwrap();
    let values: Vec<_> = commands
        .iter()
        .filter_map(|command| match command {
            UiMountedPaintCommand::SemanticText { mechanic, .. }
                if mechanic.slot() == UiSemanticTextSlot::Value =>
            {
                Some(mechanic.text())
            }
            _ => None,
        })
        .collect();
    assert_eq!(values, expected.into_iter().collect::<Vec<_>>());
}

fn successor_source(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    remove: bool,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    use crate::runtime::tests::appearance_component_session_test_support as support;
    let original = fixture::source_text(role, false);
    let text = if remove {
        original
            .lines()
            .filter(|line| {
                !line.starts_with(&format!("component {} {{", support::APPEARANCE_NODE_A))
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        original.replace(
            &format!(
                "component {} {{ appearance {{ role test.focus.b }}",
                support::APPEARANCE_NODE_B
            ),
            &format!("component {} {{", support::APPEARANCE_NODE_B),
        )
    };
    assert_ne!(text, original);
    crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission(
        crate::runtime::WorthUiSourceProvider::in_memory("replacement-text-successor")
            .with_file("appearance/consumer.wui", text),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            "replacement-text-successor",
        )],
        session.capabilities(),
    )
}

fn prepare_replacement<'session>(
    session: &'session mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    remove: bool,
    request: crate::mounting::UiMountedFrameRequest,
) -> Result<
    crate::facade::entry::WorthUiMountedReplacementPreparationOutcome<'session>,
    crate::facade::entry::WorthUiApplicationCutoverDenial,
> {
    let source = successor_source(session, role, remove);
    let mut candidate = session.prepare_replacement(source).unwrap();
    let catalog = session
        .admit_native_replacement_allocation_catalog(&mut candidate)
        .unwrap();
    let lowered = session.lower_prepared_replacement(*candidate).unwrap();
    let pending = session.stage_prepared_replacement(lowered).unwrap();
    let boundary = session
        .execute_framework_turn(|_| {})
        .unwrap_or_else(|_| panic!("replacement turn"))
        .into_completion()
        .into_execution()
        .unwrap_or_else(|_| panic!("replacement execution"))
        .into_activation_boundary();
    session.prepare_mounted_replacement(pending, catalog, boundary, None, request)
}

fn add_surface(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    graph: crate::graph::UiGraphNodeIdentity,
) -> UiSemanticSurfaceIdentity {
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
    let node = session.mounted_graph_node(graph).unwrap();
    session.mount_instance(node, surface).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(session, surface);
    surface
}
