//! Receiver-owned capture of an authentic, port-free issued grant.
use super::{
    WorthQueryProviderSessionDenialKind as Kind, WorthQueryProviderSessionFailure as Failure,
    WorthQueryProviderSessionGrant as Grant, WorthQueryProviderSessionProtocolStage as Stage,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Default)]
pub(in crate::domain_computation) struct ProviderSessionGrantControl {
    armed: AtomicBool,
    registration: Mutex<Option<Arc<Mutex<Registration>>>>,
}
enum Action {
    Capture,
    Consumed,
}
struct Registration {
    action: Action,
    captured: Option<Grant>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionGrantControlDenial {
    AlreadyRegistered,
}

/// Holds only an independent control cell and port-free grants. It cannot keep
/// the graph provider, product, running request or Home lease alive.
#[must_use = "keep the receiver registration until the scheduled admission"]
pub struct WorthQueryProviderSessionGrantControlForTest {
    owner: Arc<ProviderSessionGrantControl>,
    registration: Arc<Mutex<Registration>>,
}

impl ProviderSessionGrantControl {
    pub(in crate::domain_computation) fn arm(
        self: &Arc<Self>,
    ) -> Result<
        WorthQueryProviderSessionGrantControlForTest,
        WorthQueryProviderSessionGrantControlDenial,
    > {
        let mut slot = self.registration.lock().unwrap_or_else(|p| p.into_inner());
        if slot.is_some() {
            return Err(WorthQueryProviderSessionGrantControlDenial::AlreadyRegistered);
        }
        let registration = Arc::new(Mutex::new(Registration {
            action: Action::Capture,
            captured: None,
        }));
        *slot = Some(Arc::clone(&registration));
        self.armed.store(true, Ordering::Release);
        Ok(WorthQueryProviderSessionGrantControlForTest {
            owner: Arc::clone(self),
            registration,
        })
    }

    pub(super) fn capture(&self, grant: Grant) -> Result<Grant, Failure> {
        if !self.armed.load(Ordering::Acquire) {
            return Ok(grant);
        }
        let slot = self.registration.lock().unwrap_or_else(|p| p.into_inner());
        let Some(cell) = slot.as_ref() else {
            return Ok(grant);
        };
        let mut state = cell.lock().unwrap_or_else(|p| p.into_inner());
        match std::mem::replace(&mut state.action, Action::Consumed) {
            Action::Capture => {
                let counters = grant.counters();
                state.captured = Some(grant);
                Err(Failure::new(
                    Kind::ProviderRejected,
                    Stage::PlanReadmission,
                    "receiver capture stopped before physical session readmission",
                    counters,
                ))
            }
            Action::Consumed => Ok(grant),
        }
    }
}

impl WorthQueryProviderSessionGrantControlForTest {
    /// Transfers the actual issued grant; this never constructs authority from
    /// an observation or changes its original Home identity or contract.
    pub fn take_captured(&self) -> Option<Grant> {
        self.registration
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .captured
            .take()
    }
}
impl Drop for WorthQueryProviderSessionGrantControlForTest {
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
            self.owner.armed.store(false, Ordering::Release);
        }
    }
}
