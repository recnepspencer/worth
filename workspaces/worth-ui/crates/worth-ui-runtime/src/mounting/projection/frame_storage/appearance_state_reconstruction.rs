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
        complete: bool,
    ) -> Result<Vec<UiAppearanceInspectionRecord>, UiMountedAppearanceOutputDenial> {
        let mut node_by_identity =
            HashMap::with_capacity(nodes.len().min(super::APPEARANCE_STATE_CAPACITY));
        node_by_identity.extend(
            nodes
                .iter()
                .map(|node| (appearance_state_membership::local_node_key(node), node)),
        );
        let node_by_instance = nodes
            .iter()
            .map(|node| (node.mounted_instance, node))
            .collect::<HashMap<_, _>>();
        let (mut keys, work) = self.members.keys_for_reconstruction();
        self.selection.record_membership_work(work);
        if !complete {
            keys.retain(|key| {
                node_by_instance
                    .get(&key.mounted_instance)
                    .is_some_and(|node| node.semantic_surface == key.surface)
            });
        }
        // Capture retained entries before pending attempts become retained, so
        // freshly resolved nodes are reconstructed exactly once below.
        let mut records = self.lower_pending(
            presentation,
            geometry,
            portal_instances,
            AppearanceLoweringPosture::Reconstruction,
        )?;
        // A denied semantic refresh restores its accepted physical predecessor.
        // Cold reconstruction must still reissue that paint in this same pass.
        let (restored_physical, work) = self.members.physical_only_keys_for_reconstruction();
        self.selection.record_membership_work(work);
        keys.extend(restored_physical.into_iter().filter(|key| {
            complete
                || node_by_instance
                    .get(&key.mounted_instance)
                    .is_some_and(|node| node.semantic_surface == key.surface)
        }));
        keys.sort();
        keys.dedup();
        for key in keys {
            let (membership, work) = self.members.remove(&key);
            self.selection.record_membership_work(work);
            if let Some(UiMountedAppearanceStateMembership::PhysicalOnly(mut physical)) = membership
            {
                let node = node_by_instance
                    .get(&key.mounted_instance)
                    .copied()
                    .filter(|node| {
                        node.semantic_surface == key.surface && node.incarnation == key.incarnation
                    });
                let Some(node) = node else {
                    self.restore_physical_predecessor(
                        super::super::appearance_state_predecessor::UiMountedAppearanceStatePredecessor::PhysicalOnly(physical),
                    );
                    return Err(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
                };
                let predecessor = physical.sidecar.current_node_receipt();
                let work = match physical.sidecar.reconstruct_physical(
                    node.issuer(),
                    node.node_receipt,
                    presentation,
                ) {
                    Ok(work) => work,
                    Err(_) => {
                        self.restore_physical_predecessor(
                            super::super::appearance_state_predecessor::UiMountedAppearanceStatePredecessor::PhysicalOnly(physical),
                        );
                        return Err(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
                    }
                };
                physical.key = appearance_state_membership::local_node_key(node);
                self.node_work.push(
                    super::super::appearance_output::UiMountedAppearanceNodeWork {
                        predecessor,
                        successor: Some(node.node_receipt),
                        work,
                    },
                );
                self.restore_physical_predecessor(
                    super::super::appearance_state_predecessor::UiMountedAppearanceStatePredecessor::PhysicalOnly(
                        physical,
                    ),
                );
                continue;
            }
            let Some(UiMountedAppearanceStateMembership::Retained(mut entry)) = membership else {
                continue;
            };
            let Some(node) = node_by_identity.get(&key).copied() else {
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
