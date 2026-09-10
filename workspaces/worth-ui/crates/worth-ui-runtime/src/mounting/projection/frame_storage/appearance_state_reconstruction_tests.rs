use super::UiMountedAppearanceFrameState;
use crate::mounting::projection::appearance::UiMountedAppearanceGeometryScope;
use crate::mounting::projection::frame_storage::UiMountedAppearanceNodeInputContext;

fn known_allocation() -> worth_ui_host_contract::UiMountedAllocationProjection {
    worth_ui_host_contract::UiMountedAllocationProjection::Known {
        bounds: worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
            worth_ui_host_contract::UiMountedCanonicalBoxInput {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 80.0,
                coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
            },
        )
        .unwrap(),
        basis: worth_ui_host_contract::UiMountedAllocationBasis::new(
            1,
            1,
            3,
            worth_ui_host_contract::UiMountedTransformProjection::Identity,
        ),
    }
}

fn retained_context(
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    fixture: &crate::mounting::projection::appearance::MountedAppearanceReconstructionTestFixture,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
) -> crate::runtime::appearance::UiAppearanceAttemptContext {
    let target = crate::runtime::appearance::UiAppearanceTarget::new(
        session,
        fixture.surface,
        fixture.graph_node,
        fixture.instance,
        incarnation,
        fixture.receipt,
    )
    .unwrap();
    crate::runtime::appearance::UiAppearanceAttemptContext::new(
        target,
        fixture.frame,
        fixture.issuer,
        7,
        known_allocation(),
        crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
        Some(0),
        None,
        Box::new([]),
        generation.clone(),
        0,
        0,
    )
}

fn successor_node(
    fixture: &crate::mounting::projection::appearance::MountedAppearanceReconstructionTestFixture,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
) -> (
    worth_ui_host_contract::UiMountedFrameIdentity,
    worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    UiMountedAppearanceNodeInputContext,
) {
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let receipt = issuer.receipt_for(fixture.instance);
    (
        frame,
        receipt,
        UiMountedAppearanceNodeInputContext {
            geometry_input: None,
            frame,
            semantic_surface: fixture.surface,
            mounted_instance: fixture.instance,
            graph_node: fixture.graph_node,
            incarnation,
            node_receipt: receipt,
            issuer,
            plan_digest: 7,
            surface_paint_order: Some(0),
            text_foreground_spans: Box::new([]),
            allocation: known_allocation(),
            appearance_clip:
                crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
        },
    )
}

#[test]
fn reconstruction_uses_successor_receipt_and_retained_projection_facts() {
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
    let mut retained_fixture =
        crate::mounting::projection::appearance::mounted_sidecar_with_retained_facts_for_test();
    let incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
    let retained = retained_context(
        session_identity,
        &generation,
        &retained_fixture,
        incarnation,
    );
    let retained_overlay = retained_fixture
        .sidecar
        .current_overlay_order()
        .expect("retained mounted facts include overlay order");
    let mut state = UiMountedAppearanceFrameState::default();
    state.retain_projection_for_test(&retained, projection);
    state.replace_sidecar_for_test(&retained, std::mem::take(&mut retained_fixture.sidecar));

    let (successor_frame, successor_receipt, successor_node) =
        successor_node(&retained_fixture, incarnation);
    state.prepare_reconstruction(vec![successor_node]);
    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let records = state
        .lower(
            presentation,
            &UiMountedAppearanceGeometryScope::new(&[], None),
        )
        .unwrap();

    assert_eq!(records.len(), 1);
    match &records[0] {
        crate::runtime::appearance::UiAppearanceInspectionRecord::Projection {
            consumers_selected,
            receipt,
            ..
        } => {
            assert_eq!(*consumers_selected, 0);
            assert!(receipt.mounting_result_available());
            assert!(!receipt.mounted_mechanical_output_changed());
            assert!(!receipt.equal_output_suppressed());
        }
        crate::runtime::appearance::UiAppearanceInspectionRecord::Denial { denial, .. } => {
            panic!("reconstruction was denied: {denial:?}");
        }
    }
    let rebuilt = &state
        .retained_entry_for_test(&retained)
        .expect("reconstructed entry remains retained")
        .sidecar;
    assert_eq!(rebuilt.current_frame_identity(), Some(successor_frame));
    assert_eq!(rebuilt.current_overlay_order(), Some(retained_overlay));
    assert_eq!(
        rebuilt.current_node_receipts(),
        vec![successor_receipt].into_boxed_slice()
    );
    assert!(!rebuilt
        .last_delta_mechanics_changed()
        .expect("reconstruction records a delta summary"));
    let _ = session.shutdown();
}

#[path = "appearance_state_reconstruction_tests/physical_only.rs"]
mod physical_only;

#[test]
fn reconstruction_matches_reversed_nodes_and_denies_missing_nodes_without_erasing_facts() {
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
    let mut first_fixture =
        crate::mounting::projection::appearance::mounted_sidecar_with_retained_facts_for_test();
    let mut second_fixture =
        crate::mounting::projection::appearance::mounted_sidecar_with_retained_facts_for_test();
    let first_incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
    let second_incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
    let first = retained_context(
        session_identity,
        &generation,
        &first_fixture,
        first_incarnation,
    );
    let second = retained_context(
        session_identity,
        &generation,
        &second_fixture,
        second_incarnation,
    );
    let (_, _, first_node) = successor_node(&first_fixture, first_incarnation);
    let (_, _, second_node) = successor_node(&second_fixture, second_incarnation);
    let mut state = UiMountedAppearanceFrameState::default();
    state.retain_projection_for_test(&first, projection.clone());
    state.retain_projection_for_test(&second, projection);
    state.replace_sidecar_for_test(&first, std::mem::take(&mut first_fixture.sidecar));
    state.replace_sidecar_for_test(&second, std::mem::take(&mut second_fixture.sidecar));

    state.prepare_reconstruction(vec![second_node, first_node]);
    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let records = state
        .lower(
            presentation,
            &UiMountedAppearanceGeometryScope::new(&[], None),
        )
        .unwrap();
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| matches!(
        record,
        crate::runtime::appearance::UiAppearanceInspectionRecord::Projection { .. }
    )));
    assert_eq!(state.membership_counts(), (2, 0, 0));
    assert!(state.retained_entry_for_test(&first).is_some());
    assert!(state.retained_entry_for_test(&second).is_some());

    let first_facts_before_missing = state
        .retained_entry_for_test(&first)
        .and_then(|entry| entry.sidecar.current().cloned())
        .expect("first retained facts exist before missing-node reconstruction");
    let first_receipts_before_missing = state
        .retained_entry_for_test(&first)
        .expect("first retained entry exists before missing-node reconstruction")
        .sidecar
        .current_node_receipts();
    let second_facts_before_missing = state
        .retained_entry_for_test(&second)
        .and_then(|entry| entry.sidecar.current().cloned())
        .expect("second retained facts exist before missing-node reconstruction");
    let second_receipts_before_missing = state
        .retained_entry_for_test(&second)
        .expect("second retained entry exists before missing-node reconstruction")
        .sidecar
        .current_node_receipts();

    state.prepare_reconstruction(Vec::new());
    let records = state
        .lower(
            presentation,
            &UiMountedAppearanceGeometryScope::new(&[], None),
        )
        .unwrap();
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| matches!(
        record,
        crate::runtime::appearance::UiAppearanceInspectionRecord::Denial {
            denial: crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
            ..
        }
    )));
    assert_eq!(state.membership_counts(), (2, 0, 0));
    let first_after_missing = state
        .retained_entry_for_test(&first)
        .expect("first retained entry survives missing-node reconstruction");
    assert_eq!(
        first_after_missing.sidecar.current(),
        Some(&first_facts_before_missing)
    );
    assert_eq!(
        first_after_missing.sidecar.current_node_receipts(),
        first_receipts_before_missing
    );
    let second_after_missing = state
        .retained_entry_for_test(&second)
        .expect("second retained entry survives missing-node reconstruction");
    assert_eq!(
        second_after_missing.sidecar.current(),
        Some(&second_facts_before_missing)
    );
    assert_eq!(
        second_after_missing.sidecar.current_node_receipts(),
        second_receipts_before_missing
    );
    let _ = session.shutdown();
}
