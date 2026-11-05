//! Court doors on output production's own state.
use super::{WorthQueryInstalledOutputProducerRoutes, WorthQueryOutputProduction};

impl<Schema> WorthQueryOutputProduction<Schema> {
    #[doc(hidden)]
    pub fn producer_routes_mut_for_test(&mut self) -> &mut WorthQueryInstalledOutputProducerRoutes {
        &mut self.producer_routes
    }

    /// Leaves one producer attempt ordinal before exhaustion.
    #[doc(hidden)]
    pub fn leave_one_readiness_attempt_for_test(&self) {
        self.next_producer_attempt
            .store(u64::MAX - 1, std::sync::atomic::Ordering::Relaxed);
    }
}
