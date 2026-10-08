//! A new exact posting outranks closed, unheld cached Ready custody.

use std::sync::atomic::Ordering;

use super::super::WorthQueryOutputDemandRegistry;
use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn reserve_settlement_vacancy(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(Posting, Box<PendingVacancyCleanup>), WorthQueryOutputDemandDenial> {
        let mut prior_retained = None;
        loop {
            let growth = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let growth = reservation::quote(&mut state, identity, admission)?;
                let required = state
                    .required_reserved_bytes
                    .checked_add(growth)
                    .ok_or_else(capacity_denial)?;
                if state.has_required_capacity(required) {
                    return state.reserve_settlement_vacancy(identity, admission);
                }
                let retained = state
                    .required_reserved_bytes
                    .checked_add(
                        state
                            .required_custody_retained_bytes
                            .load(Ordering::Acquire),
                    )
                    .ok_or_else(capacity_denial)?;
                if prior_retained.is_some_and(|prior| retained >= prior) {
                    // Nothing reclaimable funded this request. Concurrent
                    // custody growth cannot turn the loop into a fresh loan.
                    return Err(capacity_denial());
                }
                prior_retained = Some(retained);
                growth
            };
            // This owner retires only eligible rows and drops their custody
            // outside the registry lock. Removing a final posting can remove
            // its source root, so the next pass charges a fresh exact quote.
            self.reclaim_cached_rows(growth, admission)?;
        }
    }
}
