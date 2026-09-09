use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUiApplicationBuilder,
};
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentHitTestContract, ComponentHitTestOrder,
    ComponentStaticPaintContract, ComponentStaticPaintOrder, ComponentViewportAxisPlacement,
    SurfaceKind, SurfacePlacementClass, SurfaceStateClass,
};
use worth_ui_platform_pulse::product_world::{
    PlatformPulseCompositionExtent, PlatformPulseCompositionLayout, PlatformPulseLogicalRect,
    PlatformPulseMosaicSurface, PlatformPulsePaletteRole, PlatformPulseProductComponent,
    PlatformPulseTextRole,
};

use super::product_structure_geometry::{fixed_end, fixed_start, stretch, viewport_rect};

#[path = "product_structure/descriptor.rs"]
mod descriptor;
use descriptor::{
    component, interactive_region, portal_interactive_region, portal_occupying_region,
    portal_region, portal_text, region, surface, text, text_with_token, token_id,
};

pub(in crate::application) fn register_structure(
    builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    let layout =
        PlatformPulseCompositionLayout::for_extent(PlatformPulseCompositionExtent::DEFAULT)
            .expect("the default Pulse product extent is admitted by its authored courtroom");
    let action = PlatformPulseLogicalRect::new(296, 416, 216, 48).allocation();
    let action_text = PlatformPulseLogicalRect::new(320, 430, 168, 20).allocation();
    let portal_action = PlatformPulseLogicalRect::new(528, 416, 112, 48).allocation();
    let portal_action_text = PlatformPulseLogicalRect::new(548, 430, 72, 20).allocation();
    let query_action = viewport_rect(fixed_end(48, 232), fixed_start(176, 104));
    let query_action_text = viewport_rect(fixed_end(72, 184), fixed_start(220, 48));
    let root_allocation = ComponentAllocationMeasurementContract::fill_viewport();
    builder
        .register_component(
            component(PlatformPulseProductComponent::Root)
                .with_static_paint(
                    ComponentStaticPaintContract::opaque_fill(
                        PlatformPulsePaletteRole::Canvas.token_id(),
                        ComponentStaticPaintOrder::back_to_front(0),
                    ),
                    root_allocation,
                )
                .with_hit_test(ComponentHitTestContract::allocation_bounds(
                    ComponentHitTestOrder::front_to_back(3),
                    root_allocation,
                )),
        )
        .register_component(region(
            PlatformPulseProductComponent::MastheadBorder,
            PlatformPulsePaletteRole::StructuralRule.token_id(),
            viewport_rect(
                ComponentViewportAxisPlacement::stretch_between(24, 24),
                fixed_start(79, 1),
            ),
            1,
        ))
        .register_component(text(
            PlatformPulseProductComponent::Brand,
            PlatformPulseTextRole::Masthead,
            PlatformPulsePaletteRole::PrimaryText,
            PlatformPulseLogicalRect::new(40, 42, 200, 20).allocation(),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::RuntimeBadge,
            PlatformPulseTextRole::Meta,
            PlatformPulsePaletteRole::Positive,
            viewport_rect(
                ComponentViewportAxisPlacement::fixed_from_end(40, 216)
                    .expect("Pulse badge width is nonzero"),
                fixed_start(44, 16),
            ),
            6,
        ))
        .register_component(region(
            PlatformPulseProductComponent::EvidenceBorder,
            PlatformPulsePaletteRole::StructuralRule.token_id(),
            layout.evidence_border_allocation(),
            1,
        ))
        .register_component(region(
            PlatformPulseProductComponent::EvidenceRail,
            PlatformPulsePaletteRole::RaisedSurface.token_id(),
            layout.evidence_rail_allocation(),
            2,
        ))
        .register_component(text(
            PlatformPulseProductComponent::EvidenceTitle,
            PlatformPulseTextRole::Section,
            PlatformPulsePaletteRole::SecondaryText,
            PlatformPulseLogicalRect::new(48, 136, 168, 16).allocation(),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::EvidenceBody,
            PlatformPulseTextRole::Masthead,
            PlatformPulsePaletteRole::PrimaryText,
            PlatformPulseLogicalRect::new(48, 176, 168, 64).allocation(),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::SourceSignalTitle,
            PlatformPulseTextRole::Body,
            PlatformPulsePaletteRole::SecondaryText,
            PlatformPulseLogicalRect::new(48, 256, 160, 48).allocation(),
            6,
        ))
        .register_component(region(
            PlatformPulseProductComponent::SourceSignalActive,
            token_id("theme.platform_pulse.fill"),
            viewport_rect(fixed_start(48, 144), fixed_end(112, 4)),
            4,
        ))
        .register_component(text(
            PlatformPulseProductComponent::EvidenceServiceLabel,
            PlatformPulseTextRole::Section,
            PlatformPulsePaletteRole::SecondaryText,
            PlatformPulseLogicalRect::new(48, 328, 168, 16).allocation(),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::EvidenceServiceBody,
            PlatformPulseTextRole::Body,
            PlatformPulsePaletteRole::PrimaryText,
            PlatformPulseLogicalRect::new(48, 360, 168, 64).allocation(),
            6,
        ))
        .register_component(region(
            PlatformPulseProductComponent::ServiceStageBorder,
            PlatformPulsePaletteRole::StructuralRule.token_id(),
            viewport_rect(stretch(263, 303), stretch(103, 71)),
            1,
        ))
        .register_component(region(
            PlatformPulseProductComponent::ServiceStage,
            PlatformPulsePaletteRole::ElevatedSurface.token_id(),
            viewport_rect(stretch(264, 304), stretch(104, 72)),
            2,
        ))
        .register_component(region(
            PlatformPulseProductComponent::QueryAccent,
            PlatformPulsePaletteRole::PrincipalAccent.token_id(),
            PlatformPulseLogicalRect::new(264, 104, 96, 2).allocation(),
            3,
        ))
        .register_component(text(
            PlatformPulseProductComponent::ServiceEyebrow,
            PlatformPulseTextRole::Section,
            PlatformPulsePaletteRole::SecondaryText,
            PlatformPulseLogicalRect::new(296, 136, 240, 16).allocation(),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::ServiceTitle,
            PlatformPulseTextRole::Display,
            PlatformPulsePaletteRole::PrimaryText,
            viewport_rect(stretch(296, 344), fixed_start(184, 112)),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::ServiceBody,
            PlatformPulseTextRole::Body,
            PlatformPulsePaletteRole::SecondaryText,
            viewport_rect(stretch(296, 344), fixed_start(328, 64)),
            6,
        ))
        .register_component(interactive_region(
            PlatformPulseProductComponent::ActionTarget,
            PlatformPulsePaletteRole::PrincipalAccent,
            action,
            1,
        ))
        .register_component(text(
            PlatformPulseProductComponent::ActionLabel,
            PlatformPulseTextRole::Action,
            PlatformPulsePaletteRole::PrimaryText,
            action_text,
            6,
        ))
        .register_component(interactive_region(
            PlatformPulseProductComponent::PortalTarget,
            PlatformPulsePaletteRole::RaisedSurface,
            portal_action,
            2,
        ))
        .register_component(text(
            PlatformPulseProductComponent::PortalLabel,
            PlatformPulseTextRole::Action,
            PlatformPulsePaletteRole::SecondaryText,
            portal_action_text,
            6,
        ))
        .register_component(portal_occupying_region(
            PlatformPulseProductComponent::PortalSurface,
            PlatformPulsePaletteRole::ElevatedSurface,
            PlatformPulseLogicalRect::new(0, 0, 280, 320).allocation(),
            0,
            100,
        ))
        .register_component(portal_region(
            PlatformPulseProductComponent::PortalAccent,
            PlatformPulsePaletteRole::PrincipalAccent,
            PlatformPulseLogicalRect::new(0, 0, 280, 3).allocation(),
            1,
        ))
        .register_component(portal_region(
            PlatformPulseProductComponent::PortalIconTile,
            PlatformPulsePaletteRole::RaisedSurface,
            PlatformPulseLogicalRect::new(24, 24, 36, 36).allocation(),
            2,
        ))
        .register_component(portal_text(
            PlatformPulseProductComponent::PortalIconText,
            PlatformPulseTextRole::Masthead,
            PlatformPulsePaletteRole::PrincipalAccent,
            PlatformPulseLogicalRect::new(36, 31, 18, 20).allocation(),
            6,
        ))
        .register_component(portal_text(
            PlatformPulseProductComponent::PortalTitle,
            PlatformPulseTextRole::Masthead,
            PlatformPulsePaletteRole::PrimaryText,
            PlatformPulseLogicalRect::new(76, 25, 180, 24).allocation(),
            6,
        ))
        .register_component(portal_text(
            PlatformPulseProductComponent::PortalBody,
            PlatformPulseTextRole::Body,
            PlatformPulsePaletteRole::SecondaryText,
            PlatformPulseLogicalRect::new(24, 88, 232, 64).allocation(),
            6,
        ))
        .register_component(portal_interactive_region(
            PlatformPulseProductComponent::PortalCancelTarget,
            PlatformPulsePaletteRole::RaisedSurface,
            PlatformPulseLogicalRect::new(24, 248, 104, 40).allocation(),
            21,
        ))
        .register_component(portal_text(
            PlatformPulseProductComponent::PortalCancelLabel,
            PlatformPulseTextRole::Action,
            PlatformPulsePaletteRole::SecondaryText,
            PlatformPulseLogicalRect::new(49, 258, 64, 20).allocation(),
            7,
        ))
        .register_component(portal_interactive_region(
            PlatformPulseProductComponent::PortalPrimaryTarget,
            PlatformPulsePaletteRole::PrincipalAccent,
            PlatformPulseLogicalRect::new(136, 248, 120, 40).allocation(),
            20,
        ))
        .register_component(portal_text(
            PlatformPulseProductComponent::PortalPrimaryLabel,
            PlatformPulseTextRole::Action,
            PlatformPulsePaletteRole::ActionText,
            PlatformPulseLogicalRect::new(154, 258, 88, 20).allocation(),
            7,
        ))
        .register_component(region(
            PlatformPulseProductComponent::QueryCardBorder,
            PlatformPulsePaletteRole::StructuralRule.token_id(),
            viewport_rect(fixed_end(23, 282), fixed_start(103, 202)),
            1,
        ))
        .register_component(region(
            PlatformPulseProductComponent::QueryCard,
            PlatformPulsePaletteRole::RaisedSurface.token_id(),
            viewport_rect(fixed_end(24, 280), fixed_start(104, 200)),
            2,
        ))
        .register_component(text(
            PlatformPulseProductComponent::QueryLabel,
            PlatformPulseTextRole::Section,
            PlatformPulsePaletteRole::SecondaryText,
            viewport_rect(fixed_end(48, 232), fixed_start(136, 16)),
            6,
        ))
        .register_component(interactive_region(
            PlatformPulseProductComponent::ConfirmationTarget,
            PlatformPulsePaletteRole::ElevatedSurface,
            query_action,
            0,
        ))
        .register_component(text_with_token(
            PlatformPulseProductComponent::ProjectedStatus,
            PlatformPulseTextRole::Masthead,
            token_id("theme.platform_pulse.projected_status.text"),
            query_action_text,
            6,
        ))
        .register_component(region(
            PlatformPulseProductComponent::LowerShelfDivider,
            PlatformPulsePaletteRole::StructuralRule.token_id(),
            viewport_rect(stretch(24, 24), fixed_end(48, 1)),
            3,
        ))
        .register_component(region(
            PlatformPulseProductComponent::NativeCardBorder,
            PlatformPulsePaletteRole::StructuralRule.token_id(),
            viewport_rect(fixed_end(23, 282), stretch(327, 71)),
            1,
        ))
        .register_component(region(
            PlatformPulseProductComponent::NativeCard,
            PlatformPulsePaletteRole::RaisedSurface.token_id(),
            viewport_rect(fixed_end(24, 280), stretch(328, 72)),
            2,
        ))
        .register_component(text(
            PlatformPulseProductComponent::NativeLabel,
            PlatformPulseTextRole::Section,
            PlatformPulsePaletteRole::Positive,
            viewport_rect(fixed_end(48, 232), fixed_start(360, 16)),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::NativeBody,
            PlatformPulseTextRole::Body,
            PlatformPulsePaletteRole::PrimaryText,
            viewport_rect(fixed_end(48, 232), fixed_start(392, 48)),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::QueryDenialLabel,
            PlatformPulseTextRole::Section,
            PlatformPulsePaletteRole::Caution,
            viewport_rect(fixed_end(48, 232), fixed_start(448, 16)),
            6,
        ))
        .register_component(text(
            PlatformPulseProductComponent::QueryDenialBody,
            PlatformPulseTextRole::Body,
            PlatformPulsePaletteRole::PrimaryText,
            viewport_rect(fixed_end(48, 232), fixed_start(476, 48)),
            6,
        ))
        .register_component(
            component(PlatformPulseProductComponent::StatusBand)
                .with_allocation_measurement_contract(layout.status_band_allocation()),
        )
        .register_component(text(
            PlatformPulseProductComponent::StatusText,
            PlatformPulseTextRole::Meta,
            PlatformPulsePaletteRole::PrimaryText,
            viewport_rect(stretch(40, 40), fixed_end(28, 16)),
            6,
        ))
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
