//! A receiver-owned schedule exercises actual lower terminal cleanup on a native worker.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ManagedCleanupControl {
    active: AtomicBool,
    registration: Mutex<Option<Arc<Mutex<bool>>>>,
}
/// A receiver already owns a live managed-cleanup scheduling registration.
#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryManagedCleanupRegistrationDenial {
    AlreadyRegistered,
}
/// Drop disarms this receiver only; already returned native failure custody is independent.
#[must_use = "keep the receiver registration alive for the intended native cleanup"]
pub struct WorthQueryManagedCleanupScheduleForTest {
    owner: Arc<ManagedCleanupControl>,
    registration: Arc<Mutex<bool>>,
}
impl ManagedCleanupControl {
    pub(in crate::domain_computation::primary_graph) fn arm(
        self: &Arc<Self>,
    ) -> Result<WorthQueryManagedCleanupScheduleForTest, WorthQueryManagedCleanupRegistrationDenial>
    {
        let mut slot = self.registration.lock().unwrap_or_else(|p| p.into_inner());
        if slot.is_some() {
            return Err(WorthQueryManagedCleanupRegistrationDenial::AlreadyRegistered);
        }
        let registration = Arc::new(Mutex::new(false));
        *slot = Some(Arc::clone(&registration));
        self.active.store(true, Ordering::Release);
        Ok(WorthQueryManagedCleanupScheduleForTest {
            owner: Arc::clone(self),
            registration,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn take(&self) -> bool {
        if !self.active.load(Ordering::Acquire) {
            return false;
        }
        let slot = self.registration.lock().unwrap_or_else(|p| p.into_inner());
        let Some(registration) = slot.as_ref() else {
            return false;
        };
        let mut consumed = registration.lock().unwrap_or_else(|p| p.into_inner());
        if *consumed {
            false
        } else {
            *consumed = true;
            true
        }
    }
}
impl Drop for WorthQueryManagedCleanupScheduleForTest {
    fn drop(&mut self) {
        let mut slot = self
            .owner
            .registration
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if slot
            .as_ref()
            .is_some_and(|active| Arc::ptr_eq(active, &self.registration))
        {
            *slot = None;
            self.owner.active.store(false, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receiver_leases_are_independent_conflicting_and_one_shot() {
        let a = Arc::new(ManagedCleanupControl::default());
        let b = Arc::new(ManagedCleanupControl::default());
        let ga = a.arm().unwrap();
        let gb = b.arm().unwrap();
        assert!(matches!(
            a.arm(),
            Err(WorthQueryManagedCleanupRegistrationDenial::AlreadyRegistered)
        ));
        assert!(a.take());
        assert!(!a.take());
        assert!(b.take());
        drop(ga);
        let _next = a.arm().unwrap();
        assert!(a.take());
        drop(gb);
    }
    #[test]
    fn unwind_disarms_only_the_owning_receiver() {
        let a = Arc::new(ManagedCleanupControl::default());
        let b = Arc::new(ManagedCleanupControl::default());
        let _b = b.arm().unwrap();
        let _ = std::panic::catch_unwind(|| {
            let _a = a.arm().unwrap();
            panic!("native guard unwind");
        });
        assert!(!a.take());
        assert!(b.take());
        let _a = a.arm().unwrap();
        assert!(a.take());
    }
}
