use std::collections::HashMap;

use crate::runtime::appearance::{
    UiAppearanceChangeReceipt, UiAppearanceInspectionDenial, UiAppearanceInspectionRecord,
    UiAppearanceMountAffinity, UiAppearanceProjectionAttempt,
};

use super::super::appearance_state_membership::{self, UiMountedAppearanceStateMembership};
use super::{UiMountedAppearanceFrameState, UiMountedAppearanceStateEntry};

impl UiMountedAppearanceFrameState {
    pub(crate) fn lower(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Vec<UiAppearanceInspectionRecord> {
        if let Some(nodes) = self.reconstruction_nodes.take() {
            return self.lower_reconstruction(presentation, &nodes);
        }
        let mut records = Vec::new();
        for key in self.members.take_pending_keys_for_lowering() {
            let (membership, work) = self.members.remove(&key);
            self.selection.record_membership_work(work);
            let Some(membership) = membership else {
                continue;
            };
            match membership {
                UiMountedAppearanceStateMembership::Retained(entry) => {
                    self.restore_entry(entry);
                }
                UiMountedAppearanceStateMembership::Reserved => {}
                UiMountedAppearanceStateMembership::Staged {
                    attempt,
                    predecessor,
                } => self.lower_staged(attempt, predecessor, presentation, &mut records),
            }
        }
        records
    }

    fn lower_staged(
        &mut self,
        attempt: UiAppearanceProjectionAttempt,
        mut predecessor: Option<UiMountedAppearanceStateEntry>,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        records: &mut Vec<UiAppearanceInspectionRecord>,
    ) {
        let context = attempt.context();
        let key = appearance_state_membership::state_key(context);
        let Some(projection) = attempt.projection() else {
            records.push(denial_record(
                context.clone(),
                attempt
                    .denial()
                    .unwrap_or(UiAppearanceInspectionDenial::Basis),
            ));
            self.restore_predecessor(&mut predecessor);
            return;
        };
        let mut sidecar = predecessor
            .as_ref()
            .map_or_else(Default::default, |entry| entry.sidecar.clone());
        let input = match context.lower_resolved(projection, presentation) {
            Ok(input) => input,
            Err(_) => {
                records.push(denial_record(
                    context.clone(),
                    UiAppearanceInspectionDenial::MountLowering,
                ));
                self.restore_predecessor(&mut predecessor);
                return;
            }
        };
        let affinity = UiAppearanceMountAffinity {
            session: context.target().session(),
            generation: context.generation().clone(),
            frame: context.frame(),
            surface: context.semantic_surface(),
            graph_node: context.graph_node(),
            mounted_instance: context.mounted_instance(),
            incarnation: context.incarnation(),
            node_receipt: context.node_receipt(),
            issuer: context.issuer(),
            presentation,
        };
        let work = match sidecar.mount(input) {
            Ok(work) => work,
            Err(_) => {
                records.push(denial_record(
                    context.clone(),
                    UiAppearanceInspectionDenial::MountLowering,
                ));
                self.restore_predecessor(&mut predecessor);
                return;
            }
        };
        let predecessor_projection = predecessor.as_ref().map(|entry| &entry.projection);
        let receipt = match UiAppearanceChangeReceipt::from_resolved_mount(
            predecessor_projection,
            projection,
            &work,
            affinity,
        ) {
            Ok(receipt) => receipt,
            Err(_) => {
                records.push(denial_record(
                    context.clone(),
                    UiAppearanceInspectionDenial::MountAffinity,
                ));
                self.restore_predecessor(&mut predecessor);
                return;
            }
        };
        self.restore_entry(UiMountedAppearanceStateEntry {
            key,
            context: context.clone(),
            projection: projection.clone(),
            sidecar,
        });
        records.push(UiAppearanceInspectionRecord::Projection {
            projection: projection.clone(),
            consumers_selected: context.consumers_selected(),
            receipt,
        });
    }

    fn lower_reconstruction(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        nodes: &[super::super::UiMountedAppearanceNodeInputContext],
    ) -> Vec<UiAppearanceInspectionRecord> {
        let mut node_by_identity =
            HashMap::with_capacity(nodes.len().min(super::APPEARANCE_STATE_CAPACITY));
        node_by_identity.extend(
            nodes
                .iter()
                .map(|node| (appearance_state_membership::local_node_key(node), node)),
        );
        let (keys, work) = self.members.retained_keys_for_reconstruction();
        self.selection.record_membership_work(work);
        let mut records = Vec::new();
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
            let input = match node.lower_retained_projection(&entry.projection, presentation) {
                Ok(input) => input,
                Err(_) => {
                    records.push(denial_record(
                        entry.context.clone(),
                        UiAppearanceInspectionDenial::MountLowering,
                    ));
                    self.restore_entry(entry);
                    continue;
                }
            };
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
            records.push(UiAppearanceInspectionRecord::Projection {
                projection: entry.projection.clone(),
                consumers_selected: 0,
                receipt,
            });
            self.restore_entry(entry);
        }
        records
    }

    fn restore_predecessor(&mut self, predecessor: &mut Option<UiMountedAppearanceStateEntry>) {
        if let Some(entry) = predecessor.take() {
            self.restore_entry(entry);
        }
    }

    fn restore_entry(&mut self, entry: UiMountedAppearanceStateEntry) {
        let (result, work) = self.members.insert_retained(entry);
        self.selection.record_membership_work(work);
        result.expect("appearance membership indexes retain one exact local identity");
    }
}

fn denial_record(
    context: crate::runtime::appearance::UiAppearanceAttemptContext,
    denial: UiAppearanceInspectionDenial,
) -> UiAppearanceInspectionRecord {
    UiAppearanceInspectionRecord::Denial {
        context,
        denial,
        receipt: UiAppearanceChangeReceipt::for_denial(),
    }
}
