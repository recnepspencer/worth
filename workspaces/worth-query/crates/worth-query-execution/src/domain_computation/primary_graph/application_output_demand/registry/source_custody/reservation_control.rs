//! Receiver-owned control of the actual abandoned-source reservation primitive.
use super::super::{DemandRegistryState, WorthQueryOutputDemandRegistry};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
/// A reservation registration already lives on this exact registry receiver.
#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryDiscoveredRetentionControlDenial {
    AlreadyRegistered,
}
/// One actual eligible reservation may refuse; drop disarms only this registration.
/// This proves native reservation-capacity refusal, not ordinary allocator OOM.
#[must_use]
pub struct WorthQueryDiscoveredRetentionReservationForTest {
    owner: Arc<Mutex<DemandRegistryState>>,
    registration: Arc<AtomicBool>,
}
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn arm_retention_reservation(
        &self,
    ) -> Result<
        WorthQueryDiscoveredRetentionReservationForTest,
        WorthQueryDiscoveredRetentionControlDenial,
    > {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if state.retention_reservation.is_some() {
            return Err(WorthQueryDiscoveredRetentionControlDenial::AlreadyRegistered);
        }
        let registration = Arc::new(AtomicBool::new(true));
        state.retention_reservation = Some(Arc::clone(&registration));
        Ok(WorthQueryDiscoveredRetentionReservationForTest {
            owner: Arc::clone(&self.state),
            registration,
        })
    }
}
/// Called only for the actual eligible discovered abandoned-source vector.
pub(super) fn reservation_request(state: &DemandRegistryState) -> usize {
    if state
        .retention_reservation
        .as_ref()
        .is_some_and(|registration| registration.swap(false, Ordering::AcqRel))
    {
        usize::MAX
    } else {
        1
    }
}
impl Drop for WorthQueryDiscoveredRetentionReservationForTest {
    fn drop(&mut self) {
        let mut state = self.owner.lock().unwrap_or_else(|p| p.into_inner());
        if state
            .retention_reservation
            .as_ref()
            .is_some_and(|active| Arc::ptr_eq(active, &self.registration))
        {
            state.retention_reservation = None;
        }
    }
}
impl<S: worth_query_installation::facade::ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>
{
    /// Controls one real discovered abandoned-source Vec reservation on this registry.
    /// Ordinary reservations remain exactly one; no native error is fabricated.
    pub fn refuse_next_discovered_retention_reservation_for_test(
        &self,
    ) -> Result<
        WorthQueryDiscoveredRetentionReservationForTest,
        WorthQueryDiscoveredRetentionControlDenial,
    > {
        self.output_demands.arm_retention_reservation()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registrations_are_receiver_owned_conflicting_one_shot_and_unwind_safe() {
        let a = WorthQueryOutputDemandRegistry::default();
        let b = WorthQueryOutputDemandRegistry::default();
        let guard_a = a.arm_retention_reservation().unwrap();
        let guard_b = b.arm_retention_reservation().unwrap();
        assert!(matches!(
            a.arm_retention_reservation(),
            Err(WorthQueryDiscoveredRetentionControlDenial::AlreadyRegistered)
        ));
        assert_eq!(reservation_request(&a.state.lock().unwrap()), usize::MAX);
        assert_eq!(reservation_request(&a.state.lock().unwrap()), 1);
        assert!(
            a.arm_retention_reservation().is_err(),
            "consumption does not end the guard's registration"
        );
        drop(guard_a);
        assert_eq!(reservation_request(&a.state.lock().unwrap()), 1);
        assert_eq!(reservation_request(&b.state.lock().unwrap()), usize::MAX);
        let _ = std::panic::catch_unwind(|| {
            let _guard = a.arm_retention_reservation().unwrap();
            panic!("guard unwind");
        });
        assert_eq!(reservation_request(&a.state.lock().unwrap()), 1);
        assert!(
            b.arm_retention_reservation().is_err(),
            "A unwind cannot disarm B"
        );
        drop(guard_b);
        assert!(b.arm_retention_reservation().is_ok());
    }
}
