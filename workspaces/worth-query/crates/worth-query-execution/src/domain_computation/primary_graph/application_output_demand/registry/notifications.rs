use super::{DemandWake, WorthQueryOutputDemandNotifications};

impl DemandWake {
    pub(super) fn notify(&self) {
        let mut generation = self
            .generation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *generation = generation.saturating_add(1);
        self.changed.notify_all();
    }
}

impl WorthQueryOutputDemandNotifications {
    pub fn generation(&self) -> u64 {
        *self
            .wake
            .generation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn wait_after(&self, observed_generation: u64) -> u64 {
        let generation = self
            .wake
            .generation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let generation = self
            .wake
            .changed
            .wait_while(generation, |generation| *generation <= observed_generation)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *generation
    }
}
