use std::sync::Arc;
use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiFontSlant, UiIntentWiringSatisfied, UiTextFaceRequest, UiTextStyle,
    UiTextStyleInput, WorthUiApplicationBuilder,
};
use worth_ui::facade::appearance::{UiAppearanceAspect, UiAppearanceAspectContract};
use worth_ui::facade::declaration::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor,
    ComponentFocusSupport, ComponentHitTestContract, ComponentHitTestOrder, ComponentId,
    ComponentPortalChildContract, ComponentPropSchema, ComponentSemanticTextContract,
    ComponentSemanticTextFlow, ComponentStateOwnership, MosaicRegionKindId, ThemeTokenId,
};
use worth_ui_platform_pulse::product_world::{
    dashboard_containers, dashboard_elements, DashboardContent, PlatformPulseMosaicRegion,
};
mod scrolling;
mod surfaces;

pub(in crate::application) fn register_structure(
    mut builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
    fonts: &super::fonts::PulseFonts,
) -> WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied> {
    for (ordinal, element) in dashboard_elements().into_iter().enumerate() {
        let allocation = if element.id == "seed" {
            ComponentAllocationMeasurementContract::fill_viewport()
        } else {
            element.allocation()
        };
        let id = element.component_id();
        let mut descriptor = ComponentDescriptor::new(
            ComponentId::new(&id).unwrap(),
            ComponentPropSchema::named(format!("{id}.props")),
            if matches!(element.content, DashboardContent::Text { .. }) {
                ComponentChildPolicy::text_children()
            } else {
                ComponentChildPolicy::no_children()
            },
            ComponentStateOwnership::runtime_owned(),
        )
        .with_allocation_measurement_contract(allocation);
        let aspects = match element.content {
            DashboardContent::Text {
                size,
                serif,
                weight,
                alignment,
                line_height,
                wraps,
                ..
            } => {
                let style = UiTextStyle::new(UiTextStyleInput {
                    language: Arc::from("en"),
                    font_size_millipoints: u32::from(size) * 1_000,
                    letter_spacing_millipoints: 0,
                    word_spacing_millipoints: 0,
                    family_stack: if serif {
                        fonts.headings.clone()
                    } else {
                        fonts.body.clone()
                    },
                    face_request: UiTextFaceRequest::new(weight, 100_000, UiFontSlant::Upright)
                        .unwrap(),
                    features: Box::new([]),
                    variations: Box::new([]),
                })
                .unwrap();
                descriptor = descriptor.with_semantic_text(
                    ComponentSemanticTextContract::qualified_with_line_height(
                        ThemeTokenId::new(element.color_token()).unwrap(),
                        element.paint_order,
                        style,
                        u32::from(line_height) * 1_000,
                    )
                    .unwrap()
                    .with_alignment(alignment)
                    .with_flow(if wraps {
                        ComponentSemanticTextFlow::wrapping()
                    } else {
                        ComponentSemanticTextFlow::single_line_ellipsis()
                    })
                    .with_appearance_foreground()
                    .with_lifecycle_caption(false),
                );
                vec![UiAppearanceAspect::Foreground]
            }
            DashboardContent::Surface {
                radius,
                border,
                graphic,
            } => {
                descriptor = descriptor
                    .with_surface_paint_order(element.paint_order)
                    .with_surface_geometry(graphic.geometry());
                let mut aspects = vec![UiAppearanceAspect::Background];
                if radius > 0 {
                    aspects.push(UiAppearanceAspect::Radius);
                }
                if border {
                    aspects.push(UiAppearanceAspect::Border);
                }
                aspects
            }
        };
        descriptor = descriptor
            .with_appearance_aspect_contract(
                UiAppearanceAspectContract::component(aspects, []).unwrap(),
            )
            .unwrap();
        if matches!(
            element.id,
            "portal_target" | "review_target" | "confirmation_target"
        ) {
            descriptor = descriptor.with_portal_surface_from_children();
        }
        if element.interaction.is_some() {
            descriptor = descriptor
                .with_focus(ComponentFocusSupport::focusable())
                .with_hit_test(ComponentHitTestContract::allocation_bounds(
                    ComponentHitTestOrder::front_to_back(ordinal as u32),
                    allocation,
                ));
        } else if element.scroll_panel.is_some() {
            // Scrolled content hit-tests as itself, in front of the owner that
            // scrolls it, so a click resolves to the row under the cursor at
            // the displayed offset rather than to the panel.
            descriptor = descriptor.with_hit_test(ComponentHitTestContract::allocation_bounds(
                ComponentHitTestOrder::front_to_back(ordinal as u32),
                allocation,
            ));
        } else if element.id == "seed"
            || element.id == "review_surface"
            || element.id == "portal_surface"
            || element.id == "period_surface"
        {
            descriptor = descriptor.with_hit_test(ComponentHitTestContract::allocation_bounds(
                ComponentHitTestOrder::front_to_back(1_000 + ordinal as u32),
                allocation,
            ));
        }
        if element.id == "seed" {
            for region in PlatformPulseMosaicRegion::SURFACE {
                descriptor = descriptor.with_region_allocation(
                    MosaicRegionKindId::new(region.id()).unwrap(),
                    region
                        .surface_placement()
                        .expect("the surface owner places its own regions"),
                );
            }
        }
        if let Some(owner) = element.portal_owner {
            descriptor = descriptor.with_portal_child(ComponentPortalChildContract::new(
                ComponentId::new(format!("platform.pulse.component.{owner}")).unwrap(),
            ));
        }
        builder = builder.register_component(descriptor);
    }
    builder = builder.register_component(
        ComponentDescriptor::new(
            ComponentId::new("platform.pulse.component.status_band").unwrap(),
            ComponentPropSchema::named("pulse.status.props"),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        )
        .with_allocation_measurement_contract(
            ComponentAllocationMeasurementContract::viewport_region(
                PlatformPulseMosaicRegion::StatusBand
                    .surface_placement()
                    .expect("the surface owner places the status band"),
            ),
        ),
    );
    for container in dashboard_containers() {
        let id = container.component();
        let allocation = container.allocation();
        let descriptor = ComponentDescriptor::new(
            id.clone(),
            ComponentPropSchema::named(format!("{}.props", id.as_str())),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        )
        .with_allocation_measurement_contract(allocation)
        .with_layout(container.layout);
        builder = builder.register_component(match container.scroll_owner {
            Some(owner) => scrolling::scroll_owner(descriptor, owner, allocation),
            None => descriptor,
        });
    }
    surfaces::register(builder)
}
