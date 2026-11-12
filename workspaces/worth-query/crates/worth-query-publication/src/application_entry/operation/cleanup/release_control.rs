//! Scoped cancellation after actual native release; it never creates cleanup evidence.
use std::sync::{Arc, Mutex};
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
#[derive(Default)]
pub(super) struct CleanupReleaseControl {
    registration: Option<Arc<Mutex<Registration>>>,
}
struct Registration {
    active: bool,
    remaining: usize,
    cancellation: WorthQueryCancellationSource,
}
#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryCleanupReleaseRegistrationDenial {
    AlreadyRegistered,
}
#[must_use = "retain this receiver's scoped interruption registration"]
pub struct WorthQueryCleanupReleaseInterruptionForTest {
    registration: Arc<Mutex<Registration>>,
}
impl CleanupReleaseControl {
    pub(super) fn arm(
        &mut self,
        count: std::num::NonZeroUsize,
        cancellation: WorthQueryCancellationSource,
    ) -> Result<
        WorthQueryCleanupReleaseInterruptionForTest,
        WorthQueryCleanupReleaseRegistrationDenial,
    > {
        if self
            .registration
            .as_ref()
            .is_some_and(|active| active.lock().unwrap_or_else(|p| p.into_inner()).active)
        {
            return Err(WorthQueryCleanupReleaseRegistrationDenial::AlreadyRegistered);
        }
        let registration = Arc::new(Mutex::new(Registration {
            active: true,
            remaining: count.get(),
            cancellation,
        }));
        self.registration = Some(Arc::clone(&registration));
        Ok(WorthQueryCleanupReleaseInterruptionForTest { registration })
    }
    pub(super) fn after_release(&self) {
        let Some(registration) = &self.registration else {
            return;
        };
        let mut registration = registration.lock().unwrap_or_else(|p| p.into_inner());
        if registration.active && registration.remaining > 0 {
            registration.remaining -= 1;
            if registration.remaining == 0 {
                registration.cancellation.cancel();
            }
        }
    }
}
impl Drop for WorthQueryCleanupReleaseInterruptionForTest {
    fn drop(&mut self) {
        self.registration
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .active = false;
    }
}
