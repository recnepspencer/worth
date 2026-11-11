//! Receiver-owned placement and observation controls on actual Home admission.
use crate::open::physical_admission::PhysicalOpening;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalRecoveryProcessYieldpoint, ServingPhysicalRuntime,
};

#[derive(Default)]
pub(crate) struct HomeControlsForTest {
    pub(crate) opening: Option<PhysicalOpening>,
    pub(crate) relational: Option<
        Box<
            dyn Fn(
                    worth_relational::facade::publication::PatchStreamPosition,
                    worth_store_relational::RelationalStoreColdReadCounters,
                ) + Send
                + Sync,
        >,
    >,
}

impl crate::ApplicationHomeRequest {
    pub fn with_media_for_test(mut self, media: FilesystemMediaAdmission) -> Self {
        let opening = self
            .controls
            .opening
            .take()
            .unwrap_or_else(PhysicalOpening::production);
        self.controls.opening = Some(opening.with_media(media));
        self
    }

    pub fn with_recovery_yieldpoint_for_test(
        mut self,
        point: PhysicalRecoveryProcessYieldpoint,
    ) -> Self {
        let opening = self
            .controls
            .opening
            .take()
            .unwrap_or_else(PhysicalOpening::production);
        self.controls.opening = Some(opening.with_recovery_yieldpoint(point));
        self
    }

    pub fn recover_with_descriptor_observer_for_test(
        self,
        media: FilesystemMediaAdmission,
        observer: impl FnOnce(&ServingPhysicalRuntime),
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_proof::StoreRecovered<crate::RecoveredPhysicalHome>,
        crate::ApplicationHomeOpenRefusal,
    > {
        crate::open::recover_home_observed(self.with_media_for_test(media), execution, observer)
    }

    pub fn observe_relational_reads_for_test(
        mut self,
        observer: impl Fn(
                worth_relational::facade::publication::PatchStreamPosition,
                worth_store_relational::RelationalStoreColdReadCounters,
            ) + Send
            + Sync
            + 'static,
    ) -> Self {
        self.controls.relational = Some(Box::new(observer));
        self
    }
}

impl crate::ReadmittedApplicationHome {
    /// Observe the actual original input tickets, without allocating or quoting.
    pub fn checkpoint_input_memory_bytes_for_test(&self) -> Option<u64> {
        self.context.checkpoint.input_memory_bytes_for_test()
    }
    pub fn checkpoint_with_step_observer_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observer: impl Fn(&worth_store::physical_runtime::AnchoredRecordStepRegistration) + Send + Sync,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        crate::checkpoint::journey::checkpoint(
            &self.context.serving,
            &self.context.checkpoint,
            None,
            None,
            execution,
            |home, through, digest| {
                self.relational
                    .complete_checkpoint_preparation()
                    .prepare_home(home, through, digest, execution)
            },
            Some(&observer),
            None,
            None,
        )
    }

    pub fn checkpoint_with_acknowledgement_observer_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observer: impl Fn() + Send + Sync,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        crate::checkpoint::journey::checkpoint(
            &self.context.serving,
            &self.context.checkpoint,
            None,
            None,
            execution,
            |home, through, digest| {
                self.relational
                    .complete_checkpoint_preparation()
                    .prepare_home(home, through, digest, execution)
            },
            None,
            None,
            Some(&observer),
        )
    }
    pub fn close_with_physical_observer_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observe: impl FnOnce(&worth_store::physical_runtime::PhysicalStoreClosePlan) + Send + 'static,
    ) -> crate::ApplicationHomeCloseOutcome {
        self.close_observed(execution, observe)
    }

    pub fn physical_serving_for_test(&self) -> &ServingPhysicalRuntime {
        &self.context.serving
    }

    pub fn abort_for_test(self) {
        drop(self.relational);
        if let Ok(serving) = std::sync::Arc::try_unwrap(self.context.serving) {
            let _aborted = serving.abort();
        }
    }
}
