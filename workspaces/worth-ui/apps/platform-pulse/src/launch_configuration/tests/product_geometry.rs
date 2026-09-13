use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentId, ComponentViewportAxisPlacement,
};
use worth_ui_platform_pulse::product_world::PlatformPulseProductComponent as Component;

pub(super) fn assert_service_and_native_tiles(
    capabilities: &worth_ui::facade::diagnostics::CapabilitySnapshot,
) {
    let component = |identity: Component| {
        capabilities
            .components()
            .get(&ComponentId::new(identity.id()).unwrap())
            .unwrap()
    };
    // Product oracle, in viewport points: the service tile ends at the native
    // card's left edge. Half-open bounds are disjoint; the seam contract gives
    // Service sole paint ownership. The native pixel oracle checks that Native
    // starts with fill beside Service's border. Also verify viewport resizing.
    for (extent, expected_service, expected_native) in [
        ([960, 600], [264, 104, 656, 528], [656, 328, 936, 528]),
        ([1120, 720], [264, 104, 816, 648], [816, 328, 1096, 648]),
    ] {
        assert_eq!(
            rect(
                component(Component::ServiceStage)
                    .allocation_measurement_contract()
                    .unwrap(),
                extent
            ),
            expected_service
        );
        assert_eq!(
            rect(
                component(Component::NativeCard)
                    .allocation_measurement_contract()
                    .unwrap(),
                extent
            ),
            expected_native
        );
        assert_eq!(expected_service[2], expected_native[0]);
    }
    assert_eq!(
        component(Component::ServiceStage).surface_paint_order(),
        Some(2)
    );
    assert_eq!(
        component(Component::NativeCard).surface_paint_order(),
        Some(2)
    );
    let seam = capabilities.mosaic_regions().seam_paint().unwrap();
    assert_eq!(seam.shared_edges().len(), 1);
    assert_eq!(seam.owners().len(), 1);
    assert_eq!(
        seam.owners()[0].owner().as_str(),
        "platform.pulse.mosaic.region.service_tile"
    );
    // The old expanded rectangles are geometry declarations only. They must
    // never regain a paint rank to hide the two-pixel overlap they would create.
    for identity in [Component::ServiceStageBorder, Component::NativeCardBorder] {
        assert_eq!(component(identity).surface_paint_order(), None);
    }
}

fn rect(contract: ComponentAllocationMeasurementContract, extent: [i32; 2]) -> [i32; 4] {
    let ComponentAllocationMeasurementContract::ViewportRegion(region) = contract else {
        panic!("product tiles must follow the viewport");
    };
    let [left, right] = edges(region.horizontal(), extent[0]);
    let [top, bottom] = edges(region.vertical(), extent[1]);
    [left, top, right, bottom]
}

fn edges(axis: ComponentViewportAxisPlacement, extent: i32) -> [i32; 2] {
    match axis {
        ComponentViewportAxisPlacement::StretchBetween {
            start_logical_points,
            end_logical_points,
        } => [
            i32::from(start_logical_points),
            extent - i32::from(end_logical_points),
        ],
        ComponentViewportAxisPlacement::FixedFromEnd {
            end_logical_points,
            extent_logical_points,
        } => {
            let end = extent - i32::from(end_logical_points);
            [end - i32::from(extent_logical_points), end]
        }
        ComponentViewportAxisPlacement::FixedFromStart { .. } => {
            panic!("these tiles resize from the bottom and right edges")
        }
    }
}
