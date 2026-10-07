use super::ServingPhysicalRuntime;
use crate::physical_runtime::{
    ManagedPhysicalIntegrityScrubHandle, ManagedPhysicalIntegrityScrubRequest,
    PhysicalIntegrityScrubRequestDenial, PhysicalIntegrityScrubSource,
};

impl ServingPhysicalRuntime {
    /// Starts a bounded diagnostic session over explicit expected targets.
    /// Windows use Store-owned C6 memory and C5 diagnostic-background pacing.
    /// Selected records additionally retain a protected C5 root and route.
    /// Completion grants no repair authority.
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
        let selected_reader = if request.targets().iter().any(|target| {
            matches!(
                target.source(),
                PhysicalIntegrityScrubSource::SelectedRecord(_)
            )
        }) {
            let reader = self
                .records()
                .map_err(PhysicalIntegrityScrubRequestDenial::RootProtection)?;
            if request.targets().iter().any(|target| {
                target
                    .issued_under()
                    .is_some_and(|root| root != reader.protected_root())
            }) {
                return Err(PhysicalIntegrityScrubRequestDenial::SelectedRootChanged);
            }
            Some(reader.for_diagnostic_scrub())
        } else {
            None
        };
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
            selected_reader,
            self.parts.residency.ports().clone(),
            self.parts.core.lifecycle_state(),
            self.runtime_identity(),
        )
    }
}
