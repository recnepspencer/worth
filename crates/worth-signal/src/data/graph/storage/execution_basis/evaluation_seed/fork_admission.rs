use super::SignalEvaluationStorage;
use crate::data::error::SignalError;
use crate::data::graph::runtime::graph::{map_node_edit_accounting, map_node_edit_retention};
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial, SignalConditionalRetentionReservation as Reservation,
};
use std::sync::Arc;

impl SignalEvaluationStorage {
    /// Admit before conversion. Backing reservations survive either root; inline
    /// candidate custody follows installation, rejection, and unwind ownership.
    pub(in crate::data::graph) fn try_fork_persistent(
        &mut self,
        work: &mut Work,
    ) -> Result<Self, SignalError> {
        let ledger = self
            .retained_node_ledger
            .as_ref()
            .ok_or(SignalError::EvaluationStorageUnavailable)?
            .clone();
        let growth = self
            .prepare_fork_growth(work)
            .map_err(|denial| match denial {
                RetainedStorageForkGrowthDenial::PreparationRequired => {
                    SignalError::EvaluationStorageUnavailable
                }
                RetainedStorageForkGrowthDenial::Accounting(denial) => {
                    map_node_edit_accounting(denial)
                }
            })?;
        let inline = Charge::capacity::<Self>(1)
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<Reservation>()?))
            .map_err(map_node_edit_accounting)?;
        work.reserve_visits(usize::try_from(inline.bytes()).map_err(|_| {
            map_node_edit_accounting(RetainedStoragePreparationDenial::ChargeOverflow)
        })?)
        .map_err(map_node_edit_accounting)?;
        let mut resources = ledger
            .reserve(
                0,
                growth
                    .checked_add(inline)
                    .map_err(map_node_edit_accounting)?,
            )
            .map_err(map_node_edit_retention)?;
        let mut candidate = self.fork_reserved(&mut resources);
        candidate.fork_custody = Some(Arc::new(resources));
        Ok(candidate)
    }
}
