use worth_ui::facade::app::UiTextOriginalRange;
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor,
    ComponentFocusSupport, ComponentHitTestContract, ComponentHitTestOrder, ComponentId,
    ComponentPortalChildContract, ComponentPropSchema, ComponentSemanticTextSpanContract,
    ComponentStateOwnership, ComponentStaticPaintContract, ComponentStaticPaintOrder,
    SurfaceDescriptor, SurfaceId, SurfaceKind, SurfacePlacementClass, SurfaceStateClass,
    ThemeTokenId,
};
use worth_ui_platform_pulse::product_world::{
    PlatformPulseMosaicSurface, PlatformPulsePaletteRole, PlatformPulseProductComponent,
    PlatformPulseTextRole,
};

pub(super) fn component(component: PlatformPulseProductComponent) -> ComponentDescriptor {
    let id = component.id();
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid Pulse component id"),
        ComponentPropSchema::named(format!("{id}.props")),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

pub(super) fn region(
    identity: PlatformPulseProductComponent,
    fill: ThemeTokenId,
    allocation: ComponentAllocationMeasurementContract,
    order: u32,
) -> ComponentDescriptor {
    component(identity)
        .with_static_paint(
            ComponentStaticPaintContract::opaque_fill(
                fill,
                ComponentStaticPaintOrder::back_to_front(order),
            ),
            allocation,
        )
        .with_surface_paint_order(order)
}

pub(super) fn text(
    identity: PlatformPulseProductComponent,
    role: PlatformPulseTextRole,
    foreground: PlatformPulsePaletteRole,
    allocation: ComponentAllocationMeasurementContract,
    order: u32,
) -> ComponentDescriptor {
    text_with_token(identity, role, foreground.token_id(), allocation, order)
}

pub(super) fn text_with_token(
    identity: PlatformPulseProductComponent,
    role: PlatformPulseTextRole,
    foreground: ThemeTokenId,
    allocation: ComponentAllocationMeasurementContract,
    order: u32,
) -> ComponentDescriptor {
    let id = identity.id();
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid Pulse text component id"),
        ComponentPropSchema::named(format!("{id}.props")),
        ComponentChildPolicy::text_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(allocation)
    .with_semantic_text(
        role.style()
            .semantic_text_contract(foreground, order)
            .expect("Pulse text contract is qualified"),
    )
}

pub(super) fn text_with_appearance_foreground(
    identity: PlatformPulseProductComponent,
    role: PlatformPulseTextRole,
    foreground: PlatformPulsePaletteRole,
    source: &str,
    allocation: ComponentAllocationMeasurementContract,
    order: u32,
) -> ComponentDescriptor {
    let foreground = foreground.token_id();
    let source_end = u32::try_from(source.len()).expect("Pulse copy fits text range mechanics");
    let range = UiTextOriginalRange::new(0, source_end).expect("Pulse copy is nonempty");
    let span = ComponentSemanticTextSpanContract::new(
        range,
        foreground.clone(),
        role.style().qualified_style(),
    )
    .expect("Pulse appearance text span is nonempty")
    .with_appearance_foreground();
    let contract = role
        .style()
        .semantic_text_contract(foreground, order)
        .expect("Pulse text contract is qualified")
        .with_scalar_spans([span])
        .expect("Pulse appearance text span covers the source contiguously");
    let id = identity.id();
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid Pulse text component id"),
        ComponentPropSchema::named(format!("{id}.props")),
        ComponentChildPolicy::text_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_allocation_measurement_contract(allocation)
    .with_semantic_text(contract)
}

pub(super) fn interactive_region(
    identity: PlatformPulseProductComponent,
    background: PlatformPulsePaletteRole,
    control_allocation: ComponentAllocationMeasurementContract,
    hit_order: u32,
) -> ComponentDescriptor {
    component(identity)
        .with_static_paint(
            ComponentStaticPaintContract::opaque_fill(
                background.token_id(),
                ComponentStaticPaintOrder::back_to_front(4),
            ),
            control_allocation,
        )
        .with_hit_test(ComponentHitTestContract::allocation_bounds(
            ComponentHitTestOrder::front_to_back(hit_order),
            control_allocation,
        ))
        .with_semantic_text(
            PlatformPulseTextRole::Action
                .style()
                .semantic_text_contract(token_id("theme.platform_pulse.projected_status.text"), 5)
                .expect("Pulse interactive posture text is qualified"),
        )
        .with_focus(ComponentFocusSupport::focusable())
}

fn portal_child(descriptor: ComponentDescriptor) -> ComponentDescriptor {
    descriptor.with_portal_child(ComponentPortalChildContract::new(
        ComponentId::new(PlatformPulseProductComponent::PortalTarget.id())
            .expect("valid Pulse Portal owner component id"),
    ))
}

pub(super) fn portal_region(
    identity: PlatformPulseProductComponent,
    fill: PlatformPulsePaletteRole,
    allocation: ComponentAllocationMeasurementContract,
    order: u32,
) -> ComponentDescriptor {
    portal_child(region(identity, fill.token_id(), allocation, order))
}

pub(super) fn portal_occupying_region(
    identity: PlatformPulseProductComponent,
    fill: PlatformPulsePaletteRole,
    allocation: ComponentAllocationMeasurementContract,
    paint_order: u32,
    hit_order: u32,
) -> ComponentDescriptor {
    portal_child(
        region(identity, fill.token_id(), allocation, paint_order).with_hit_test(
            ComponentHitTestContract::allocation_bounds(
                ComponentHitTestOrder::front_to_back(hit_order),
                allocation,
            ),
        ),
    )
}

pub(super) fn portal_text(
    identity: PlatformPulseProductComponent,
    role: PlatformPulseTextRole,
    foreground: PlatformPulsePaletteRole,
    allocation: ComponentAllocationMeasurementContract,
    order: u32,
) -> ComponentDescriptor {
    portal_child(text(identity, role, foreground, allocation, order))
}

pub(super) fn portal_interactive_region(
    identity: PlatformPulseProductComponent,
    background: PlatformPulsePaletteRole,
    allocation: ComponentAllocationMeasurementContract,
    hit_order: u32,
) -> ComponentDescriptor {
    portal_child(interactive_region(
        identity, background, allocation, hit_order,
    ))
}

pub(super) fn surface(
    identity: PlatformPulseMosaicSurface,
    kind: SurfaceKind,
    root: PlatformPulseProductComponent,
    placement: SurfacePlacementClass,
    state: SurfaceStateClass,
) -> SurfaceDescriptor {
    SurfaceDescriptor::new(
        SurfaceId::new(identity.id()).expect("valid Pulse Mosaic surface identity"),
        kind,
        ComponentId::new(root.id()).expect("valid Pulse surface root component"),
        placement,
        state,
    )
}

pub(super) fn token_id(text: &str) -> ThemeTokenId {
    ThemeTokenId::new(text).expect("valid Pulse theme token id")
}
