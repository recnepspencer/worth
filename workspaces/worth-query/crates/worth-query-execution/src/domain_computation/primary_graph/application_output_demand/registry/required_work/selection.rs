use super::super::{
    DemandRegistryState, DemandState, ReadyCompletion, WorthQueryOutputAdvancement,
    WorthQueryOutputCheckpoint,
};
mod exact_ready;
use super::discontinuity::DiscontinuityPage;
use super::*;

/// Scheduling custody from one exact required record. The selected Product,
/// source permission, lineage row, and native actor still need authentication.
pub(in crate::domain_computation::primary_graph) struct SelectedReadyReadmission {
    pub(super) completion: ReadyCompletion,
    pub(super) readmission: Arc<super::super::source_readmission::RequiredOutputReadmission>,
    pub(super) membership: Arc<RequiredWorkMembership>,
}

impl WorthQueryOutputDemandRegistry {
    /// Join one selected scheduling hint to the existing filled Ready cell.
    /// The caller must still authenticate that completion against its admitted
    /// Product and the selected native actor/lineage image.
    pub(in crate::domain_computation::primary_graph) fn selected_ready_completion(
        &self,
        selected: &SelectedRequiredWork,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(selected.key(), admission)?;
        charge_required_key_lookup(&state, selected.key(), admission)?;
        // The exact token/version, row state, and shared Arc pin are separate
        // initialized visits from the two ordered searches.
        admission
            .charge_external_work(8)
            .map_err(|_| work_denial())?;
        if !state.required_keys.contains(selected.key()) {
            return Ok(None);
        }
        let Some(record) = state.records.get(selected.key()) else {
            return Ok(None);
        };
        let required = record.is_required()
            || super::super::refreshed_rejoin::awaited_by_stale_owner_admitted(
                &state.records,
                selected.key(),
                admission,
            )?;
        if !required
            || !record
                .work_membership
                .as_ref()
                .is_some_and(|membership| Arc::ptr_eq(membership, &selected.membership))
            || !selected.membership.is_selected_version(selected.version)
        {
            return Ok(None);
        }
        let DemandState::Output(output) = &record.state else {
            return Ok(None);
        };
        if !matches!(output.advancement, WorthQueryOutputAdvancement::Idle) {
            return Ok(None);
        }
        let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint else {
            return Ok(None);
        };
        let Some(readmission) = record.readmission_source.as_ref() else {
            return Ok(None);
        };
        Ok(Some(SelectedReadyReadmission {
            completion: completion.clone(),
            readmission: Arc::clone(readmission),
            membership: Arc::clone(
                record
                    .work_membership
                    .as_ref()
                    .expect("selected token retained"),
            ),
        }))
    }

    /// Read the installed image for the exact admitted Product basis before
    /// entering the registry. This is scheduling only; the selected work is
    /// subsequently authenticated against its current actor and lineage row.
    pub(in crate::domain_computation::primary_graph) fn observe_selected_discontinuity(
        &self,
        product: &crate::basis::WorthQueryProductObservationLease,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
        owner: &crate::domain_computation::primary_graph::SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let basis = product.relational_basis_descriptor();
        admission
            .charge_external_work(4)
            .map_err(|_| work_denial())?;
        if basis.runtime_instance_id() != selected.runtime_instance_id()
            || basis.branch_id() != selected.branch_id()
            || basis.root_identity() != selected.root_id()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "selected required-work Product basis differs from its native root",
            ));
        }
        let epoch = owner
            .selected_installed_discontinuity(selected, admission)
            .map_err(native_denial)?;
        let Some(epoch) = epoch else {
            return Ok(());
        };
        loop {
            let page = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state.page_required_discontinuity(
                    product.observation().lifecycle_incarnation(),
                    selected.branch_id(),
                    epoch,
                    admission,
                )?
            };
            match page {
                DiscontinuityPage::Advanced => continue,
                DiscontinuityPage::Queued | DiscontinuityPage::Complete => return Ok(()),
            }
        }
    }

    /// Pops only a selected required-work hint. The registry key and native
    /// settlement remain descriptive until the caller authenticates them.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn next_required_work(
        &self,
        source_owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedRequiredWork>, WorthQueryOutputDemandDenial> {
        self.drain_terminal_cleanup_admitted(source_owner, admission)?;
        let queue = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.required_work_queue.clone()
        };
        let Some(queue) = queue else {
            return Ok(None);
        };
        loop {
            admission
                .charge_external_work(6)
                .map_err(|_| work_denial())?;
            let selected = match queue.pop() {
                RequiredWorkPop::Empty => return Ok(None),
                RequiredWorkPop::Stale => continue,
                RequiredWorkPop::Selected(selected) => selected,
            };
            let live = {
                let state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state.charge_record_lookup(selected.key(), admission)?;
                charge_required_key_lookup(&state, selected.key(), admission)?;
                let live = if let Some(record) = state.records.get(selected.key()) {
                    (record.is_required()
                        || super::super::refreshed_rejoin::awaited_by_stale_owner_admitted(
                            &state.records,
                            selected.key(),
                            admission,
                        )?)
                        && record
                            .work_membership
                            .as_ref()
                            .is_some_and(|member| Arc::ptr_eq(member, &selected.membership))
                        && state.required_keys.contains(selected.key())
                } else {
                    false
                };
                if !live {
                    // Activation uses this same registry-to-token order. A new
                    // interest cannot reopen the row before deactivation.
                    selected.membership.set_required(false);
                }
                live
            };
            if live {
                return Ok(Some(selected));
            }
            drop(selected);
        }
    }

    /// Select one required cue for the already admitted Product and its exact
    /// native branch. A foreign-cell hint stays queued; a later hint for this
    /// cell cannot be hidden behind it.
    pub(in crate::domain_computation::primary_graph) fn next_required_work_for_selected(
        &self,
        product: &crate::basis::WorthQueryProductObservationLease,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
        source_owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedRequiredWork>, WorthQueryOutputDemandDenial> {
        let basis = product.relational_basis_descriptor();
        let branch_work = selected
            .branch_id()
            .0
            .len()
            .checked_add(4)
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(branch_work).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        if basis.runtime_instance_id() != selected.runtime_instance_id()
            || basis.branch_id() != selected.branch_id()
            || basis.root_identity() != selected.root_id()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "selected required-work Product differs from its native root",
            ));
        }
        admission
            .charge_external_work(2)
            .map_err(|_| work_denial())?;
        self.drain_terminal_cleanup_admitted(source_owner, admission)?;
        let queue = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.required_work_queue.clone()
        };
        let Some(queue) = queue else {
            return Ok(None);
        };
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        let queued = queue.queued_count();
        for _ in 0..queued {
            admission
                .charge_external_work(6)
                .map_err(|_| work_denial())?;
            let mut work = match queue.pop() {
                RequiredWorkPop::Empty => return Ok(None),
                RequiredWorkPop::Stale => continue,
                RequiredWorkPop::Selected(work) => work,
            };
            let membership = {
                let state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state.charge_record_lookup(work.key(), admission)?;
                charge_required_key_lookup(&state, work.key(), admission)?;
                admission
                    .charge_external_work(5)
                    .map_err(|_| work_denial())?;
                let occurrence = match state.records.get(work.key()) {
                    Some(record)
                        if (record.is_required()
                            || super::super::refreshed_rejoin::awaited_by_stale_owner_admitted(
                                &state.records,
                                work.key(),
                                admission,
                            )?)
                            && record
                                .work_membership
                                .as_ref()
                                .is_some_and(|member| Arc::ptr_eq(member, &work.membership))
                            && state.required_keys.contains(work.key()) =>
                    {
                        Some(record.product_occurrence)
                    }
                    _ => None,
                };
                if occurrence.is_none() {
                    work.membership.set_required(false);
                }
                occurrence
            };
            let Some(occurrence) = membership else {
                continue;
            };
            if occurrence != product.observation().lifecycle_incarnation() {
                continue;
            }
            if work.for_selected_branch(selected.branch_id(), admission)? {
                return Ok(Some(work));
            }
        }
        Ok(None)
    }
}

impl RequiredWorkMembership {
    fn is_selected_version(&self, version: u64) -> bool {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active && state.marked && !state.version_exhausted && state.version == version
    }
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn charge_required_key_lookup(
    state: &DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let count = state.required_keys.len();
    let levels = usize::BITS as usize - count.max(1).leading_zeros() as usize;
    let comparisons = count.min(11).checked_mul(levels).ok_or_else(work_denial)?;
    let maximum_levels =
        usize::BITS as usize - state.required_budget_bytes.max(1).leading_zeros() as usize;
    let work = 11usize
        .checked_mul(maximum_levels)
        .and_then(|comparisons| comparisons.checked_mul(key.comparison_work()?))
        .and_then(|work| work.checked_add(1))
        .ok_or_else(work_denial)?;
    admission
        .charge_external_work(u64::try_from(work).map_err(|_| work_denial())?)
        .and_then(|()| admission.charge_ordered_operations(1, comparisons as u64))
        .map_err(|_| work_denial())
}

fn native_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    let kind = match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::PublicationStale,
    };
    WorthQueryOutputDemandDenial::new(kind, "selected installed discontinuity cannot be observed")
}

pub(super) fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "selected required-work lookup exceeds request work",
    )
}

#[cfg(feature = "test-query-execution-observer")]
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn queued_required_work_for_test(&self) -> usize {
        let queue = self
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .required_work_queue
            .clone();
        queue.map_or(0, |queue| queue.queued_count())
    }
}
