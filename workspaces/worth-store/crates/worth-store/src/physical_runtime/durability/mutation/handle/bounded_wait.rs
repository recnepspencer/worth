//! A deadline-bounded observation of terminal visibility; it takes no custody.
use super::*;

impl PhysicalMutationAttempt {
    /// False when the terminal is still not visible at `deadline`.
    pub(in crate::physical_runtime) fn terminal_visible_by(
        &self,
        deadline: std::time::Instant,
    ) -> bool {
        let mut state = self.state();
        loop {
            if state.terminal_visible {
                return true;
            }
            let now = std::time::Instant::now();
            if now >= deadline {
                return false;
            }
            state = self
                .changed
                .wait_timeout(state, deadline - now)
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
    }
}
