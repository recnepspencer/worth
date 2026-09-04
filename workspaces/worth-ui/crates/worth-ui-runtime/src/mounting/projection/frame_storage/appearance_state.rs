use std::rc::Rc;

use crate::runtime::appearance::{UiAppearanceInvalidationBatch, UiAppearanceProjectionAttempt};

use super::super::UiMountedAppearanceProjectionSelection;
use super::appearance_state_membership::{
    self, UiMountedAppearanceStateEntry, UiMountedAppearanceStateMembers,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceStateMutationDenial {
    Capacity(UiAppearanceStateCapacityExceeded),
    LocalIdentityMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiMountedAppearanceEpoch {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
}

#[derive(Clone)]
pub(crate) struct UiMountedAppearanceFrameState {
    members: UiMountedAppearanceStateMembers,
    epoch: Option<UiMountedAppearanceEpoch>,
    batch: Option<UiAppearanceInvalidationBatch>,
    capacity_error: Option<UiAppearanceStateCapacityExceeded>,
    reconstruction_nodes: Option<Vec<super::UiMountedAppearanceNodeInputContext>>,
    selection: Rc<UiMountedAppearanceProjectionSelection>,
}

impl Default for UiMountedAppearanceFrameState {
    fn default() -> Self {
        Self {
            members: UiMountedAppearanceStateMembers::default(),
            epoch: None,
            batch: None,
            capacity_error: None,
            reconstruction_nodes: None,
            selection: Rc::new(UiMountedAppearanceProjectionSelection::empty()),
        }
    }
}

impl UiMountedAppearanceFrameState {
    pub(crate) fn fork(
        predecessor: Option<&Self>,
        selection: Rc<UiMountedAppearanceProjectionSelection>,
    ) -> Self {
        Self {
            members: predecessor
                .map(|state| state.members.fork())
                .unwrap_or_default(),
            epoch: predecessor.and_then(|state| state.epoch.clone()),
            batch: None,
            capacity_error: None,
            reconstruction_nodes: None,
            selection,
        }
    }

    #[cfg(test)]
    pub(super) fn inherit_from(&mut self, predecessor: Option<&Self>) {
        let selection = predecessor
            .map(|state| Rc::clone(&state.selection))
            .unwrap_or_else(|| Rc::new(UiMountedAppearanceProjectionSelection::empty()));
        *self = Self::fork(predecessor, selection);
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

    pub(crate) fn begin_epoch(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        retired_instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> usize {
        let epoch = UiMountedAppearanceEpoch {
            session,
            generation: generation.clone(),
        };
        let mut retired_memberships = 0;
        if self.epoch.as_ref() != Some(&epoch) {
            let (retired, work) = self.members.clear_for_epoch();
            retired_memberships = retired;
            self.selection.record_membership_work(work);
            self.epoch = Some(epoch);
            self.capacity_error = None;
            self.reconstruction_nodes = None;
        }
        let (retired, work) = self.members.retire_instances(retired_instances);
        self.selection.record_membership_work(work);
        let retired_memberships = retired_memberships.saturating_add(retired);
        retired_memberships
    }

    pub(crate) fn reserve(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), UiMountedAppearanceStateMutationDenial> {
        if !self.context_belongs_to_epoch(context) {
            return Err(UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch);
        }
        let (result, work) = self.members.reserve(context, APPEARANCE_STATE_CAPACITY);
        self.selection.record_membership_work(work);
        if let Err(UiMountedAppearanceStateMutationDenial::Capacity(error)) = result {
            self.capacity_error = Some(error);
            return Err(UiMountedAppearanceStateMutationDenial::Capacity(error));
        }
        result
    }

    pub(crate) fn stage(
        &mut self,
        attempt: UiAppearanceProjectionAttempt,
    ) -> Result<(), UiMountedAppearanceStateMutationDenial> {
        if !self.context_belongs_to_epoch(attempt.context()) {
            return Err(UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch);
        }
        let (result, work) = self.members.stage(attempt, APPEARANCE_STATE_CAPACITY);
        self.selection.record_membership_work(work);
        if let Err(UiMountedAppearanceStateMutationDenial::Capacity(error)) = result {
            self.capacity_error = Some(error);
            return Err(UiMountedAppearanceStateMutationDenial::Capacity(error));
        }
        result
    }

    fn context_belongs_to_epoch(
        &self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> bool {
        self.epoch.as_ref().is_some_and(|epoch| {
            epoch.session == context.target().session() && epoch.generation == *context.generation()
        })
    }

    pub(crate) const fn capacity_error(&self) -> Option<UiAppearanceStateCapacityExceeded> {
        self.capacity_error
    }

    pub(crate) fn validate_selection(
        &self,
        batch: &UiAppearanceInvalidationBatch,
    ) -> Result<(), super::super::UiMountedProjectionDenial> {
        self.selection
            .matches_batch(batch)
            .then_some(())
            .ok_or(super::super::UiMountedProjectionDenial::AppearanceSelectionBatchMismatch)
    }

    pub(crate) fn selection_cost_report(
        &self,
    ) -> super::super::UiMountedAppearanceSelectionCostReport {
        self.selection.cost_report()
    }

    pub(crate) fn selected_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.selection.selected_instances()
    }

    pub(crate) fn retired_instances(&self) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.selection.retired_instances()
    }

    pub(crate) fn record_materialized_contexts(&self, count: usize) {
        self.selection.record_materialized_contexts(count);
    }

    pub(crate) fn record_lifecycle_memberships_retired(&self, count: usize) {
        self.selection.record_lifecycle_memberships_retired(count);
    }

    #[cfg(test)]
    pub(super) fn membership_counts(&self) -> (usize, usize, usize) {
        self.members.membership_counts()
    }

    #[cfg(test)]
    pub(super) fn membership_roots_shared_with(&self, other: &Self) -> bool {
        self.members.roots_shared_with(&other.members)
    }

    #[cfg(test)]
    pub(super) fn membership_work(&self) -> (usize, usize, usize) {
        let report = self.selection.cost_report();
        (
            report.membership_key_probes(),
            report.membership_copied_avl_nodes(),
            report.membership_traversed_entries(),
        )
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
        let (result, work) = self.members.insert_retained(UiMountedAppearanceStateEntry {
            key,
            context: context.clone(),
            projection,
            sidecar: Default::default(),
        });
        self.selection.record_membership_work(work);
        result.expect("test retention uses one exact mounted identity");
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
        let (result, work) = self.members.insert_retained(replacement);
        self.selection.record_membership_work(work);
        result.expect("test sidecar replacement uses one exact mounted identity");
    }
}

#[path = "appearance_state_lowering.rs"]
mod lowering;

pub(super) fn state_key(
    context: &crate::runtime::appearance::UiAppearanceAttemptContext,
) -> appearance_state_membership::UiMountedAppearanceStateKey {
    appearance_state_membership::state_key(context)
}

#[cfg(test)]
#[path = "appearance_state_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "appearance_state_reconstruction_tests.rs"]
mod reconstruction_tests;
