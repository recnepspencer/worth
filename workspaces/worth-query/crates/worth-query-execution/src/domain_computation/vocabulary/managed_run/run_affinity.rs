//! Moved whole from `domain_computation::managed_run::workflow::run_affinity`: the items below and every impl they own.

pub(in crate::domain_computation) struct WorthQueryWorkflowRunTransitionPermit {
    _owner: (),
}

impl WorthQueryWorkflowRunTransitionPermit {
    const fn mint() -> Self {
        Self { _owner: () }
    }
}
