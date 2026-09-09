use super::{UiMountedAppearanceOrderDenial as Denial, Update};
use crate::mounting::spatial_index::UiMountedSpatialWork;
use worth_ui_host_contract::{
    UiAppearanceClip, UiAppearanceVisualBounds, UiMountedAppearanceMechanic as Mechanic,
    UiMountedInstanceIdentity, UiSurfaceBindingGeneration,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct PartitionKey {
    pub(super) binding: UiSurfaceBindingGeneration,
    pub(super) space: u8,
    pub(super) portal: Option<UiMountedInstanceIdentity>,
    pub(super) rank: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Row {
    pub(super) key: PartitionKey,
    pub(super) family: usize,
    pub(super) bounds: [f64; 4],
}

pub(super) fn derive(
    frame: &super::super::UiMountedProjectionFrame,
    node: &super::super::appearance_output::UiMountedAppearanceNodeWork,
    work: &mut UiMountedSpatialWork,
) -> Result<Update, Denial> {
    let surface = node.work.successor().semantic_surface();
    let instance = node
        .successor
        .or(node.predecessor)
        .ok_or(Denial::MountedGeometryUnavailable)?
        .mounted_instance();
    let mut rows = Vec::new();
    let mechanics = node.work.successor().mechanics();
    if mechanics.iter().any(|m| {
        matches!(
            m,
            Mechanic::Surface(_) | Mechanic::Outline(_) | Mechanic::PortalSurface(_)
        )
    }) {
        let (mounted, probes) = frame.semantic.nodes.get_with_probes(&instance);
        work.map_key_probes += probes;
        let mounted = mounted.ok_or(Denial::MountedGeometryUnavailable)?;
        let geometry = mounted.completed_appearance_geometry();
        let space = match geometry.allocation {
            worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, .. }
            | worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
                bounds,
                ..
            } => bounds.coordinate_space() as u8,
            _ => return Err(Denial::MountedGeometryUnavailable),
        };
        let (mounted_surface, probes) = frame.semantic.surface_for_with_probes(surface);
        work.map_key_probes += probes;
        let binding = mounted_surface
            .ok_or(Denial::MountedGeometryUnavailable)?
            .binding;
        for mechanic in mechanics {
            let (rank, family, bounds, clip, portal) = match mechanic {
                Mechanic::Surface(m) => (
                    m.surface_paint_order(),
                    0,
                    m.visual_bounds(),
                    m.clip(),
                    geometry.portal_group,
                ),
                Mechanic::Outline(m) => (
                    m.surface_paint_order(),
                    1,
                    m.visual_bounds(),
                    m.clip(),
                    geometry.portal_group,
                ),
                Mechanic::PortalSurface(m) => (
                    m.surface().surface_paint_order(),
                    0,
                    m.surface().visual_bounds(),
                    m.surface().clip(),
                    Some(m.portal_instance()),
                ),
                _ => continue,
            };
            if let Some(bounds) = clipped(bounds, clip) {
                rows.push(Row {
                    key: PartitionKey {
                        binding,
                        space,
                        portal,
                        rank,
                    },
                    family,
                    bounds,
                });
            }
        }
    }
    Ok(Update {
        surface,
        instance,
        rows,
    })
}

fn clipped(bounds: UiAppearanceVisualBounds, clip: UiAppearanceClip) -> Option<[f64; 4]> {
    let x0 = i64::from(bounds.x()).max(i64::from(clip.x()));
    let y0 = i64::from(bounds.y()).max(i64::from(clip.y()));
    let x1 = (i64::from(bounds.x()) + i64::from(bounds.width()))
        .min(i64::from(clip.x()) + i64::from(clip.width()));
    let y1 = (i64::from(bounds.y()) + i64::from(bounds.height()))
        .min(i64::from(clip.y()) + i64::from(clip.height()));
    // The sum of an i32 origin and u32 extent is exactly representable in f64.
    (x0 < x1 && y0 < y1).then_some([x0 as f64, y0 as f64, x1 as f64, y1 as f64])
}
