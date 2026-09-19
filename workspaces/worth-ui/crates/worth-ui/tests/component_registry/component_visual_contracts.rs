use worth_ui::facade::{
    app::WorthUi,
    declaration::{
        ComponentAllocationMeasurementContract, ComponentDescriptor, ComponentHitTestContract,
        ComponentHitTestOrder, ComponentViewportInset,
    },
    diagnostics::CapabilityDiagnosticCode,
};

use super::component_registry_assertions::{
    assert_diagnostic_codes_and_identities, assert_registered_component_ids,
};
use super::component_registry_fixtures::component_descriptor;

#[test]
fn matching_paint_and_hit_allocations_admit_in_either_builder_order() {
    let allocation = ComponentAllocationMeasurementContract::fill_viewport();
    let first = paint_then_hit("workspace.component.visual_first", allocation, allocation);
    let second = hit_then_paint("workspace.component.visual_second", allocation, allocation);
    let app = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(first)
        .register_component(second)
        .freeze()
        .expect("matching visual allocation contracts should freeze");

    assert_registered_component_ids(
        app.capabilities().components(),
        &[
            "workspace.component.visual_first",
            "workspace.component.visual_second",
        ],
    );
}

#[test]
fn conflicting_paint_and_hit_allocations_reject_in_either_builder_order() {
    let fill = ComponentAllocationMeasurementContract::fill_viewport();
    let inset = ComponentAllocationMeasurementContract::viewport_inset(
        ComponentViewportInset::symmetric(8, 8),
    );
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(paint_then_hit(
            "workspace.component.visual_first",
            fill,
            inset,
        ))
        .register_component(hit_then_paint(
            "workspace.component.visual_second",
            fill,
            inset,
        ))
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_diagnostic_codes_and_identities(
        report.registration_diagnostics(),
        &[
            (
                CapabilityDiagnosticCode::ConflictingComponentAllocationContract,
                "workspace.component.visual_first",
            ),
            (
                CapabilityDiagnosticCode::ConflictingComponentAllocationContract,
                "workspace.component.visual_second",
            ),
        ],
    );
}

fn paint_then_hit(
    id: &str,
    paint_allocation: ComponentAllocationMeasurementContract,
    hit_allocation: ComponentAllocationMeasurementContract,
) -> ComponentDescriptor {
    component_descriptor(id)
        .with_allocation_measurement_contract(paint_allocation)
        .with_surface_paint_order(0)
        .with_hit_test(hit_contract(hit_allocation))
}

fn hit_then_paint(
    id: &str,
    paint_allocation: ComponentAllocationMeasurementContract,
    hit_allocation: ComponentAllocationMeasurementContract,
) -> ComponentDescriptor {
    component_descriptor(id)
        .with_hit_test(hit_contract(hit_allocation))
        .with_allocation_measurement_contract(paint_allocation)
        .with_surface_paint_order(0)
}

fn hit_contract(allocation: ComponentAllocationMeasurementContract) -> ComponentHitTestContract {
    ComponentHitTestContract::allocation_bounds(ComponentHitTestOrder::front_to_back(0), allocation)
}
