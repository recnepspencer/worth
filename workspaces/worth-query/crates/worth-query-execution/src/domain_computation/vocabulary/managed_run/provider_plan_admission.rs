//! Moved whole from `domain_computation::managed_run::workflow::provider_plan_admission`: the items below and every impl they own.

pub(in crate::domain_computation) struct WorthQueryWorkflowProviderPlanPermit {
    _owner: (),
}

impl WorthQueryWorkflowProviderPlanPermit {
    fn mint() -> Self {
        Self { _owner: () }
    }
}
