//! Consuming Home admission reclaims idle caches and checks live output custody.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
};

impl WorthQueryOutputDemandRegistry {
    /// Closed answers and stale scheduling hints carry no live demand.
    pub(in crate::domain_computation::primary_graph) fn reclaim_home_retirement_cache(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        loop {
            request.consult().map_err(Denial::Execution)?;
            if self
                .reclaim_unheld_observation()
                .map_err(|_| Denial::Busy(Owner::InstalledProducers))?
            {
                continue;
            }
            let queue = self
                .state
                .lock()
                .map_err(|_| Denial::Busy(Owner::InstalledProducers))?
                .required_work_queue
                .clone();
            let Some(queue) = queue else {
                return Ok(());
            };
            match queue.pop() {
                required_work::RequiredWorkPop::Stale => continue,
                required_work::RequiredWorkPop::Empty => return Ok(()),
                // Dropping the claim requeues the original live work.
                required_work::RequiredWorkPop::Claimed(_) => {
                    return Err(Denial::Busy(Owner::InstalledProducers));
                }
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
    ) -> Result<(), Denial> {
        let state = Arc::get_mut(&mut self.state)
            .ok_or(Denial::Busy(Owner::InstalledProducers))?
            .get_mut()
            .map_err(|_| Denial::Busy(Owner::InstalledProducers))?;
        // Native observers may retain inactive membership and its weak queue
        // reference. Only live registry custody and queued work prevent close.
        if state
            .required_work_queue
            .as_ref()
            .is_some_and(|queue| !queue.is_quiescent())
        {
            return Err(Denial::Busy(Owner::InstalledProducers));
        }
        if !state.records.is_empty()
            || state.join_pause.is_some()
            || !state.required_keys.is_empty()
            || !state.source_preparations.is_empty()
            || !state.source_custody.is_empty()
            || state.retention_reservation.is_some()
            || state.pending_cleanup_head.is_some()
            || state.custody_cleanup.head.is_some()
            || state.custody_cleanup.len != 0
            || !state.closed_cleanup.is_quiescent()
            || !state.settlement_keys.is_quiescent()
        {
            return Err(Denial::Busy(Owner::InstalledProducers));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(registry: &mut WorthQueryOutputDemandRegistry) {
        assert!(matches!(
            registry.preflight_home_retirement(),
            Err(Denial::Busy(Owner::InstalledProducers))
        ));
    }

    #[test]
    fn strong_and_weak_output_aliases_refuse_until_the_original_is_exclusive() {
        let mut original = WorthQueryOutputDemandRegistry::default();
        let alias = original.clone();
        refused(&mut original);
        drop(alias);
        let weak = Arc::downgrade(&original.state);
        refused(&mut original);
        drop(weak);
        assert!(original.preflight_home_retirement().is_ok());
    }

    #[test]
    fn poisoned_output_custody_is_refused_without_recovery_or_replacement() {
        let mut original = WorthQueryOutputDemandRegistry::default();
        let _ = std::panic::catch_unwind(|| {
            let _held = original.state.lock().unwrap();
            panic!("poison the actual owner");
        });
        refused(&mut original);
        assert!(original.state.is_poisoned());
    }

    #[test]
    fn retention_reservation_refuses_without_visible_rows() {
        let mut reserved = WorthQueryOutputDemandRegistry::default();
        let reservation = reserved.arm_retention_reservation().unwrap();
        refused(&mut reserved);
        drop(reservation);
        assert!(reserved.preflight_home_retirement().is_ok());
    }
}
