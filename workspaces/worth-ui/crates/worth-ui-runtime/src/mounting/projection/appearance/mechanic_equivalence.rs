use worth_ui_host_contract::UiMountedAppearanceMechanic;

pub(super) fn same_physical_output(
    predecessor: &UiMountedAppearanceMechanic,
    successor: &UiMountedAppearanceMechanic,
) -> bool {
    match (predecessor, successor) {
        (
            UiMountedAppearanceMechanic::Surface(left),
            UiMountedAppearanceMechanic::Surface(right),
        ) => {
            left.bounds() == right.bounds()
                && left.clip() == right.clip()
                && left.surface_paint_order() == right.surface_paint_order()
                && left.visual_bounds() == right.visual_bounds()
                && left.radii() == right.radii()
                && left.border_edges() == right.border_edges()
                && left.border_omissions() == right.border_omissions()
                && left.paint() == right.paint()
                && left.opacity() == right.opacity()
        }
        (
            UiMountedAppearanceMechanic::PortalSurface(left),
            UiMountedAppearanceMechanic::PortalSurface(right),
        ) => {
            left.portal_instance() == right.portal_instance()
                && same_physical_output(
                    &UiMountedAppearanceMechanic::Surface(left.surface().clone()),
                    &UiMountedAppearanceMechanic::Surface(right.surface().clone()),
                )
        }
        (
            UiMountedAppearanceMechanic::Outline(left),
            UiMountedAppearanceMechanic::Outline(right),
        ) => {
            left.clip() == right.clip()
                && left.surface_paint_order() == right.surface_paint_order()
                && left.allocation() == right.allocation()
                && left.visual_bounds() == right.visual_bounds()
                && left.color() == right.color()
                && left.width() == right.width()
                && left.offset() == right.offset()
                && left.anti_alias_fringe() == right.anti_alias_fringe()
                && left.radii() == right.radii()
                && left.opacity() == right.opacity()
        }
        (
            UiMountedAppearanceMechanic::TextForeground(left),
            UiMountedAppearanceMechanic::TextForeground(right),
        ) => {
            left.command() == right.command()
                && left.paint_span() == right.paint_span()
                && left.foreground() == right.foreground()
                && left.opacity() == right.opacity()
        }
        (
            UiMountedAppearanceMechanic::Pointer(left),
            UiMountedAppearanceMechanic::Pointer(right),
        ) => left == right,
        (
            UiMountedAppearanceMechanic::Backdrop(left),
            UiMountedAppearanceMechanic::Backdrop(right),
        ) => {
            left.identity() == right.identity()
                && left.semantic_surface() == right.semantic_surface()
                && left.placement().ordinal() == right.placement().ordinal()
                && left.extent() == right.extent()
                && left.clip() == right.clip()
                && left.background() == right.background()
                && left.opacity() == right.opacity()
        }
        _ => false,
    }
}
