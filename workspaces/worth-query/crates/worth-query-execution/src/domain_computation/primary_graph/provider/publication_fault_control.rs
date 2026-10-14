//! Receiver-owned leases for the two native publication posture boundaries.
use super::fault_port::WorthQueryPrimaryGraphFault;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Clone, Copy, PartialEq)]
enum Boundary {
    PreTransaction,
    PostCommitSnapshot,
}
struct Registration {
    boundary: Boundary,
    consumed: bool,
}
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct PublicationFaultControl {
    active: AtomicBool,
    registration: Mutex<Option<Arc<Mutex<Registration>>>>,
}

/// A receiver already owns a live publication fault registration; no second arm is installed.
#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryPublicationFaultRegistrationDenial {
    AlreadyRegistered,
}

/// Dropping the lease disarms only this receiver's registration, never publication custody.
#[must_use = "keep the native boundary registration alive for the intended attempt"]
pub struct WorthQueryPublicationFaultGuardForTest {
    owner: Arc<PublicationFaultControl>,
    registration: Arc<Mutex<Registration>>,
}
impl Drop for WorthQueryPublicationFaultGuardForTest {
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
impl PublicationFaultControl {
    fn arm(
        self: &Arc<Self>,
        boundary: Boundary,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        let mut slot = self.registration.lock().unwrap_or_else(|p| p.into_inner());
        if slot.is_some() {
            return Err(WorthQueryPublicationFaultRegistrationDenial::AlreadyRegistered);
        }
        let registration = Arc::new(Mutex::new(Registration {
            boundary,
            consumed: false,
        }));
        *slot = Some(Arc::clone(&registration));
        self.active.store(true, Ordering::Release);
        Ok(WorthQueryPublicationFaultGuardForTest {
            owner: Arc::clone(self),
            registration,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn reject_pretransaction(
        self: &Arc<Self>,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.arm(Boundary::PreTransaction)
    }
    pub(in crate::domain_computation::primary_graph) fn hold_snapshot_failure(
        self: &Arc<Self>,
    ) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial>
    {
        self.arm(Boundary::PostCommitSnapshot)
    }
    pub(super) fn take(&self, fault: WorthQueryPrimaryGraphFault) -> bool {
        if !self.active.load(Ordering::Acquire) {
            return false;
        }
        let slot = self.registration.lock().unwrap_or_else(|p| p.into_inner());
        let Some(active) = slot.as_ref() else {
            return false;
        };
        let mut registration = active.lock().unwrap_or_else(|p| p.into_inner());
        match (registration.boundary, fault) {
            (
                Boundary::PreTransaction,
                WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction,
            ) if !registration.consumed => {
                registration.consumed = true;
                true
            }
            (
                Boundary::PostCommitSnapshot,
                WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot,
            ) => true,
            _ => false,
        }
    }
}
#[cfg(test)]
mod tests;
