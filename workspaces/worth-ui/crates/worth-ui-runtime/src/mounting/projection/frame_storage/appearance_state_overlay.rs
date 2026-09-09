use std::collections::BTreeSet;

use crate::mounting::projection::appearance::{
    UiMountedAppearanceGeometryScope, UiMountedAppearanceSurfaceOverlayInput,
};

use super::super::appearance_output::{
    UiMountedAppearanceOutputDenial, UiMountedAppearanceOverlayWork,
};
use super::super::appearance_state_membership::local_node_key;
use super::super::UiMountedProjectionFrame;
use super::UiMountedAppearanceFrameState;

pub(in crate::mounting::projection::frame_storage) fn portal_instances(
    overlays: &[UiMountedAppearanceSurfaceOverlayInput],
) -> BTreeSet<worth_ui_host_contract::UiMountedInstanceIdentity> {
    overlays
        .iter()
        .flat_map(|overlay| overlay.portal_instances.iter().copied())
        .collect()
}

impl UiMountedAppearanceFrameState {
    pub(in crate::mounting::projection::frame_storage) fn stage_portal_ownership_changes(
        &mut self,
        frame: &UiMountedProjectionFrame,
        bindings: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
        overlays: &[UiMountedAppearanceSurfaceOverlayInput],
    ) -> Result<(), UiMountedAppearanceOutputDenial> {
        for binding in bindings {
            let surface = binding.semantic_surface();
            let current: BTreeSet<_> = overlays
                .iter()
                .find(|overlay| overlay.semantic_surface == surface)
                .map(|overlay| overlay.portal_instances.iter().copied().collect())
                .unwrap_or_default();
            let previous = self
                .active_portal_instances
                .get(&surface)
                .cloned()
                .unwrap_or_default();
            let (current_children, _) = frame
                .semantic
                .portal_children_for_owners(&current.iter().copied().collect::<Vec<_>>());
            let (previous_children, _) = frame
                .semantic
                .portal_children_for_owners(&previous.iter().copied().collect::<Vec<_>>());
            let current_children = current_children.into_iter().collect::<BTreeSet<_>>();
            let previous_children = previous_children.into_iter().collect::<BTreeSet<_>>();
            let mut refresh = previous
                .symmetric_difference(&current)
                .copied()
                .filter(|portal| {
                    self.selection
                        .selected_instances()
                        .binary_search(portal)
                        .is_err()
                })
                .collect::<BTreeSet<_>>();
            refresh.extend(
                previous_children
                    .symmetric_difference(&current_children)
                    .copied(),
            );
            refresh.retain(|instance| {
                self.selection
                    .selected_instances()
                    .binary_search(instance)
                    .is_err()
            });
            for portal in refresh {
                let context = frame
                    .appearance_node_input(portal)
                    .map_err(|_| UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)?;
                if context.semantic_surface != surface {
                    return Err(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
                }
                self.stage_input_refresh(&context);
            }
            if current.is_empty() {
                self.active_portal_instances.remove(&surface);
            } else {
                self.active_portal_instances.insert(surface, current);
            }
        }
        Ok(())
    }

    pub(in crate::mounting::projection::frame_storage) fn lower_overlays(
        &mut self,
        frame: &UiMountedProjectionFrame,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        geometry: &UiMountedAppearanceGeometryScope,
        overlays: &[UiMountedAppearanceSurfaceOverlayInput],
    ) -> Result<(), UiMountedAppearanceOutputDenial> {
        let active_surfaces = overlays
            .iter()
            .map(|overlay| overlay.semantic_surface)
            .collect::<BTreeSet<_>>();
        for overlay in overlays {
            if !geometry.includes_surface(overlay.semantic_surface) {
                return Err(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
            }
            let portals = overlay
                .portal_instances
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            let mut nodes = Vec::with_capacity(portals.len());
            for portal in &portals {
                if !frame
                    .portal_has_appearance_attachment(*portal, overlay.semantic_surface)
                    .map_err(|_| UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)?
                {
                    continue;
                }
                let context = frame
                    .appearance_node_input(*portal)
                    .map_err(|_| UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)?;
                let entry = self
                    .members
                    .retained_entry_for_local_node(&local_node_key(&context))
                    .ok_or(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)?;
                let mut input = context
                    .lower_retained_projection(
                        &entry.projection,
                        presentation,
                        geometry.outline_fringe(context.semantic_surface),
                    )
                    .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
                input
                    .retain_portal_surface(context.mounted_instance)
                    .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
                nodes.extend(input.into_nodes());
            }
            let mut input = overlay.lowering_input(frame.frame_identity(), presentation, nodes);
            input
                .compose_accepted_motion(geometry)
                .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
            let sidecar = self
                .overlay_sidecars
                .entry(overlay.semantic_surface)
                .or_default();
            let work = if self.reconstruct_overlays && sidecar.has_current() {
                sidecar.reconstruct(input)
            } else {
                sidecar.mount(input)
            }
            .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
            self.overlay_work.push(UiMountedAppearanceOverlayWork {
                surface: overlay.semantic_surface,
                work,
            });
        }
        let retired = self
            .overlay_sidecars
            .keys()
            .copied()
            .filter(|surface| {
                geometry.includes_surface(*surface) && !active_surfaces.contains(surface)
            })
            .collect::<Vec<_>>();
        for surface in retired {
            let sidecar = self
                .overlay_sidecars
                .remove(&surface)
                .expect("retired overlay surface was retained");
            let work = sidecar
                .removal_work(frame.frame_identity(), presentation)
                .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
            self.overlay_work
                .push(UiMountedAppearanceOverlayWork { surface, work });
        }
        self.reconstruct_overlays = false;
        Ok(())
    }
}
