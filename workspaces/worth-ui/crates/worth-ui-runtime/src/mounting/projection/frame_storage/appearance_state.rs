use std::collections::HashMap;

use crate::runtime::appearance::{
    UiAppearanceChangeReceipt, UiAppearanceInspectionDenial, UiAppearanceInspectionRecord,
    UiAppearanceInvalidationBatch, UiAppearanceMountAffinity, UiAppearanceProjectionAttempt,
};

use super::appearance_state_membership::{
    self, UiMountedAppearanceStateEntry, UiMountedAppearanceStateMembers,
    UiMountedAppearanceStateMembership,
};

pub(crate) const APPEARANCE_STATE_CAPACITY: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceStateCapacityExceeded {
    capacity: usize,
}

impl UiAppearanceStateCapacityExceeded {
    pub(super) const fn new(capacity: usize) -> Self {
        Self { capacity }
    }

    pub const fn capacity(self) -> usize {
        self.capacity
    }
}

#[derive(Clone, Default)]
pub(crate) struct UiMountedAppearanceFrameState {
    members: UiMountedAppearanceStateMembers,
    batch: Option<UiAppearanceInvalidationBatch>,
    capacity_error: Option<UiAppearanceStateCapacityExceeded>,
    reconstruction_nodes: Option<Vec<super::UiMountedAppearanceNodeInputContext>>,
}

impl UiMountedAppearanceFrameState {
    pub(crate) fn inherit_from(&mut self, predecessor: Option<&Self>) {
        self.members
            .inherit_from(predecessor.map(|state| &state.members));
        self.batch = None;
        self.capacity_error = None;
        self.reconstruction_nodes = None;
    }

    pub(crate) fn set_batch(&mut self, batch: UiAppearanceInvalidationBatch) {
        self.batch = Some(batch);
    }

    pub(crate) fn prepare_reconstruction(
        &mut self,
        nodes: Vec<super::UiMountedAppearanceNodeInputContext>,
    ) {
        self.reconstruction_nodes = Some(nodes);
    }

    pub(crate) fn batch(&self) -> Option<&UiAppearanceInvalidationBatch> {
        self.batch.as_ref()
    }

    pub(crate) fn prune_to_current_nodes(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        nodes: &[super::UiMountedAppearanceNodeInputContext],
    ) {
        self.members
            .prune_to_current_nodes(session, generation, nodes);
    }

    pub(crate) fn reserve(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), UiAppearanceStateCapacityExceeded> {
        let result = self.members.reserve(context, APPEARANCE_STATE_CAPACITY);
        if let Err(error) = result {
            self.capacity_error = Some(error);
        }
        result
    }

    pub(crate) fn stage(
        &mut self,
        attempt: UiAppearanceProjectionAttempt,
    ) -> Result<(), UiAppearanceStateCapacityExceeded> {
        if let Err(error) = self.members.stage(attempt, APPEARANCE_STATE_CAPACITY) {
            self.capacity_error = Some(error);
            return Err(error);
        }
        Ok(())
    }

    pub(crate) const fn capacity_error(&self) -> Option<UiAppearanceStateCapacityExceeded> {
        self.capacity_error
    }

    #[cfg(test)]
    pub(super) fn membership_counts(&self) -> (usize, usize, usize) {
        self.members.membership_counts()
    }

    #[cfg(test)]
    pub(super) fn retained_entry_for_test(
        &self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Option<&UiMountedAppearanceStateEntry> {
        let key = appearance_state_membership::state_key(context);
        self.members.retained_entry(&key)
    }

    #[cfg(test)]
    pub(super) fn retain_projection_for_test(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
        projection: crate::runtime::appearance::UiAppearanceProjection,
    ) {
        let key = appearance_state_membership::state_key(context);
        self.members.insert_retained(UiMountedAppearanceStateEntry {
            key,
            context: context.clone(),
            projection,
            sidecar: Default::default(),
        });
    }

    #[cfg(test)]
    pub(super) fn replace_sidecar_for_test(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
        sidecar: super::super::appearance::UiMountedAppearanceSidecar,
    ) {
        let key = appearance_state_membership::state_key(context);
        let entry = self
            .members
            .retained_entry(&key)
            .expect("test state entry should exist before sidecar replacement");
        let mut replacement = entry.clone();
        replacement.sidecar = sidecar;
        self.members.insert_retained(replacement);
    }

    pub(crate) fn lower(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Vec<UiAppearanceInspectionRecord> {
        if let Some(nodes) = self.reconstruction_nodes.take() {
            return self.lower_reconstruction(presentation, &nodes);
        }
        let mut records = Vec::new();
        for key in self.members.take_pending_keys_for_lowering() {
            let Some(membership) = self.members.remove(&key) else {
                continue;
            };
            match membership {
                UiMountedAppearanceStateMembership::Retained(entry) => {
                    self.members.insert_retained(entry);
                }
                UiMountedAppearanceStateMembership::Reserved => {}
                UiMountedAppearanceStateMembership::Staged {
                    attempt,
                    predecessor,
                } => self.lower_staged(key, attempt, predecessor, presentation, &mut records),
            }
        }
        records
    }

    fn lower_staged(
        &mut self,
        key: appearance_state_membership::UiMountedAppearanceStateKey,
        attempt: UiAppearanceProjectionAttempt,
        mut predecessor: Option<UiMountedAppearanceStateEntry>,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        records: &mut Vec<UiAppearanceInspectionRecord>,
    ) {
        let context = attempt.context();
        let Some(projection) = attempt.projection() else {
            records.push(denial_record(
                context.clone(),
                attempt
                    .denial()
                    .unwrap_or(UiAppearanceInspectionDenial::Basis),
            ));
            restore_predecessor(&mut self.members, &mut predecessor);
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
                restore_predecessor(&mut self.members, &mut predecessor);
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
                restore_predecessor(&mut self.members, &mut predecessor);
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
                restore_predecessor(&mut self.members, &mut predecessor);
                return;
            }
        };
        self.members.insert_retained(UiMountedAppearanceStateEntry {
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
        nodes: &[super::UiMountedAppearanceNodeInputContext],
    ) -> Vec<UiAppearanceInspectionRecord> {
        let mut node_by_identity =
            HashMap::with_capacity(nodes.len().min(APPEARANCE_STATE_CAPACITY));
        node_by_identity.extend(
            nodes
                .iter()
                .map(|node| (appearance_state_membership::local_node_key(node), node)),
        );
        let mut records = Vec::new();
        let keys = self.members.retained_keys_for_reconstruction();
        for key in keys {
            let Some(entry) = self.members.retained_entry_mut(&key) else {
                continue;
            };
            let Some(node) = node_by_identity.get(key.local_node()).copied() else {
                records.push(denial_record(
                    entry.context.clone(),
                    UiAppearanceInspectionDenial::Basis,
                ));
                continue;
            };
            let input = match node.lower_retained_projection(&entry.projection, presentation) {
                Ok(input) => input,
                Err(_) => {
                    records.push(denial_record(
                        entry.context.clone(),
                        UiAppearanceInspectionDenial::MountLowering,
                    ));
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
                    continue;
                }
            };
            let Some(receipt) = UiAppearanceChangeReceipt::from_reconstruction_mount(&work) else {
                records.push(denial_record(
                    entry.context.clone(),
                    UiAppearanceInspectionDenial::MountAffinity,
                ));
                continue;
            };
            records.push(UiAppearanceInspectionRecord::Projection {
                projection: entry.projection.clone(),
                consumers_selected: 0,
                receipt,
            });
        }
        records
    }
}

fn restore_predecessor(
    members: &mut UiMountedAppearanceStateMembers,
    predecessor: &mut Option<UiMountedAppearanceStateEntry>,
) {
    if let Some(entry) = predecessor.take() {
        members.insert_retained(entry);
    }
}

pub(super) fn state_key(
    context: &crate::runtime::appearance::UiAppearanceAttemptContext,
) -> appearance_state_membership::UiMountedAppearanceStateKey {
    appearance_state_membership::state_key(context)
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

#[cfg(test)]
#[path = "appearance_state_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "appearance_state_reconstruction_tests.rs"]
mod reconstruction_tests;
