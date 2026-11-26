//! The model keys a run hands out.
use super::{Action, Intent};

/// Keys in the order a run first uses them (`k1`, `k2`, ...) and the last
/// stopped envelope, which `RetryStopped` repeats. A replay may pick any
/// recorded key, a writer's too: the model keys its journal by namespace
/// (refinement (v)), so an envelope under a writer's key records a new step.
#[derive(Default)]
pub struct Keys {
    used: u32,
    pub(super) stopped: Option<(u32, Intent)>,
}

impl Keys {
    pub(super) fn fresh(&mut self) -> u32 {
        self.used += 1;
        self.used
    }

    /// An envelope under a fresh key; one that may stop is remembered.
    pub(super) fn envelope(&mut self, intent: Intent, stop: Option<u32>) -> Action {
        let key = self.fresh();
        if stop.is_some() {
            self.stopped = Some((key, intent.clone()));
        }
        Action::Envelope { key, intent, stop }
    }
}
