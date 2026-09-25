//! Places each open Portal again from where this frame lays out its anchor.
//!
//! A Portal's anchor is its owner occurrence, and its children present
//! relative to that owner. When layout moves the owner or the surface resizes,
//! the Portal is placed again by the same Portal arithmetic that opened it,
//! from the owner's allocation and the surface viewport this frame carries. A
//! Portal whose anchor or viewport this frame cannot place keeps the
//! placement it has.
use crate::mounting::presentation::UiPublishedRect;
use worth_ui_host_contract::UiMountedAllocationProjection;

pub(in crate::mounting) fn succeed_portal_placements(
    state: &super::UiMountedIdentityState,
    geometry: &super::UiMountedOccurrenceGeometryState,
    overlays: std::rc::Rc<[super::UiMountedPortalOverlayProjectionInput]>,
) -> std::rc::Rc<[super::UiMountedPortalOverlayProjectionInput]> {
    if overlays.is_empty() {
        return overlays;
    }
    // Parents first: a nested owner presents inside its parent's placement.
    let mut order = (0..overlays.len()).collect::<Vec<_>>();
    order.sort_by_key(|&index| overlays[index].placement().layer().depth());
    let mut succeeded = overlays.to_vec();
    // Each placed Portal's owner layout and where it now paints, which is how
    // far it moves the content laid out inside it.
    let mut placed = Vec::with_capacity(overlays.len());
    for index in order {
        let input = overlays[index];
        let Some(owner) = owner_allocation(state, geometry, input.owner(), input.surface()) else {
            continue;
        };
        let layer = input.placement().layer();
        let anchor = match layer.parent() {
            None => Some(owner),
            Some(parent) => placed.iter().find(|(portal, _, _)| *portal == parent).map(
                |&(_, parent_owner, parent_paint)| nested_anchor(owner, parent_owner, parent_paint),
            ),
        };
        let successor = anchor
            .zip(portal_viewport(state, geometry, input.surface()))
            .and_then(|(anchor, viewport)| input.placement().succeeded(anchor, viewport).ok());
        if let Some(placement) = successor {
            succeeded[index] = input.with_placement(placement);
        }
        let paint = succeeded[index].placement().paint_bounds().rect();
        placed.push((layer.portal(), owner, paint));
    }
    succeeded.into()
}

/// A nested Portal's owner is laid out inside its parent's content, which
/// presents where the parent paints, so its anchor moves by the same distance.
#[expect(
    clippy::disallowed_methods,
    reason = "a nested Portal's anchor is its owner's layout moved by its parent Portal's placement"
)]
fn nested_anchor(
    owner: UiPublishedRect,
    parent_owner: UiPublishedRect,
    parent_paint: UiPublishedRect,
) -> UiPublishedRect {
    let [paint_x, paint_y, _, _] = parent_paint.components();
    let [owner_x, owner_y, _, _] = parent_owner.components();
    owner.translated([paint_x - owner_x, paint_y - owner_y])
}

/// The owner's allocation in this frame, as a box in the client viewport.
fn owner_allocation(
    state: &super::UiMountedIdentityState,
    geometry: &super::UiMountedOccurrenceGeometryState,
    owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> Option<UiPublishedRect> {
    let view = state.projection_instance(owner)?;
    let (binding, _) = state.projection_surface(surface)?;
    match geometry.projection(&view).ok()?? {
        UiMountedAllocationProjection::Known { bounds, .. } => {
            viewport_rect(bounds, binding.profile().coordinate_posture())
        }
        UiMountedAllocationProjection::PortalAnchorObservation { .. }
        | UiMountedAllocationProjection::Omitted(_) => None,
    }
}

/// The surface's current layout viewport, when that layout belongs to the
/// binding the surface presents on. Portals fit inside it and Backdrops
/// cover it.
pub(in crate::mounting) fn portal_viewport(
    state: &super::UiMountedIdentityState,
    geometry: &super::UiMountedOccurrenceGeometryState,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> Option<UiPublishedRect> {
    let (binding, _) = state.projection_surface(surface)?;
    let (layout_binding, _, viewport) = geometry.surface_viewport(surface)?;
    (layout_binding == binding.binding_generation()).then_some(())?;
    viewport_rect(viewport, binding.profile().coordinate_posture())
}

/// A committed surface box published where the client viewport sees it.
fn viewport_rect(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    posture: super::UiSurfaceBindingCoordinatePosture,
) -> Option<UiPublishedRect> {
    super::projection::viewport_bounds(bounds, posture)
        .ok()
        .map(UiPublishedRect::from_committed_box)
}
