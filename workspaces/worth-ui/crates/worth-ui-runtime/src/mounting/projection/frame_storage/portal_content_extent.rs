use super::{UiMountedProjectionDenial, UiMountedSemanticProjection};
use crate::mounting::projection::UiMountedAppearanceClip;
use worth_ui_host_contract::{
    UiMountedAllocationProjection, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedCoordinateSpace, UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
    UiSurfaceGeometry,
};

impl UiMountedSemanticProjection {
    /// The extent the owner's Portal content lays out over in this projection:
    /// the content roots this projection holds, where it lays each out, as far
    /// as `clip`, the coverage regions inside the content leave each root,
    /// lets the Portal show it. A root that shows nothing, such as a layout
    /// container that only divides its allocation among the roots it lays
    /// out, adds nothing; those roots count themselves. An open measures the
    /// published projection; a successor frame measures the one it prepares,
    /// so a root it adds or replaces counts the frame it arrives. Cost follows
    /// the selected owner's indexed children, including currently suppressed
    /// content. No unrelated surface is scanned.
    pub(in crate::mounting) fn portal_content_extent(
        &self,
        owner: UiMountedInstanceIdentity,
        clip: impl Fn(UiSemanticSurfaceIdentity, UiMountedInstanceIdentity) -> UiMountedAppearanceClip,
    ) -> Result<Option<crate::runtime::portal::UiPortalContentBounds>, UiMountedProjectionDenial>
    {
        let (children, _) = self.portal_children_for_owners(&[owner]);
        if children.is_empty() {
            return Ok(None);
        }
        let bounds = |allocation| match allocation {
            UiMountedAllocationProjection::Known { bounds, .. }
            | UiMountedAllocationProjection::PortalAnchorObservation { bounds, .. } => Ok(bounds),
            UiMountedAllocationProjection::Omitted(_) => {
                Err(UiMountedProjectionDenial::PortalOverlayOwnerMissing)
            }
        };
        let anchor = bounds(
            self.node(owner)
                .ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?
                .occurrence_allocation
                .into_layout_space(),
        )?;
        // Content is one union measured in anchor-relative space. An authored
        // overlay routinely begins before the control that opens it: a
        // viewport-inset panel is wider than its anchor, and a menu opens
        // upward. Carrying the union's near edge admits those worlds, where
        // assuming the anchor origin bounded every child denied them outright.
        let mut near = [f32::MAX, f32::MAX];
        let mut far = [f32::MIN, f32::MIN];
        let mut layout = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        let mut has_shadow = false;
        for instance in children {
            let child = self
                .node(instance)
                .ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?;
            if !child.has_appearance_attachment && child.semantic_text.is_none() {
                continue;
            }
            let inset = match child.surface_geometry {
                UiSurfaceGeometry::SoftShadow(shadow) => {
                    has_shadow = true;
                    shadow.sigma().points_f32() * 3.0
                }
                UiSurfaceGeometry::RoundedRectangle | UiSurfaceGeometry::Vector(_) => 0.0,
            };
            let clip = clip(child.receipt.semantic_surface(), instance);
            let child = bounds(child.occurrence_allocation.into_layout_space())?;
            if child.coordinate_space() != anchor.coordinate_space() {
                return Err(UiMountedProjectionDenial::CoordinateBasisMismatch);
            }
            if child.width() <= inset * 2.0 || child.height() <= inset * 2.0 {
                return Err(UiMountedProjectionDenial::NegativeExtent);
            }
            // A shadow lays out as the body that casts it, inset from its blur.
            let caster = rebased(
                local_bounds([
                    child.x() + inset,
                    child.y() + inset,
                    child.width() - inset * 2.0,
                    child.height() - inset * 2.0,
                ])?,
                child,
            );
            // A region can leave a shadow's blur showing where it hides the
            // body casting it, so what paints and what lays out count apart.
            if let Some(child) = shown(child, clip) {
                near[0] = near[0].min(child.x() - anchor.x());
                near[1] = near[1].min(child.y() - anchor.y());
                far[0] = far[0].max(child.x() + child.width() - anchor.x());
                far[1] = far[1].max(child.y() + child.height() - anchor.y());
            }
            if let Some(caster) = shown(caster, clip) {
                layout[0] = layout[0].min(caster.x() - anchor.x());
                layout[1] = layout[1].min(caster.y() - anchor.y());
                layout[2] = layout[2].max(caster.x() + caster.width() - anchor.x());
                layout[3] = layout[3].max(caster.y() + caster.height() - anchor.y());
            }
        }
        if near[0] > far[0] {
            // Nothing of the content can appear where it is laid out.
            return Ok(None);
        }
        let extent = [far[0] - near[0], far[1] - near[1]];
        if extent
            .iter()
            .any(|value| *value <= 0.0 || value.ceil() > f32::from(u16::MAX))
        {
            return Err(UiMountedProjectionDenial::NegativeExtent);
        }
        let paint = local_bounds([
            near[0].floor(),
            near[1].floor(),
            extent[0].ceil(),
            extent[1].ceil(),
        ])?;
        // Content whose regions show only the blur of its shadows lays out as
        // far as it paints.
        let layout = if has_shadow && layout[0] <= layout[2] {
            local_bounds([
                layout[0].floor(),
                layout[1].floor(),
                layout[2].ceil() - layout[0].floor(),
                layout[3].ceil() - layout[1].floor(),
            ])?
        } else {
            paint
        };
        Ok(Some(crate::runtime::portal::UiPortalContentBounds {
            layout: crate::mounting::presentation::UiPublishedRect::from_committed_box(layout),
            paint: crate::mounting::presentation::UiPublishedRect::from_committed_box(paint),
        }))
    }
}

/// What of `child` its Portal can show: the part the regions inside its
/// content leave it. The Portal presents those clips moved by the same step
/// as the child, so rows a list inside the Portal scrolls out of view add
/// nothing to its extent. `None` when nothing of it can appear. A clip still
/// unresolved leaves the child whole: the extent may then exceed what the
/// Portal comes to show, never fall short of it.
fn shown(
    child: UiMountedCanonicalBox,
    clip: UiMountedAppearanceClip,
) -> Option<UiMountedCanonicalBox> {
    match clip {
        UiMountedAppearanceClip::Suppressed => None,
        UiMountedAppearanceClip::Ancestor(clip) => {
            child.intersection(rebased(clip.canonical_box()?, child))
        }
        UiMountedAppearanceClip::Unclipped | UiMountedAppearanceClip::Unresolved(_) => Some(child),
    }
}

/// `bounds` in the coordinate space of `space`. It relabels without moving:
/// the child's inset caster and the clips its regions leave it are measured
/// in the same coordinates as the child, carried under their own label.
fn rebased(bounds: UiMountedCanonicalBox, space: UiMountedCanonicalBox) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: bounds.x(),
        y: bounds.y(),
        width: bounds.width(),
        height: bounds.height(),
        coordinate_space: space.coordinate_space(),
    })
    .expect("a canonical box stays canonical in another space")
}

fn local_bounds(
    [x, y, width, height]: [f32; 4],
) -> Result<UiMountedCanonicalBox, UiMountedProjectionDenial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::GraphNodeLocal,
    })
    .map_err(|denial| match denial {
        worth_ui_host_contract::UiMountedGeometryDenial::NonFinite => {
            UiMountedProjectionDenial::NonFiniteGeometry
        }
        worth_ui_host_contract::UiMountedGeometryDenial::NegativeExtent => {
            UiMountedProjectionDenial::NegativeExtent
        }
    })
}
