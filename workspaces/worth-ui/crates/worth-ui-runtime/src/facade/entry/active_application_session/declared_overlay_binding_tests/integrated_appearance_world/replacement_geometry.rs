use super::{authored, session::World};
use crate::capability::*;

#[test]
fn changed_layout_under_identical_regions_denies_stale_geometry_before_presentation() {
    let mut world = World::launch();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let predecessor = world.session.active_generation_identity();
    let region = world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings()
        .region_named("workspace.surface.overlay", "workspace.region.primary")
        .unwrap();
    let bounds = world
        .session
        .mounted
        .current_region_extents(world.surfaces[0], predecessor.prepared_generation(), region)
        .unwrap();
    let paint = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .cloned();
    let calls = world.host.presentation_calls();
    // Only the actual executed sizing reference changes. Region declarations,
    // mount identities and the capability catalog are the same on both sides.
    let source = authored::source().replacen(
        "sizing workspace.sizing.mosaic_support;",
        "sizing workspace.sizing.mosaic_support.alternate;",
        1,
    );
    assert_ne!(source, authored::source());
    let submission =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission(
            crate::runtime::WorthUiSourceProvider::in_memory("integrated-overlay")
                .with_file("app/main.wui", &source),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                "integrated-overlay",
            )],
            world.session.application.capabilities(),
        );
    let mut candidate = world.session.prepare_replacement(submission).unwrap();
    let catalog = world
        .session
        .admit_native_replacement_allocation_catalog(&mut candidate)
        .unwrap();
    let lowered = world
        .session
        .lower_prepared_replacement(*candidate)
        .unwrap();
    let pending = world.session.stage_prepared_replacement(lowered).unwrap();
    let boundary = world
        .session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_completion()
        .into_execution()
        .unwrap()
        .into_activation_boundary();
    let denial = match world.session.prepare_mounted_replacement(
        pending,
        catalog,
        boundary,
        None,
        crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("old coordinates cannot prove changed layout"),
    };
    assert!(
        matches!(
            denial,
            crate::facade::WorthUiApplicationCutoverDenial::OccurrenceGeometry(
                crate::mounting::UiMountedOccurrenceGeometryDenial::UnknownExecutedRegion
            )
        ),
        "{denial:?}"
    );
    assert_eq!(world.session.active_generation_identity(), predecessor);
    assert_eq!(world.host.presentation_calls(), calls);
    assert_eq!(
        world
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap(),
        paint.as_ref()
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_region_extents(world.surfaces[0], predecessor.prepared_generation(), region)
            .unwrap(),
        bounds
    );
    let _ = world.session.shutdown();
}

pub(super) fn alternate_sizing() -> MosaicSizingContractDescriptor {
    MosaicSizingContractDescriptor::new(
        MosaicSizingContractId::new("workspace.sizing.mosaic_support.alternate").unwrap(),
        MosaicSizingKind::fill(),
    )
    .with_measurement_authority(MosaicMeasurementAuthority::runtime_token())
    .with_resize_permission(MosaicResizePermission::user_resizable())
    .with_persistence(MosaicSizingPersistence::restorable())
    .with_overflow_behavior(MosaicOverflowBehavior::scroll_when_constrained())
    .with_parent_growth_behavior(MosaicParentGrowthBehavior::does_not_force_parent())
    .with_viewport_constraint(MosaicViewportConstraint::clamp_to_viewport())
    .with_named_measurement(NamedMeasurementDefinition::new(
        NamedMeasurementToken::new("workspace.measurement.mosaic_support.alternate").unwrap(),
        MeasurementValue::logical_pixels(360),
        MeasurementConstraint::between(
            MeasurementValue::logical_pixels(240),
            MeasurementValue::logical_pixels(720),
        ),
    ))
}
