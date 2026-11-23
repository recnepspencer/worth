//! Exact owner-state sharing and weak admission bindings.
use super::RelationalRuntimeState;
use std::sync::{Arc, Weak};

pub(super) type SharedRelationalRuntimeState = Arc<dyn RuntimeStateBacking>;
pub(crate) type WeakRelationalRuntimeState = Weak<dyn RuntimeStateBacking>;

// Prove thread-safe sharing at the runtime owner. Transferring its state
// binding does not require a fresh proof over all of the subsystem backing.
pub(crate) trait RuntimeStateBacking: std::fmt::Debug + Send + Sync {
    fn runtime_state(&self) -> &RelationalRuntimeState;
    fn runtime_state_mut(&mut self) -> &mut RelationalRuntimeState;
}

impl RuntimeStateBacking for RelationalRuntimeState {
    fn runtime_state(&self) -> &RelationalRuntimeState {
        self
    }

    fn runtime_state_mut(&mut self) -> &mut RelationalRuntimeState {
        self
    }
}

impl std::ops::Deref for dyn RuntimeStateBacking {
    type Target = RelationalRuntimeState;

    fn deref(&self) -> &Self::Target {
        self.runtime_state()
    }
}
