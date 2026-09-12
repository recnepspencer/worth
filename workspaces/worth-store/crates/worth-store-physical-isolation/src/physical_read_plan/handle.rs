use super::{
    PhysicalReadPlanAdmissionDenial, PhysicalReadPlanCompletionReceipt,
    PhysicalReadPlanReleaseReceipt, StablePhysicalReadPlan,
};
use crate::CurrentGenerationPhysicalReference;

#[derive(Debug)]
pub struct StablePhysicalReadHandle {
    plan: StablePhysicalReadPlan,
    released: bool,
}

impl StablePhysicalReadHandle {
    pub(crate) const fn new(plan: StablePhysicalReadPlan) -> Self {
        Self {
            plan,
            released: false,
        }
    }

    pub const fn plan(&self) -> &StablePhysicalReadPlan {
        &self.plan
    }

    pub fn read_protected_reference(
        &self,
        reference: CurrentGenerationPhysicalReference,
    ) -> Result<(), PhysicalReadPlanAdmissionDenial> {
        if self.plan.footprint().admits_reference(reference) {
            Ok(())
        } else {
            Err(PhysicalReadPlanAdmissionDenial::ExecutionTimeReferenceDiscovery)
        }
    }

    /// Complete local planning obligations without claiming byte execution.
    pub fn complete_plan(self) -> PhysicalReadPlanCompletionReceipt {
        let counters = self.plan.counters();
        PhysicalReadPlanCompletionReceipt::new(self.release(), counters)
    }

    pub fn release(mut self) -> PhysicalReadPlanReleaseReceipt {
        self.released = true;
        PhysicalReadPlanReleaseReceipt::new(
            self.plan.root(),
            self.plan.reachability_barrier().footprint_basis(),
        )
    }
}
