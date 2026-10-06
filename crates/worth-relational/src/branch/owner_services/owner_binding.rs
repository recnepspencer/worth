use std::sync::Weak;

use crate::runtime::{
    RelationalRuntime, RelationalRuntimeAdmissionPosture, RelationalRuntimeOwnerBinding,
    RelationalRuntimeState,
};

/// Weak entry to the exact runtime state and its real operation-admission gate.
#[derive(Debug, Clone)]
pub(super) struct RelationalOwnerServiceBinding {
    state: Weak<RelationalRuntimeState>,
    lifecycle: RelationalRuntimeOwnerBinding,
}

impl RelationalOwnerServiceBinding {
    pub(super) fn new(
        state: Weak<RelationalRuntimeState>,
        lifecycle: RelationalRuntimeOwnerBinding,
    ) -> Self {
        Self { state, lifecycle }
    }

    /// The owner's admission posture, read from the owner's own lifecycle.
    ///
    /// Whether the state is still alive says nothing here: a sealed owner is
    /// closed while its state lives on, and answers closed.
    pub(super) fn lifecycle_posture(&self) -> RelationalRuntimeAdmissionPosture {
        self.lifecycle.admission_posture()
    }

    pub(super) fn admitted_runtime(&self) -> Option<RelationalRuntime> {
        let state = self.state.upgrade()?;
        let operation = self.lifecycle.admit()?;
        Some(RelationalRuntime::admitted(state, operation))
    }
}
