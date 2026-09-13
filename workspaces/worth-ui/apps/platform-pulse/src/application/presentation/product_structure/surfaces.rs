use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUiApplicationBuilder,
};
use worth_ui::facade::declaration::{SurfaceKind, SurfacePlacementClass, SurfaceStateClass};
use worth_ui_platform_pulse::product_world::{
    PlatformPulseMosaicSurface, PlatformPulseProductComponent,
};

fn surface(
    identity: PlatformPulseMosaicSurface,
    kind: SurfaceKind,
    root: PlatformPulseProductComponent,
    placement: SurfacePlacementClass,
    state: SurfaceStateClass,
) -> worth_ui::facade::declaration::SurfaceDescriptor {
    use worth_ui::facade::declaration::{ComponentId, SurfaceDescriptor, SurfaceId};
    SurfaceDescriptor::new(
        SurfaceId::new(identity.id()).unwrap(),
        kind,
        ComponentId::new(root.id()).unwrap(),
        placement,
        state,
    )
}

pub(super) fn register(
    builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    builder
        .register_surface(surface(
            PlatformPulseMosaicSurface::Main,
            SurfaceKind::primary_content(),
            PlatformPulseProductComponent::Root,
            SurfacePlacementClass::primary_region(),
            SurfaceStateClass::ephemeral(),
        ))
        .register_surface(surface(
            PlatformPulseMosaicSurface::Evidence,
            SurfaceKind::auxiliary_content(),
            PlatformPulseProductComponent::EvidenceRail,
            SurfacePlacementClass::auxiliary_region(),
            SurfaceStateClass::restorable(),
        ))
        .register_surface(surface(
            PlatformPulseMosaicSurface::Service,
            SurfaceKind::primary_content(),
            PlatformPulseProductComponent::ServiceStage,
            SurfacePlacementClass::primary_region(),
            SurfaceStateClass::restorable(),
        ))
        .register_surface(surface(
            PlatformPulseMosaicSurface::Status,
            SurfaceKind::status_content(),
            PlatformPulseProductComponent::StatusBand,
            SurfacePlacementClass::status_region(),
            SurfaceStateClass::ephemeral(),
        ))
}
