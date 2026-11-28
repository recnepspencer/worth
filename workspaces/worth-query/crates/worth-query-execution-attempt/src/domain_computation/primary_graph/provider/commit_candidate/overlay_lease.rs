use super::super::application_attempt_state::SharedApplicationAttemptStore;
use crate::domain_computation::WorthQueryProviderSessionTerminalBinding;

/// Holds the registered candidate phase until the provider takes publication custody.
pub(super) struct CandidatePhaseLease {
    attempts: SharedApplicationAttemptStore,
    binding: WorthQueryProviderSessionTerminalBinding,
    active: bool,
}

impl CandidatePhaseLease {
    pub(super) fn new(
        attempts: SharedApplicationAttemptStore,
        binding: WorthQueryProviderSessionTerminalBinding,
    ) -> Self {
        Self {
            attempts,
            binding,
            active: true,
        }
    }

    pub(super) fn discard(&mut self) {
        if self.active {
            self.active = false;
            let retired = self
                .attempts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .retire_candidate(&self.binding);
            // Retired members can contain application-owned completion values.
            // Match session cleanup's unwind isolation after releasing the store.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(retired)));
        }
    }

    pub(super) fn release_to_provider_resolution(&mut self) {
        self.active = false;
    }
}

impl Drop for CandidatePhaseLease {
    fn drop(&mut self) {
        self.discard();
    }
}
