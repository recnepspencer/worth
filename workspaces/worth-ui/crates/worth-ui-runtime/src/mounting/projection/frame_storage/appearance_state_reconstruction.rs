use super::super::appearance_output::UiMountedAppearanceOutputDenial;
use super::super::appearance_state_membership::{self, UiMountedAppearanceStateMembership};
use super::lowering::{denial_record, geometry_output_denial, AppearanceLoweringPosture};
use super::UiMountedAppearanceFrameState;
use crate::mounting::projection::appearance::UiMountedAppearanceGeometryScope;
use crate::runtime::appearance::{
    UiAppearanceChangeReceipt, UiAppearanceInspectionDenial, UiAppearanceInspectionRecord,
};
use std::collections::HashMap;

impl UiMountedAppearanceFrameState {
    pub(super) fn lower_reconstruction(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        geometry: &UiMountedAppearanceGeometryScope,
        portal_instances: &std::collections::BTreeSet<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
        nodes: &[super::super::UiMountedAppearanceNodeInputContext],
    ) -> Result<Vec<UiAppearanceInspectionRecord>, UiMountedAppearanceOutputDenial> {
        let mut node_by_identity =
            HashMap::with_capacity(nodes.len().min(super::APPEARANCE_STATE_CAPACITY));
        node_by_identity.extend(
            nodes
                .iter()
                .map(|node| (appearance_state_membership::local_node_key(node), node)),
        );
        let (keys, work) = self.members.retained_keys_for_reconstruction();
        self.selection.record_membership_work(work);
        // Capture retained entries before pending attempts become retained, so
        // freshly resolved nodes are reconstructed exactly once below.
        let keys = keys?;
        let mut records = self.lower_pending(
            presentation,
            geometry,
            portal_instances,
            AppearanceLoweringPosture::Reconstruction,
        )?;
        for key in keys {
            let (membership, work) = self.members.remove(key.local_node());
            self.selection.record_membership_work(work);
            let Some(UiMountedAppearanceStateMembership::Retained(mut entry)) = membership else {
                continue;
            };
            let Some(node) = node_by_identity.get(key.local_node()).copied() else {
                records.push(denial_record(
                    entry.context.clone(),
                    UiAppearanceInspectionDenial::Basis,
                ));
                self.restore_entry(entry);
                continue;
            };
            let mut input = match node.lower_retained_projection(
                &entry.projection,
                presentation,
                geometry.outline_fringe(node.semantic_surface),
            ) {
                Ok(input) => input,
                Err(denial) => {
                    records.push(denial_record(
                        entry.context.clone(),
                        UiAppearanceInspectionDenial::MountLowering,
                    ));
                    self.restore_entry(entry);
                    if let Some(denial) = geometry_output_denial(denial) {
                        return Err(denial);
                    }
                    continue;
                }
            };
            if input.compose_accepted_motion(geometry).is_err() {
                records.push(denial_record(
                    entry.context.clone(),
                    UiAppearanceInspectionDenial::MountLowering,
                ));
                self.restore_entry(entry);
                continue;
            }
            input.retain_node_owned_families(portal_instances);
            let predecessor_receipt = entry.sidecar.current_node_receipt();
            let work = match entry.sidecar.reconstruct(input) {
                Ok(work) => work,
                Err(_) => {
                    records.push(denial_record(
                        entry.context.clone(),
                        UiAppearanceInspectionDenial::MountLowering,
                    ));
                    self.restore_entry(entry);
                    continue;
                }
            };
            let Some(receipt) = UiAppearanceChangeReceipt::from_reconstruction_mount(&work) else {
                records.push(denial_record(
                    entry.context.clone(),
                    UiAppearanceInspectionDenial::MountAffinity,
                ));
                self.restore_entry(entry);
                continue;
            };
            self.node_work.push(
                super::super::appearance_output::UiMountedAppearanceNodeWork {
                    predecessor: predecessor_receipt,
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
