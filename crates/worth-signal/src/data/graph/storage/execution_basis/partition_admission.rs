//! Admission precedes immutable-root cloning and observation allocation.
use super::SignalExecutionBasis;
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::{map_node_edit_accounting, map_node_edit_retention};
use crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    SignalConditionalRetentionReservation as Reservation,
};
use std::sync::Arc;

impl SignalExecutionBasis {
    pub(crate) fn try_fork_evaluation_partition_from(
        &self,
        predecessor: &mut SignalEvaluationPartition,
        work: &mut Work,
        admission: &mut Reservation,
    ) -> Result<SignalEvaluationPartition, SignalError> {
        work.reserve_visits(std::mem::size_of::<SignalEvaluationPartition>())
            .map_err(map_node_edit_accounting)?;
        let ledger = self
            .evaluation
            .retention_ledger()
            .ok_or(SignalError::EvaluationStorageUnavailable)?;
        let charge = self
            .retained_storage_charge()
            .checked_add(
                SignalEvaluationPartition::initial_heap_charge()
                    .map_err(map_node_edit_accounting)?,
            )
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<Reservation>()?))
            .map_err(map_node_edit_accounting)?;
        let storage = Arc::new(ledger.reserve(0, charge).map_err(map_node_edit_retention)?);
        let mut evaluation = predecessor.try_fork_evaluation_storage(work)?;
        evaluation
            .prepare_seed_diagnostics(work)
            .map_err(map_node_edit_accounting)?;
        let slot = admission
            .split(1, Charge::ZERO)
            .map_err(map_node_edit_retention)?;
        Ok(SignalEvaluationPartition::from_admitted_retained(
            self.definitions.clone(),
            evaluation,
            slot,
            storage,
        ))
    }

    pub(crate) fn try_new_evaluation_partition(
        &self,
        work: &mut Work,
    ) -> Result<SignalEvaluationPartition, SignalError> {
        // Captured collection roots are shared. The diagnostic seed has only its
        // fixed bootstrap catalog; this byte-sized bound also covers its clone.
        work.reserve_visits(std::mem::size_of::<SignalEvaluationPartition>())
            .map_err(map_node_edit_accounting)?;
        let ledger = self
            .evaluation
            .retention_ledger()
            .ok_or(SignalError::EvaluationStorageUnavailable)?;
        let charge = self
            .retained_storage_charge()
            .checked_add(
                SignalEvaluationPartition::initial_heap_charge()
                    .map_err(map_node_edit_accounting)?,
            )
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<Reservation>()?))
            .map_err(map_node_edit_accounting)?;
        // Conservatively charge the inline handles as well as both reservation
        // handles. Slot custody drops separately from surviving payload storage.
        let slot = ledger
            .reserve(1, Charge::ZERO)
            .map_err(map_node_edit_retention)?;
        let storage = Arc::new(ledger.reserve(0, charge).map_err(map_node_edit_retention)?);
        let mut evaluation = self.evaluation.clone();
        evaluation
            .prepare_seed_diagnostics(work)
            .map_err(map_node_edit_accounting)?;
        Ok(SignalEvaluationPartition::from_admitted_retained(
            self.definitions.clone(),
            evaluation,
            slot,
            storage,
        ))
    }

    pub(crate) fn try_new_evaluation_partition_with_reserved_slot(
        &self,
        work: &mut Work,
        admission: &mut Reservation,
    ) -> Result<SignalEvaluationPartition, SignalError> {
        work.reserve_visits(std::mem::size_of::<SignalEvaluationPartition>())
            .map_err(map_node_edit_accounting)?;
        let ledger = self
            .evaluation
            .retention_ledger()
            .ok_or(SignalError::EvaluationStorageUnavailable)?;
        let charge = self
            .retained_storage_charge()
            .checked_add(
                SignalEvaluationPartition::initial_heap_charge()
                    .map_err(map_node_edit_accounting)?,
            )
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<Reservation>()?))
            .map_err(map_node_edit_accounting)?;
        let storage = Arc::new(ledger.reserve(0, charge).map_err(map_node_edit_retention)?);
        let mut evaluation = self.evaluation.clone();
        evaluation
            .prepare_seed_diagnostics(work)
            .map_err(map_node_edit_accounting)?;
        let slot = admission
            .split(1, Charge::ZERO)
            .map_err(map_node_edit_retention)?;
        Ok(SignalEvaluationPartition::from_admitted_retained(
            self.definitions.clone(),
            evaluation,
            slot,
            storage,
        ))
    }
}
