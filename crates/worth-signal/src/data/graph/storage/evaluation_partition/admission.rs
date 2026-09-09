//! Initial partition storage and observation lifetime; later growth admits separately.
use super::{EvaluationObservationStorage, SignalEvaluationPartition};
use crate::data::graph::storage::execution_basis::{
    SignalEvaluationStorage, SignalExecutionDefinitions,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparationDenial,
    SignalConditionalRetentionReservation as Reservation,
};
use std::sync::Arc;

impl SignalEvaluationPartition {
    pub(in crate::data::graph) fn initial_heap_charge(
    ) -> Result<Charge, RetainedStoragePreparationDenial> {
        Charge::capacity::<Self>(1)?
            .checked_add(EvaluationObservationStorage::initial_heap_charge()?)
    }

    pub(in crate::data::graph) fn from_admitted_retained(
        definitions: SignalExecutionDefinitions,
        mut evaluation: SignalEvaluationStorage,
        slot: Reservation,
        storage: Arc<Reservation>,
    ) -> Self {
        let observation = EvaluationObservationStorage::new(
            definitions.installed_policy(),
            Some(Arc::clone(&storage)),
        );
        evaluation.retain_seed_custody(storage);
        Self {
            definitions,
            evaluation,
            observation,
            traversal: Default::default(),
            pending_unwind: None,
            _slot_custody: Some(slot),
        }
    }
}
