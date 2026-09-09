use super::super::appearance_publication_support::SOURCE;
use super::palette;
use crate::capability::*;
use worth_ui_dsl::*;

pub(super) const COMPONENTS: [&str; 4] = [
    "workspace.component.overlay",
    "workspace.component.peer",
    "workspace.component.child",
    "workspace.component.content",
];
pub(super) const PAINT_ORDERS: [u32; 4] = [170, 110, 230, 240];
const HIT_ORDERS: [u32; 4] = [2, 0, 1, 3];

pub(super) fn source() -> String {
    source_with_seam(false)
}

pub(super) fn seam_source() -> String {
    source_with_seam(true)
}

pub(super) fn multi_region_seam_source() -> String {
    seam_source().replace(
        "    interaction activate routes workspace.intent.open opens portal overlay.menu;",
        "    region workspace.region.secondary {\n        sizing workspace.sizing.mosaic_support;\n    }\n    interaction activate routes workspace.intent.open opens portal overlay.menu;",
    )
}

fn source_with_seam(seam: bool) -> String {
    let mut source = SOURCE.replace(
        "appearance role overlay.content applies_to workspace.component.overlay {\n    background use token(overlay.content.background)\n}",
        &role_source(COMPONENTS[0], "overlay.content"),
    ).replace("dismiss escape", "dismiss escape anchor_gone")
        .replace("scope surface_singleton", "scope per_portal_instance overlay.menu")
        .replace("motion none", "motion follow portal overlay.menu presentation");
    if seam {
        source = source.replace(
            "extent surface_viewport workspace.surface.overlay",
            "extent presented_mosaic_region workspace.surface.overlay workspace.region.secondary",
        );
    }
    for (index, component) in COMPONENTS.iter().enumerate().skip(1) {
        let role = format!("overlay.content{index}");
        let region = if seam && index == 1 {
            "workspace.region.secondary"
        } else {
            "workspace.region.primary"
        };
        source.push_str(&role_source(component, &role));
        source.push_str(&format!("component {component} {{ appearance {{ role {role} }} region {region} {{ sizing workspace.sizing.mosaic_support; }} }}\n"));
    }
    source.push_str(
        r#"
portal overlay.child {
    surface workspace.surface.overlay
    anchor workspace.anchor
    layer modal
    dismiss anchor_gone
    focus first_enabled
    motion system_popover
}

backdrop overlay.after {
    scope per_portal_instance overlay.menu
    extent presented_mosaic_region workspace.surface.overlay workspace.region.primary
    presence while portal overlay.menu presented
    motion follow portal overlay.menu presentation
    place immediately_after portal overlay.menu
    appearance { role overlay.scrim }
}
backdrop overlay.ambient {
    scope surface_singleton
    extent presented_mosaic_region workspace.surface.secondary workspace.region.primary
    presence always
    motion none
    place above_surface_content
    appearance { role overlay.scrim }
}
"#,
    );
    source
}

pub(super) fn secondary_region() -> MosaicRegionKindDescriptor {
    secondary_region_with_clipping(MosaicClippingPosture::clip_to_region())
}

pub(super) fn secondary_region_allowing_escape() -> MosaicRegionKindDescriptor {
    secondary_region_with_clipping(MosaicClippingPosture::allow_overlay_escape())
}

fn secondary_region_with_clipping(clipping: MosaicClippingPosture) -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(
        MosaicRegionKindId::new("workspace.region.secondary").unwrap(),
        MosaicRegionRole::auxiliary(),
    )
    .with_sizing_behavior(MosaicSizingBehavior::fills_available_space())
    .with_scroll_ownership(MosaicScrollOwnership::region_owned())
    .with_focus_scope(MosaicFocusScopeKind::active_surface_scope())
    .with_child_rule(MosaicChildRule::accepts_surfaces())
    .with_allowed_surface_class(SurfacePlacementClass::primary_region())
    .with_persistence(MosaicRegionPersistence::restorable())
    .with_clipping(clipping)
    .with_hit_test(MosaicHitTestPosture::participates())
}

fn role_source(component: &str, role: &str) -> String {
    format!(
        r#"
appearance role {role} applies_to {component} {{
    background over [hover] {{
        cell outside when hover = outside use token(overlay.content.background)
        cell inside when hover = hovered use token(overlay.content.hovered)
    }}
    foreground use token(overlay.content.foreground)
    border use token(overlay.content.border)
    radius use token(overlay.content.radius)
    outline use token(overlay.content.outline)
    opacity use token(overlay.content.opacity)
}}
"#
    )
}

pub(super) fn role(index: usize) -> UiAppearanceRoleDeclaration {
    let name = if index == 0 {
        "overlay.content".to_owned()
    } else {
        format!("overlay.content{index}")
    };
    let mut authoring = UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(name).unwrap())
        .applies_to_component(UiDslComponentReference::new(COMPONENTS[index]).unwrap());
    let hover = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Hover,
    )])
    .with_cell(
        UiAppearanceCell::named("outside")
            .when([UiAppearanceAxisPredicate::exact(
                UiAppearanceAxisClass::HoverOutside,
            )])
            .uses_slot(
                UiThemeSlotIdentity::new("overlay.content.background").unwrap(),
                UiThemeValueKind::Color,
            ),
    )
    .with_cell(
        UiAppearanceCell::named("inside")
            .when([UiAppearanceAxisPredicate::exact(
                UiAppearanceAxisClass::Hovered,
            )])
            .uses_slot(
                UiThemeSlotIdentity::new("overlay.content.hovered").unwrap(),
                UiThemeValueKind::Color,
            ),
    );
    authoring = authoring
        .cover(UiAppearanceAspect::Background, hover)
        .unwrap();
    for (aspect, slot, kind) in [
        (
            UiAppearanceAspect::Foreground,
            "foreground",
            UiThemeValueKind::Color,
        ),
        (
            UiAppearanceAspect::Border,
            "border",
            UiThemeValueKind::SolidStroke,
        ),
        (
            UiAppearanceAspect::Radius,
            "radius",
            UiThemeValueKind::CornerRadii,
        ),
        (
            UiAppearanceAspect::Outline,
            "outline",
            UiThemeValueKind::SolidOutline,
        ),
        (
            UiAppearanceAspect::Opacity,
            "opacity",
            UiThemeValueKind::Opacity,
        ),
    ] {
        authoring = authoring
            .cover(
                aspect,
                UiAppearancePartitionAuthoring::new([]).with_cell(
                    UiAppearanceCell::when([]).uses_slot(
                        UiThemeSlotIdentity::new(format!("overlay.content.{slot}")).unwrap(),
                        kind,
                    ),
                ),
            )
            .unwrap();
    }
    authoring.build().unwrap()
}

pub(super) fn component(index: usize) -> ComponentDescriptor {
    let [x, y, width, height] = super::geometry::BOXES[if index == 3 { 4 } else { index }];
    let allocation =
        ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
            ComponentViewportAxisPlacement::fixed_from_start(x as u16, width as u16).unwrap(),
            ComponentViewportAxisPlacement::fixed_from_start(y as u16, height as u16).unwrap(),
        ));
    let component = ComponentDescriptor::new(
        ComponentId::new(COMPONENTS[index]).unwrap(),
        ComponentPropSchema::named("integrated.overlay.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_hit_test(ComponentHitTestContract::allocation_bounds(
        ComponentHitTestOrder::front_to_back(HIT_ORDERS[index]),
        allocation,
    ))
    .with_surface_paint_order(PAINT_ORDERS[index])
    .with_focus(ComponentFocusSupport::focusable())
    .with_semantic_text(text_contract())
    .with_appearance_aspect_contract(role(index).aspect_contract().clone())
    .unwrap();
    if index == 3 {
        component.with_portal_child(ComponentPortalChildContract::new(
            ComponentId::new(COMPONENTS[0]).unwrap(),
        ))
    } else {
        component
    }
}

pub(super) fn text_contract() -> ComponentSemanticTextContract {
    let token = ThemeTokenId::new(palette::TEXT_TOKEN).unwrap();
    let constraints = worth_ui_text::UiTextParagraphConstraints::new(
        worth_ui_text::UiTextParagraphConstraintsInput {
            language: std::sync::Arc::from("und"),
            base_direction: worth_ui_text::UiTextBaseDirection::Auto,
            wrap: worth_ui_text::UiTextWrap::UnicodeWord,
            alignment: worth_ui_text::UiTextAlignment::Start,
            overflow: worth_ui_text::UiTextOverflow::Clip,
            font_size_millipoints: 14_000,
            width_millipoints: 160_000,
            line_height_millipoints: 18_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 56_000,
            maximum_lines: 1,
        },
    )
    .unwrap();
    let span = |start, end| {
        ComponentSemanticTextSpanContract::new(
            worth_ui_host_contract::UiTextOriginalRange::new(start, end).unwrap(),
            token.clone(),
            worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
        )
        .unwrap()
    };
    ComponentSemanticTextContract::spanned(
        token.clone(),
        7,
        [span(0, 1).with_appearance_foreground(), span(1, 2)],
    )
    .unwrap()
}
