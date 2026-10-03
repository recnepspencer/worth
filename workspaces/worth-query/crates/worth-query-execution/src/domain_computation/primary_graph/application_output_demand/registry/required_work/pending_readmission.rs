//! Exact consumed-edge join to the already retained required demand.

use super::super::{DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint};
use super::selection::{charge_required_key_lookup, work_denial, SelectedReadyReadmission};
use super::*;

impl WorthQueryOutputDemandRegistry {
    /// Scheduling lookup only: the admitted Product and actor still certify
    /// the pinned output before a downstream pending edge can be discharged.
    pub(in crate::domain_computation::primary_graph) fn pending_exact_ready_readmission(
        &self,
        pending: &crate::domain_computation::primary_graph::invariant_projection::SelectedPendingConsumedOutput<'_>,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        let branch_work = selected
            .branch_id()
            .0
            .len()
            .checked_add(3)
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(branch_work).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        if pending.selected_root() != selected {
            return Ok(None);
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(key) = state
            .settlement_keys
            .get_exact_admitted(pending.identity(), admission)?
        else {
            return Ok(None);
        };
        state.charge_record_lookup(&key, admission)?;
        charge_required_key_lookup(&state, &key, admission)?;
        admission
            .charge_external_work(8)
            .map_err(|_| work_denial())?;
        if !state.required_keys.contains(&key) {
            return Ok(None);
        }
        let Some(record) = state.records.get(key.as_ref()) else {
            return Ok(None);
        };
        if !record.is_required() {
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
        let (Some(readmission), Some(membership)) = (
            record.readmission_source.as_ref(),
            record.work_membership.as_ref(),
        ) else {
            return Ok(None);
        };
        Ok(Some(SelectedReadyReadmission {
            completion: completion.clone(),
            readmission: Arc::clone(readmission),
            membership: Arc::clone(membership),
        }))
    }
}
