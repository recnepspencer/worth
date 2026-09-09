use super::*;

#[test]
fn operability_route_removal_clears_standing_facts_at_application_cutover() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role, 1);
    let (surface, graph) = super::super::mounting_fixture::mount(&mut session, 1_000);
    close(&mut session, &role, 1, "operability-replacement-initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = prepare(&mut session);
    publish(&mut session, &host, frame, 1);
    let (instance, _) = activate(&mut session, surface, 1);
    close(&mut session, &role, 1, "operability-replacement-ready");
    let frame = project(
        &mut session,
        &[(instance, 10)],
        &[(
            surface,
            Some((instance, UiPointerAffordanceFamily::Activation)),
        )],
    );
    publish(&mut session, &host, frame, 2);
    let before = session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();
    let generation = session.active_generation_identity();
    assert!(before.fact_for(graph, instance, fixture::ROUTE).is_some());

    let source = fixture::candidate(&session, &role, 0, "operability-route-removed");
    let mut successor = session.prepare_replacement(source).unwrap();
    let catalog = session
        .admit_native_replacement_allocation_catalog(&mut successor)
        .unwrap();
    let lowered = session.lower_prepared_replacement(*successor).unwrap();
    let pending = session.stage_prepared_replacement(lowered).unwrap();
    let boundary = session
        .execute_framework_turn(|_| {})
        .unwrap_or_else(|_| panic!("replacement turn"))
        .into_completion()
        .into_execution()
        .unwrap_or_else(|_| panic!("replacement execution"))
        .into_activation_boundary();
    let prepared = session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
        )
        .unwrap();
    let crate::facade::entry::WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) =
        prepared
    else {
        panic!("removing a declared route changes application meaning");
    };
    host.push_native_display_settled_without_effects();
    assert!(matches!(
        replacement.present(UiPresentationDeadline::at_tick(100), 3),
        crate::facade::entry::WorthUiMountedApplicationReplacementOutcome::Published { .. }
    ));
    assert_ne!(session.active_generation_identity(), generation);
    assert!(session.pointer_affordance_snapshot.is_none());
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let pointer_removal = output.fragments().iter().find(|fragment| matches!(fragment.identity(),
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { surface: changed, .. } if changed == surface))
        .expect("generation replacement removes the prior pointer mechanic");
    assert!(pointer_removal.work().successor().mechanics().is_empty());
    assert_eq!(
        pointer_removal.work().changes(),
        &[UiMountedAppearanceMechanicChange::Remove(
            UiMountedAppearanceMechanicIdentity::Pointer {
                pointer: UiHostPointerIdentity::new(1),
                surface,
                target: instance
            }
        )]
    );
    assert!(
        session
            .mounted
            .current_mounted_identity_basis(instance)
            .is_none(),
        "route removal retires the old mounted declaration incarnation"
    );
    assert!(session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap()
        .facts()
        .is_empty());
    assert!(
        before.fact_for(graph, instance, fixture::ROUTE).is_some(),
        "predecessor snapshot stays immutable"
    );
    let successor_graph = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.appearance_role_attachment().is_some())
        .expect("the successor still declares the appearance control")
        .graph_node_identity();
    assert_eq!(
        session
            .application
            .prepared_authority()
            .intent_catalog()
            .single_product_route_identity(successor_graph),
        Err(crate::declaration::UiIntentSingleProductRouteDenial::Missing)
    );
    let _ = session.shutdown();
}
