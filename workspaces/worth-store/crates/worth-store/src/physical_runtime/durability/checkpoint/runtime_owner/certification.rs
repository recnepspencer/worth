use super::PhysicalCheckpointRuntimeOwner;

impl PhysicalCheckpointRuntimeOwner {
    pub(in crate::physical_runtime) fn certification_checkpoint_under_pressure(
        &self,
        foreground_pressure_events: u64,
    ) -> bool {
        self.capture
            .certification_checkpoint_under_pressure(foreground_pressure_events)
    }

    pub(in crate::physical_runtime) fn certification_fail_next_admission(&self) {
        self.capture.fail_next_admission();
    }

    pub(in crate::physical_runtime) fn certification_reclamation_under_pressure(
        &self,
        foreground_pressure_events: u64,
    ) -> bool {
        self.capture
            .certification_reclamation_under_pressure(foreground_pressure_events)
    }
}
