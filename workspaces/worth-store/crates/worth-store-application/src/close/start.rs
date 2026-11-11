//! Admission of the actual Store task, with an observer only on the cert door.
use worth_store::physical_runtime::{
    PhysicalStoreCloseStartRefused, PhysicalStoreCloseTask, ServingPhysicalRuntime,
};

pub(super) enum PhysicalCloseStart {
    Ordinary,
    #[cfg(feature = "certification-test-authority")]
    Observed(Box<dyn FnOnce(&worth_store::physical_runtime::PhysicalStoreClosePlan) + Send>),
}

impl PhysicalCloseStart {
    pub(super) fn start(
        self,
        serving: ServingPhysicalRuntime,
    ) -> Result<PhysicalStoreCloseTask, PhysicalStoreCloseStartRefused> {
        match self {
            Self::Ordinary => PhysicalStoreCloseTask::start(serving),
            #[cfg(feature = "certification-test-authority")]
            Self::Observed(observe) => {
                PhysicalStoreCloseTask::start_with_plan_observer_for_test(serving, observe)
            }
        }
    }
}
