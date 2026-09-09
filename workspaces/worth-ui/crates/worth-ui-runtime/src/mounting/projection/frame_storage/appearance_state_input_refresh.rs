use super::super::appearance_state_membership::{
    local_node_key, UiMountedAppearanceStateKey, UiMountedAppearanceStateMembership,
};
use super::super::{UiMountedAppearanceNodeInputContext, UiMountedAppearanceOutputDenial};
use super::UiMountedAppearanceFrameState;
use crate::mounting::projection::appearance::UiMountedAppearanceGeometryScope;
use crate::runtime::appearance::{UiAppearanceChangeReceipt, UiAppearanceInspectionRecord};

impl UiMountedAppearanceFrameState {
    pub(in crate::mounting::projection::frame_storage) fn stage_input_refresh(
        &mut self,
        node: &UiMountedAppearanceNodeInputContext,
    ) -> bool {
        let Some(epoch) = &self.epoch else {
            return false;
        };
        let key = UiMountedAppearanceStateKey {
            session: epoch.session,
            generation: epoch.generation.clone(),
            local_node: local_node_key(node),
        };
        let (current, work) = self.members.has_current_retained(&key);
        self.selection.record_membership_work(work);
        if current {
            self.input_refresh_nodes.push(node.clone());
        }
        current
    }

    pub(super) fn lower_input_refreshes(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        geometry: &UiMountedAppearanceGeometryScope,
        portal_instances: &std::collections::BTreeSet<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
    ) -> Result<Vec<UiAppearanceInspectionRecord>, UiMountedAppearanceOutputDenial> {
        let mut records = Vec::new();
        for node in std::mem::take(&mut self.input_refresh_nodes) {
            let (membership, cost) = self.members.remove(&local_node_key(&node));
            self.selection.record_membership_work(cost);
            let Some(UiMountedAppearanceStateMembership::Retained(mut entry)) = membership else {
                return Err(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
            };
            let mut input = node
                .lower_retained_projection(
                    &entry.projection,
                    presentation,
                    geometry.outline_fringe(node.semantic_surface),
                )
                .map_err(|denial| {
                    super::lowering::geometry_output_denial(denial)
                        .unwrap_or(UiMountedAppearanceOutputDenial::NodeLowering)
                })?;
            input
                .compose_accepted_motion(geometry)
                .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
            input.retain_node_owned_families(portal_instances);
            let predecessor = entry.sidecar.current_node_receipt();
            let work = entry
                .sidecar
                .mount(input)
                .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
            let receipt = UiAppearanceChangeReceipt::from_retained_paint_mount(&work);
            self.node_work.push(
                super::super::appearance_output::UiMountedAppearanceNodeWork {
                    predecessor,
                    successor: Some(node.node_receipt),
                    work,
                },
            );
            records.push(UiAppearanceInspectionRecord::Projection {
                projection: entry.projection.clone(),
                consumers_selected: 0,
                receipt,
            });
            self.restore_entry(entry);
        }
        Ok(records)
    }
}
