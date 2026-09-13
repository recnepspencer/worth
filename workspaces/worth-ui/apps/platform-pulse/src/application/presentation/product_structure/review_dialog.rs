use super::*;
use worth_ui::facade::declaration::{
    ComponentDescriptor, ComponentId, ComponentPortalChildContract,
};

pub(super) fn register(
    mut builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    use PlatformPulseProductComponent as C;
    builder = builder
        .register_component(portal_interactive_region(
            C::ReviewTarget,
            rect(24, 160, 232, 40),
            22,
        ))
        .register_component(portal_text(
            C::ReviewLabel,
            PlatformPulseTextRole::Action,
            PlatformPulsePaletteRole::SecondaryText,
            rect(48, 170, 184, 20),
            7,
        ));
    // Each dialog owns its own surface and child coordinates. The Portal
    // relation determines stack order; these ranks order content within a dialog.
    let surface = component(C::ReviewSurface)
        .with_allocation_measurement_contract(rect(0, 0, 280, 320))
        .with_hit_test(ComponentHitTestContract::allocation_bounds(
            ComponentHitTestOrder::front_to_back(101),
            rect(0, 0, 280, 320),
        ));
    builder = builder.register_component(child(surface));
    for (component, role, color, bounds) in [
        (
            C::ReviewTitle,
            PlatformPulseTextRole::Masthead,
            PlatformPulsePaletteRole::PrimaryText,
            rect(24, 24, 232, 28),
        ),
        (
            C::ReviewBody,
            PlatformPulseTextRole::Body,
            PlatformPulsePaletteRole::SecondaryText,
            rect(24, 76, 232, 80),
        ),
        (
            C::ReviewCancelLabel,
            PlatformPulseTextRole::Action,
            PlatformPulsePaletteRole::SecondaryText,
            rect(48, 226, 64, 20),
        ),
        (
            C::ReviewPrimaryLabel,
            PlatformPulseTextRole::Action,
            PlatformPulsePaletteRole::ActionText,
            rect(160, 226, 88, 20),
        ),
    ] {
        builder = builder.register_component(child(text(component, role, color, bounds, 7)));
    }
    for (component, bounds, hit_order) in [
        (C::ReviewCancelTarget, rect(24, 216, 104, 40), 31),
        (C::ReviewPrimaryTarget, rect(144, 216, 112, 40), 30),
    ] {
        builder =
            builder.register_component(child(interactive_region(component, bounds, hit_order)));
    }
    builder
}

fn child(component: ComponentDescriptor) -> ComponentDescriptor {
    component.with_portal_child(ComponentPortalChildContract::new(
        ComponentId::new(PlatformPulseProductComponent::ReviewTarget.id()).unwrap(),
    ))
}

fn rect(x: u32, y: u32, width: u32, height: u32) -> ComponentAllocationMeasurementContract {
    PlatformPulseLogicalRect::new(x, y, width, height).allocation()
}
