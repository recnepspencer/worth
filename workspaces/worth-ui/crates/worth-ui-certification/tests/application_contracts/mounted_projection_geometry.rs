use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedCanonicalBoxInput, UiMountedClipProjection,
    UiMountedClipReference, UiMountedClipRow, UiMountedClipTable, UiMountedCoordinateSpace,
    UiMountedFrameRequest, UiMountedGeometryDenial, UiMountedGeometryPosture,
    UiMountedLayerProjection, UiMountedLayerRow, UiMountedLayerTable,
    UiMountedTableProjectionStatus, UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile,
};
use worth_ui_test_support::WorthUiFrameworkTurnCertificationExt;
use worth_ui_test_support::WorthUiMountedFrameExecutionCertificationExt;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use super::mounted_application_lifecycle::known_empty_surface_world::{
    active_session, first_node, registered_surface,
};

#[test]
fn canonical_geometry_rejects_invalid_values_and_names_known_posture() {
    let canonical = |x, y, width, height, coordinate_space| {
        worth_ui_runtime::facade::mounted::UiMountedCanonicalBox::canonicalize(
            UiMountedCanonicalBoxInput {
                x,
                y,
                width,
                height,
                coordinate_space,
            },
        )
    };
    assert_eq!(
        canonical(f32::NAN, 0.0, 1.0, 1.0, UiMountedCoordinateSpace::Viewport),
        Err(UiMountedGeometryDenial::NonFinite)
    );
    assert_eq!(
        canonical(0.0, 0.0, -1.0, 1.0, UiMountedCoordinateSpace::Viewport),
        Err(UiMountedGeometryDenial::NegativeExtent)
    );
    let empty = canonical(0.0, 0.0, 0.0, 12.0, UiMountedCoordinateSpace::HostSurface).unwrap();
    let offscreen = canonical(-20.0, 2.0, 10.0, 12.0, UiMountedCoordinateSpace::Viewport).unwrap();
    let portal = canonical(
        -20.0,
        2.0,
        10.0,
        12.0,
        UiMountedCoordinateSpace::PortalLayer,
    )
    .unwrap();

    assert_eq!(empty.posture(), UiMountedGeometryPosture::Empty);
    assert_eq!(offscreen.posture(), UiMountedGeometryPosture::Offscreen);
    assert_eq!(portal.posture(), UiMountedGeometryPosture::Area);
    assert_eq!(
        portal.coordinate_space(),
        UiMountedCoordinateSpace::PortalLayer
    );
}

#[test]
fn portable_clip_and_layer_tables_preserve_nested_and_overlay_meaning() {
    let outer = worth_ui_runtime::facade::mounted::UiMountedCanonicalBox::canonicalize(
        UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        },
    )
    .unwrap();
    let inner = worth_ui_runtime::facade::mounted::UiMountedCanonicalBox::canonicalize(
        UiMountedCanonicalBoxInput {
            x: 24.0,
            y: 48.0,
            width: 320.0,
            height: 180.0,
            coordinate_space: UiMountedCoordinateSpace::PortalLayer,
        },
    )
    .unwrap();
    let clips = UiMountedClipTable::produced(vec![
        UiMountedClipRow::new(outer, None),
        UiMountedClipRow::new(inner, Some(UiMountedClipReference::new(0))),
    ]);
    let layers = UiMountedLayerTable::produced(vec![
        UiMountedLayerRow::new(
            10,
            UiMountedClipProjection::Clip(UiMountedClipReference::new(0)),
        ),
        UiMountedLayerRow::new(
            20,
            UiMountedClipProjection::Clip(UiMountedClipReference::new(1)),
        ),
    ]);

    assert_eq!(clips.status(), UiMountedTableProjectionStatus::Produced);
    assert_eq!(
        clips.rows()[1].parent(),
        Some(UiMountedClipReference::new(0))
    );
    assert_eq!(layers.status(), UiMountedTableProjectionStatus::Produced);
    assert_eq!(layers.rows()[0].semantic_order(), 10);
    assert_eq!(layers.rows()[1].semantic_order(), 20);
}

#[test]
fn unavailable_layer_truth_and_stale_binding_deny_without_moving_predecessor() {
    let mut session = active_session();
    let surface = registered_surface(&mut session);
    let stale_binding =
        session.inspect_mounted_identity().surface_bindings()[0].binding_generation();
    let node = first_node(&session);
    session.mount_instance(node, surface).unwrap();
    let current_binding = session
        .rebind_host_surface(
            stale_binding,
            UiHostSurfacePresentationMode::RecordOnly,
            UiSurfaceBindingProfile::new(
                2_000,
                UiSurfaceBindingCoordinatePosture::PhysicalPixels,
                2,
            )
            .unwrap(),
        )
        .unwrap()
        .binding_generation();
    let predecessor = session.advance_mounted_identity_frame().unwrap();
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let candidate = session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn permits projection"))
        .prepare_mounted_frame(UiMountedFrameRequest::all_bound_surfaces())
        .unwrap();

    assert!(candidate
        .surfaces()
        .iter()
        .all(|receipt| receipt.requirement().binding() != stale_binding));
    assert_eq!(
        session.inspect_mounted_identity().current_frame(),
        Some(predecessor)
    );
    let view = candidate
        .surfaces()
        .iter()
        .find(|receipt| receipt.requirement().binding() == current_binding)
        .unwrap()
        .projection();
    assert_eq!(
        view.clips().status(),
        UiMountedTableProjectionStatus::Produced
    );
    assert_eq!(
        view.layers().status(),
        UiMountedTableProjectionStatus::Produced
    );
    assert!(view
        .paint_batches()
        .rows()
        .iter()
        .all(|row| matches!(row.layer(), UiMountedLayerProjection::Layer(_))));
}
