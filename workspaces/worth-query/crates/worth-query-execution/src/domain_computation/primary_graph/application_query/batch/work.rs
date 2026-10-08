use super::*;

/// Reserves the maximum before the kernel starts. A terminal read settles
/// actual root/tree work; dropping without settlement keeps the maximum spent.
pub(in crate::domain_computation::primary_graph::application_query) struct BatchReadReservation {
    totals: Arc<Mutex<BatchTotals>>,
    maximum: usize,
}

impl WorthQueryApplicationQueryBatchAdmission {
    pub(in crate::domain_computation::primary_graph::application_query) fn reserve_read(
        &self,
        declared_maximum: usize,
    ) -> Result<BatchReadReservation, WorthQueryApplicationQueryBatchResourceDenial> {
        let mut totals = self
            .totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remaining = self
            .policy
            .maximum_work()
            .checked_sub(totals.work)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::WorkAccountingMismatch)?;
        if remaining == 0 {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::WorkLimit {
                required: totals
                    .work
                    .checked_add(1)
                    .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?,
                maximum: self.policy.maximum_work(),
            });
        }
        let maximum = declared_maximum.min(remaining);
        totals.work = totals
            .work
            .checked_add(maximum)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        Ok(BatchReadReservation {
            totals: Arc::clone(&self.totals),
            maximum,
        })
    }
}

impl BatchReadReservation {
    pub(in crate::domain_computation::primary_graph::application_query) fn maximum(&self) -> usize {
        self.maximum
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn settle(
        self,
        actual: usize,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        let unused = self
            .maximum
            .checked_sub(actual)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::WorkAccountingMismatch)?;
        let mut totals = self
            .totals
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        totals.work = totals
            .work
            .checked_sub(unused)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::WorkAccountingMismatch)?;
        Ok(())
    }
}
