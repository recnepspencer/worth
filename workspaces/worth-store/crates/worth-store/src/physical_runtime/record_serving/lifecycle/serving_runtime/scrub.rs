use super::ServingPhysicalRuntime;
use crate::physical_runtime::{
    ManagedPhysicalIntegrityScrubHandle, ManagedPhysicalIntegrityScrubRequest,
    PhysicalIntegrityScrubRequestDenial,
};

impl ServingPhysicalRuntime {
    /// Starts a bounded diagnostic session over explicit expected targets.
    /// Windows read actual media using Store-owned C6 memory and C5 background
    /// pacing. Completion is limited to the declared scope and grants no repair.
    pub fn start_physical_integrity_scrub(
        &self,
        request: ManagedPhysicalIntegrityScrubRequest,
    ) -> Result<ManagedPhysicalIntegrityScrubHandle, PhysicalIntegrityScrubRequestDenial> {
        if request.store != self.store_identity()
            || request
                .targets()
                .iter()
                .any(|target| target.scope().store_identity() != self.store_identity())
        {
            return Err(PhysicalIntegrityScrubRequestDenial::RuntimeScopeMismatch);
        }
        let read = crate::physical_runtime::record_serving::CanonicalRecordReadPort::new(
            &self.parts.work_runtime,
            self.parts.core.lifecycle_generation(),
            self.parts.work_admission,
            self.parts.scheduler_admission.clone(),
            self.parts.record_work.clone(),
        );
        ManagedPhysicalIntegrityScrubHandle::start(
            request,
            &self.scrub,
            read,
            self.parts.residency.ports().clone(),
            self.parts.core.lifecycle_state(),
            self.runtime_identity(),
        )
    }
}
