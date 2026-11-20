use crate::domain_computation::WorthQueryGraphProviderStepRetainedEvidence;

pub(super) struct WorthQueryManagedYieldSafePoint {
    checkpoint_available: bool,
    retained: WorthQueryGraphProviderStepRetainedEvidence,
}

impl WorthQueryManagedYieldSafePoint {
    pub(super) const fn new(
        checkpoint_available: bool,
        retained: WorthQueryGraphProviderStepRetainedEvidence,
    ) -> Self {
        Self {
            checkpoint_available,
            retained,
        }
    }

    pub(super) const fn checkpoint_available(&self) -> bool {
        self.checkpoint_available
    }

    pub(super) const fn retained(&self) -> WorthQueryGraphProviderStepRetainedEvidence {
        self.retained
    }
}
