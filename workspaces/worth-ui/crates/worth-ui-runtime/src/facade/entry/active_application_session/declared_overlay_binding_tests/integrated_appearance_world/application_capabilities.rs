use super::{authored, overlay, palette, test_support};
use crate::capability::*;

pub(super) fn builder(
    seam: bool,
    multi_region_owner: bool,
    role: Option<&worth_ui_dsl::UiAppearanceRoleDeclaration>,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    let mut builder = test_support::authored_overlay_builder_with_component(authored::component(0))
        .with_focus_policy_defaults(crate::declaration::UiFocusPolicy::workbench())
        .with_motion_policy_defaults(crate::declaration::UiMotionPolicy::system_respecting())
        .with_scroll_policy_defaults(crate::declaration::UiScrollPolicy::nested_region())
        .register_surface(SurfaceDescriptor::new(SurfaceId::new("workspace.surface.secondary").unwrap(),
            SurfaceKind::overlay_content(), ComponentId::new(authored::COMPONENTS[0]).unwrap(),
            SurfacePlacementClass::overlay_layer(), SurfaceStateClass::restorable()))
        .register_surface(SurfaceDescriptor::new(SurfaceId::new("aaa.surface").unwrap(),
            SurfaceKind::overlay_content(), ComponentId::new(authored::COMPONENTS[0]).unwrap(),
            SurfacePlacementClass::overlay_layer(), SurfaceStateClass::restorable()))
        .register_theme_token(crate::runtime::tests::appearance_component_session_test_support::appearance_theme_token(ThemeTokenId::new(palette::TEXT_TOKEN).unwrap()));
    if seam {
        let primary = MosaicRegionKindId::new("workspace.region.primary").unwrap();
        let secondary = MosaicRegionKindId::new("workspace.region.secondary").unwrap();
        let edge = MosaicSharedEdge::new(primary.clone(), secondary.clone()).unwrap();
        let contract = MosaicSeamPaintContract::admit(
            [primary.clone(), secondary.clone()],
            [edge.clone()],
            [MosaicSeamPaintOwner::new(edge, primary.clone()).unwrap()],
            [
                MosaicExteriorCorner::new(primary.clone(), MosaicExteriorCornerPosture::TopLeft),
                MosaicExteriorCorner::new(primary, MosaicExteriorCornerPosture::BottomLeft),
                MosaicExteriorCorner::new(secondary.clone(), MosaicExteriorCornerPosture::TopRight),
                MosaicExteriorCorner::new(secondary, MosaicExteriorCornerPosture::BottomRight),
            ],
        )
        .unwrap();
        let secondary_region = if multi_region_owner {
            authored::secondary_region_allowing_escape()
        } else {
            authored::secondary_region()
        };
        builder = builder
            .register_mosaic_region_kind(secondary_region)
            .register_mosaic_seam_paint_contract(contract)
            .unwrap();
    }
    for index in 0..authored::COMPONENTS.len() {
        if index != 0 {
            builder = builder.register_component(authored::component(index));
        }
        builder = builder
            .register_appearance_role(if index == 0 {
                role.cloned().unwrap_or_else(|| authored::role(index))
            } else {
                authored::role(index)
            })
            .unwrap();
    }
    builder
        .register_mosaic_sizing_contract(super::super::replacement_geometry::alternate_sizing())
        .register_appearance_role(super::super::replacement::successor_role())
        .unwrap()
        .register_appearance_role(overlay::backdrop_role())
        .unwrap()
        .register_appearance_theme_bundle(palette::theme())
        .unwrap()
}
