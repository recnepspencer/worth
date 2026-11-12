//! The checkout is external to its own retained payload; no same-entry Arc cycle.
use super::*;
use crate::domain_computation::primary_graph::provider::RetainedQueryDurablePreparation;

pub(in crate::domain_computation) struct QueryDurablePreparationGuard {
    reservation: Option<WorthQueryUnpublishedIdempotencyReservation>,
    preparation: Option<RetainedQueryDurablePreparation>,
}

impl std::fmt::Debug for QueryDurablePreparationGuard {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("QueryDurablePreparationGuard")
            .field("preparation", &self.preparation)
            .finish_non_exhaustive()
    }
}

impl WorthQueryUnpublishedIdempotencyReservation {
    pub(in crate::domain_computation::primary_graph::provider) fn begin_query_preparation(
        mut self,
        preparation: RetainedQueryDurablePreparation,
    ) -> QueryDurablePreparationGuard {
        // The exact row was staged before World construction. This guard is
        // its sole checkout, so no entry lock or second transfer is needed.
        self.armed = false;
        QueryDurablePreparationGuard {
            reservation: Some(self),
            preparation: Some(preparation),
        }
    }
}

impl QueryDurablePreparationGuard {
    pub(in crate::domain_computation::primary_graph::provider) fn preparation_mut(
        &mut self,
    ) -> &mut RetainedQueryDurablePreparation {
        self.preparation
            .as_mut()
            .expect("the active checkout owns the preparation")
    }

    /// A completed typed transition takes both original owners once. The
    /// reservation remains armed until the terminal releases or retains it.
    pub(in crate::domain_computation::primary_graph::provider) fn finish(
        mut self,
    ) -> (
        RetainedQueryDurablePreparation,
        WorthQueryUnpublishedIdempotencyReservation,
    ) {
        let preparation = self
            .preparation
            .take()
            .expect("completed checkout consumes its preparation");
        let mut reservation = self
            .reservation
            .take()
            .expect("completed checkout consumes its reservation");
        reservation.armed = true;
        (preparation, reservation)
    }
}

impl Drop for QueryDurablePreparationGuard {
    fn drop(&mut self) {
        let Some(preparation) = self.preparation.take() else {
            return;
        };
        let reservation = self
            .reservation
            .as_ref()
            .expect("an active preparation keeps its original reservation");
        let displaced = {
            let mut entry = reservation
                .entry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let displaced = entry.preparing.replace(preparation);
            entry.posture = WorthQueryUnpublishedIdempotencyPosture::Retained;
            displaced
        };
        // Payload Drop may release Native observations or abandon a World
        // phase. It must run after the entry and store locks have been left.
        debug_assert!(displaced.is_none());
        drop(displaced);
    }
}
