use crate::runtime::state::runtime_state::WeakRelationalRuntimeState;
use crate::runtime::{RelationalRuntime, RelationalRuntimeOwnerBinding};

/// Weak entry to the exact runtime state and its real operation-admission gate.
#[derive(Debug, Clone)]
pub(super) struct RelationalOwnerServiceBinding {
    state: WeakRelationalRuntimeState,
    lifecycle: RelationalRuntimeOwnerBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RelationalOwnerServiceLifecyclePosture {
    Open,
    Closing,
    Closed,
}

impl RelationalOwnerServiceBinding {
    pub(super) fn matches_owner(&self, owner: &RelationalRuntime) -> bool {
        self.state.ptr_eq(&owner.state_binding())
    }
    pub(super) fn new(
        state: WeakRelationalRuntimeState,
        lifecycle: RelationalRuntimeOwnerBinding,
    ) -> Self {
        Self { state, lifecycle }
    }

    pub(super) fn lifecycle_posture(&self) -> RelationalOwnerServiceLifecyclePosture {
        if self.lifecycle.accepts_operations() {
            return RelationalOwnerServiceLifecyclePosture::Open;
        }
        match self.state.upgrade() {
            Some(_state) => RelationalOwnerServiceLifecyclePosture::Closing,
            None => RelationalOwnerServiceLifecyclePosture::Closed,
        }
    }

    pub(super) fn admitted_runtime(&self) -> Option<RelationalRuntime> {
        let state = self.state.upgrade()?;
        let operation = self.lifecycle.admit()?;
        Some(RelationalRuntime::admitted(state, operation))
    }
}
