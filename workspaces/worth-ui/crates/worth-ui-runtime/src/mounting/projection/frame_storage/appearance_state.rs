use super::super::UiMountedAppearanceProjectionSelection;
use super::appearance_state_membership::{
    self, UiMountedAppearanceStateEntry, UiMountedAppearanceStateMembers,
};
use crate::runtime::appearance::{UiAppearanceInvalidationBatch, UiAppearanceProjectionAttempt};
use std::rc::Rc;

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
    retirements: super::appearance_state_retirement::UiMountedAppearanceRetirements,
    epoch: Option<UiMountedAppearanceEpoch>,
    batch: Option<UiAppearanceInvalidationBatch>,
    capacity_error: Option<UiAppearanceStateCapacityExceeded>,
    reconstruction_nodes: Option<Vec<super::UiMountedAppearanceNodeInputContext>>,
    reconstruction_complete: bool,
    reconstruct_overlays: bool,
    input_refresh_nodes: Vec<super::UiMountedAppearanceNodeInputContext>,
    selection: Rc<UiMountedAppearanceProjectionSelection>,
    node_work: Vec<super::appearance_output::UiMountedAppearanceNodeWork>,
    overlay_sidecars: std::collections::BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        super::super::appearance::UiMountedAppearanceSidecar,
    >,
    active_portal_instances: std::collections::BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        std::collections::BTreeSet<worth_ui_host_contract::UiMountedInstanceIdentity>,
    >,
    overlay_work: Vec<super::appearance_output::UiMountedAppearanceOverlayWork>,
    order: super::appearance_order::UiMountedAppearanceOrderIndex,
}

impl Default for UiMountedAppearanceFrameState {
    fn default() -> Self {
        Self {
            members: UiMountedAppearanceStateMembers::default(),
            retirements: Default::default(),
            epoch: None,
            batch: None,
            capacity_error: None,
            reconstruction_nodes: None,
            reconstruction_complete: false,
            reconstruct_overlays: false,
            input_refresh_nodes: Vec::new(),
            selection: Rc::new(UiMountedAppearanceProjectionSelection::empty()),
            node_work: Vec::new(),
            overlay_sidecars: Default::default(),
            active_portal_instances: Default::default(),
            overlay_work: Vec::new(),
            order: Default::default(),
        }
    }
}

impl UiMountedAppearanceFrameState {
    pub(crate) fn raw_opacity_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedAppearanceOpacity> {
        self.members
            .retained_projection_for_instance(instance)
            .map(super::super::appearance::resolved_opacity)
    }

    pub(crate) fn requires_epoch_transition(
        &self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> bool {
        self.epoch
            .as_ref()
            .is_none_or(|epoch| epoch.session != session || epoch.generation != *generation)
    }

    pub(in crate::mounting::projection) fn matches_geometry_input(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        input: &crate::mounting::UiMountedAppearanceGeometryInput,
    ) -> (bool, usize) {
        self.members.matches_geometry_input(instance, input)
    }

    pub(in crate::mounting) fn contains_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> bool {
        self.members.contains_instance(instance)
    }

    pub(in crate::mounting) fn unresolved_epoch_instances(
        &self,
    ) -> Vec<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.members.physical_only_instances()
    }

    pub(in crate::mounting) fn after_surface_deregistration(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Self {
        let mut successor = Self::fork(
            Some(self),
            Rc::new(UiMountedAppearanceProjectionSelection::empty()),
        );
        let mut work = successor.members.forget_surface(surface);
        work.merge(successor.retirements.forget_surface(surface));
        successor.selection.record_membership_work(work);
        let work = successor.order.forget_surface(surface);
        successor.selection.record_order_work(work);
        successor.overlay_sidecars.remove(&surface);
        successor.active_portal_instances.remove(&surface);
        successor
    }

    #[cfg(test)]
    pub(super) fn isolate_measurements(&mut self) {
        self.selection = Rc::new(self.selection.with_independent_measurements());
    }

    #[cfg(test)]
    pub(crate) fn pending_attempts_for_test(&self) -> Vec<UiAppearanceProjectionAttempt> {
        self.members.pending_attempts()
    }

    #[cfg(test)]
    pub(crate) fn physical_node_receipts_for_test(
        &self,
    ) -> Vec<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        let mut receipts = self.members.physical_node_receipts();
        receipts.extend(self.retirements.receipts());
        receipts.sort();
        receipts
    }

    pub(crate) fn fork(
        predecessor: Option<&Self>,
        selection: Rc<UiMountedAppearanceProjectionSelection>,
    ) -> Self {
        Self {
            members: predecessor
                .map(|state| state.members.fork())
                .unwrap_or_default(),
            epoch: predecessor.and_then(|state| state.epoch.clone()),
            retirements: predecessor
                .map(|state| state.retirements.clone())
                .unwrap_or_default(),
            batch: None,
            capacity_error: None,
            reconstruction_nodes: None,
            reconstruction_complete: false,
            reconstruct_overlays: false,
            input_refresh_nodes: Vec::new(),
            selection,
            node_work: Vec::new(),
            overlay_sidecars: predecessor
                .map(|state| state.overlay_sidecars.clone())
                .unwrap_or_default(),
            active_portal_instances: predecessor
                .map(|state| state.active_portal_instances.clone())
                .unwrap_or_default(),
            overlay_work: Vec::new(),
            order: predecessor
                .map(|state| state.order.clone())
                .unwrap_or_default(),
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

    pub(crate) fn clear_batch(&mut self) {
        self.batch = None;
    }

    pub(crate) fn prepare_reconstruction(
        &mut self,
        nodes: Vec<super::UiMountedAppearanceNodeInputContext>,
    ) {
        self.reconstruction_nodes = Some(nodes);
        self.reconstruction_complete = true;
        self.reconstruct_overlays = true;
    }

    pub(crate) fn prepare_surface_reconstruction(
        &mut self,
        nodes: Vec<super::UiMountedAppearanceNodeInputContext>,
    ) {
        self.reconstruction_nodes = Some(nodes);
        self.reconstruction_complete = false;
        self.reconstruct_overlays = true;
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
        }
        let (retired, work) = self
            .members
            .retire_instances(retired_instances, &mut self.retirements);
        self.selection.record_membership_work(work);
        let retired_memberships = retired_memberships.saturating_add(retired);
        retired_memberships
    }

    pub(crate) fn retire_detached_on_epoch_change(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        attached_instances: &std::collections::BTreeSet<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
    ) -> usize {
        if !self.requires_epoch_transition(session, generation) {
            return 0;
        }
        let (retired, work) = self
            .members
            .retire_unattached(attached_instances, &mut self.retirements);
        self.selection.record_membership_work(work);
        retired
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

    pub(super) fn admit_order(
        &mut self,
        frame: &super::UiMountedProjectionFrame,
    ) -> Result<(), super::UiMountedAppearanceOrderDenial> {
        let (result, work) = self.order.admit(frame, &self.node_work);
        self.selection.record_order_work(work);
        result
    }

    pub(super) fn order_retained_bytes(&self) -> usize {
        self.order.retained_bytes()
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
        self.selection.cost_report(self.order.retained_bytes())
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
}

#[path = "appearance_state_lowering.rs"]
mod lowering;

#[path = "appearance_state_reconstruction.rs"]
mod reconstruction;

#[path = "appearance_state_input_refresh.rs"]
mod input_refresh;

#[path = "appearance_state_overlay.rs"]
mod overlay;
pub(super) use overlay::portal_instances;

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

#[cfg(test)]
#[path = "appearance_state_test_support.rs"]
mod test_support;
