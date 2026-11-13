//! Consuming Home admission checks actual output custody without discharging it.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
};

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
    ) -> Result<(), Denial> {
        let state = Arc::get_mut(&mut self.state)
            .ok_or(Denial::Busy(Owner::InstalledProducers))?
            .get_mut()
            .map_err(|_| Denial::Busy(Owner::InstalledProducers))?;
        if !state.records.is_empty()
            || state.join_pause.is_some()
            || !state.required_keys.is_empty()
            || state.required_work_queue.is_some()
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
    fn invisible_required_work_and_retention_custody_refuse_without_visible_rows() {
        let mut original = WorthQueryOutputDemandRegistry::default();
        original.state.lock().unwrap().required_work_queue =
            Some(Arc::new(required_work::RequiredWorkQueue::new()));
        assert!(original.state.lock().unwrap().records.is_empty());
        refused(&mut original);
        assert!(original.state.lock().unwrap().required_work_queue.is_some());
        let mut reserved = WorthQueryOutputDemandRegistry::default();
        let reservation = reserved.arm_retention_reservation().unwrap();
        refused(&mut reserved);
        drop(reservation);
        assert!(reserved.preflight_home_retirement().is_ok());
    }
}
