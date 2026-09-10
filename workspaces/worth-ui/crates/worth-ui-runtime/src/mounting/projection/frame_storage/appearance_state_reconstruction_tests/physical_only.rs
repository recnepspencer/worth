use super::*;

#[test]
fn physical_only_reconstruction_reissues_exact_paint_for_the_current_node() {
    let (session, binding, _target, vector, theme) =
        crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let session_identity = session.session_identity();
    let generation = session.active_generation_identity();
    let mut fixture =
        crate::mounting::projection::appearance::mounted_sidecar_with_retained_facts_for_test();
    let incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
    let retained = retained_context(session_identity, &generation, &fixture, incarnation);
    let mut state = UiMountedAppearanceFrameState::default();
    state.begin_epoch(session_identity, &generation, &[]);
    state.retain_projection_for_test(&retained, projection);
    state.replace_sidecar_for_test(&retained, std::mem::take(&mut fixture.sidecar));
    state.members.clear_for_epoch();

    let (successor_frame, successor_receipt, mut node) = successor_node(&fixture, incarnation);
    let successor_graph =
        crate::graph::UiGraphNodeIdentity::new(fixture.graph_node.digest().wrapping_add(1));
    node.graph_node = successor_graph;
    let successor_issuer = node.issuer;
    state.prepare_reconstruction(vec![node.clone()]);
    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let records = state
        .lower(
            presentation,
            &UiMountedAppearanceGeometryScope::new(&[], None),
        )
        .unwrap();

    assert!(
        records.is_empty(),
        "physical replay does not forge semantic resolution"
    );
    let work = state.take_node_work();
    assert_eq!(work.len(), 1);
    assert_eq!(work[0].predecessor, Some(fixture.receipt));
    assert_eq!(work[0].successor, Some(successor_receipt));
    assert_eq!(
        work[0].work.posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Reconstruction
    );
    assert_eq!(work[0].work.predecessor(), Some(fixture.frame));
    assert_eq!(work[0].work.successor().frame(), successor_frame);
    assert!(work[0].work.changes().is_empty());
    assert!(work[0].work.damage().is_empty());
    assert!(work[0]
        .work
        .successor()
        .mechanics()
        .iter()
        .all(|mechanic| match mechanic {
            worth_ui_host_contract::UiMountedAppearanceMechanic::Surface(mechanic) => {
                mechanic.node_receipt() == successor_receipt
            }
            worth_ui_host_contract::UiMountedAppearanceMechanic::PortalSurface(mechanic) => {
                mechanic.surface().node_receipt() == successor_receipt
            }
            worth_ui_host_contract::UiMountedAppearanceMechanic::Outline(mechanic) => {
                mechanic.node_receipt() == successor_receipt
            }
            worth_ui_host_contract::UiMountedAppearanceMechanic::TextForeground(mechanic) => {
                mechanic.node_receipt() == successor_receipt
            }
            worth_ui_host_contract::UiMountedAppearanceMechanic::Pointer(_)
            | worth_ui_host_contract::UiMountedAppearanceMechanic::Backdrop(_) => false,
        }));
    assert_eq!(
        state.physical_node_receipts_for_test(),
        vec![successor_receipt]
    );

    let successor_target = crate::runtime::appearance::UiAppearanceTarget::new(
        session_identity,
        fixture.surface,
        successor_graph,
        fixture.instance,
        incarnation,
        successor_receipt,
    )
    .unwrap();
    let successor_context = crate::runtime::appearance::UiAppearanceAttemptContext::new(
        successor_target,
        successor_frame,
        successor_issuer,
        7,
        known_allocation(),
        crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
        Some(0),
        None,
        Box::new([]),
        generation.clone(),
        0,
        0,
    );
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                successor_context,
                crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
            ),
        )
        .expect("current semantic work can succeed the reissued physical predecessor");
    state.prepare_reconstruction(vec![node]);
    state
        .lower(
            presentation,
            &UiMountedAppearanceGeometryScope::new(&[], None),
        )
        .unwrap();
    let denied_reconstruction_work = state.take_node_work();
    assert_eq!(denied_reconstruction_work.len(), 1);
    assert_eq!(
        denied_reconstruction_work[0].work.posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Reconstruction
    );
    assert_eq!(
        state.physical_node_receipts_for_test(),
        vec![successor_receipt]
    );
    let _ = session.shutdown();
}
