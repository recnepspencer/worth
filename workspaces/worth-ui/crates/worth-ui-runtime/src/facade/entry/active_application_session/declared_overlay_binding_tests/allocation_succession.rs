pub(super) fn assert_preserved(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    stale_commit: crate::runtime::portal::UiPortalOverlayBindingCommit,
) {
    let before = session.authored_overlay_binding_exports().unwrap();
    assert_eq!(before.len(), 1);
    let surface = before[0].runtime_surface();
    let generation = session.active_generation_identity();
    let stack = session.portal.as_ref().unwrap().stack_snapshot();
    let graph = session.application.graph();
    let node = before[0].rows()[0].portal().owner().graph_node();
    let prior = graph
        .lookup()
        .graph_node(node)
        .unwrap()
        .value()
        .participation_posture()
        .axis(crate::graph::UiGraphParticipationAxis::Mounted);
    let transition = graph
        .mount_eligibility_transition_for_node(
            node,
            prior,
            crate::graph::UiGraphAxisParticipation::runtime_mutation(
                crate::graph::UiGraphParticipationStatus::Admitted,
            ),
        )
        .unwrap();
    let graph_commit = graph
        .commit_mount_eligibility_admissions(vec![transition])
        .unwrap();
    let stale_graph_successor = session
        .application
        .prepare_graph_successor(graph_commit)
        .unwrap();
    drop(
        session
            .authored_overlay_bindings
            .prepare_graph_succession(&stale_graph_successor)
            .unwrap(),
    );
    let nodes = session.graph().node_identities().collect::<Vec<_>>();
    for node in nodes {
        let node = session.mounted_graph_node(node).unwrap();
        if session.mounted_instances_for(node).unwrap().is_empty() {
            session.mount_instance(node, surface).unwrap();
        }
    }

    let failed = session.establish_mounted_allocation_catalog(1, []);
    assert!(
        matches!(
        failed,
        Err(crate::facade::entry::WorthUiMountedAllocationEstablishmentDenial::
            MissingMeasurementRequest(_))
    ),
        "unexpected allocation failure: {failed:?}"
    );
    assert_eq!(session.active_generation_identity(), generation);
    assert_eq!(session.authored_overlay_binding_exports().unwrap(), before);

    let assumptions = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        session.host_measurement_capability().capability_report(),
        1,
        2,
        3,
        4,
    );
    let request = crate::facade::entry::UiMountedAllocationMeasurementRequest::new(
        worth_ui_host_contract::UiMeasurementEvidenceFamily::ViewportExtent,
        crate::host::UiHostMeasurementNeed::ViewportExtent(
            worth_ui_host_contract::UiViewportExtentRequest,
        ),
        crate::host::UiHostMeasurementNormalizationContext::viewport_logical_exact(assumptions),
    );
    session
        .establish_mounted_allocation_catalog(1, [request])
        .expect("real allocation activation should carry Portal binding generation");

    let after = session.authored_overlay_binding_exports().unwrap();
    assert_ne!(session.active_generation_identity(), generation);
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].generation(), session.generation_identity());
    assert_eq!(after[0].runtime_surface(), before[0].runtime_surface());
    assert_eq!(after[0].rows(), before[0].rows());
    assert_eq!(after[0].portal_revision(), before[0].portal_revision());
    assert_eq!(session.portal.as_ref().unwrap().stack_snapshot(), stack);
    assert!(matches!(
        session
            .authored_overlay_bindings
            .prepare_graph_succession(&stale_graph_successor),
        Err(crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial::ForeignGeneration)
    ));
    assert!(matches!(
        session.commit_authored_overlay_binding(stale_commit),
        Err(crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial::TransitionMismatch)
    ));
    assert_eq!(session.authored_overlay_binding_exports().unwrap(), after);
}
