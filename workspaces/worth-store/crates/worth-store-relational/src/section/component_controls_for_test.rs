//! Receiver-owned controls used only by the closed native component courts.
use super::{PreparedRelationalSection, RelationalSectionCheckpointDenial, RelationalStoreSection};
use std::sync::Arc;
use worth_relational::facade::{
    durability::RelationalNativeCheckpoint,
    runtime::{
        RelationalRuntime, RelationalRuntimeConstructionDenial, RelationalRuntimeDefinition,
    },
};
use worth_store::physical_runtime::{AdmittedRecordPlacementPolicy, ServingPhysicalRuntime};

impl RelationalStoreSection {
    pub fn attach_with_read_observer_for_test(
        definition: RelationalRuntimeDefinition,
        serving: Arc<ServingPhysicalRuntime>,
        placement: AdmittedRecordPlacementPolicy,
        checkpoint_selection: Arc<dyn super::RelationalCheckpointSelectionSource>,
        observer: impl Fn(
                worth_relational::facade::publication::PatchStreamPosition,
                crate::RelationalStoreColdReadCounters,
            ) + Send
            + Sync
            + 'static,
    ) -> Result<Self, RelationalRuntimeConstructionDenial> {
        let mut source = crate::cold_source::StoreColdSource::new(serving.clone());
        source.observe_commit_reads(observer);
        Self::attach_with_source(
            definition,
            serving,
            placement,
            Arc::new(source),
            checkpoint_selection,
        )
    }

    /// Replace native cells to dispute foreign carried-member placement. This
    /// cannot mint an application owner or any recovery-stage proof.
    pub fn replace_runtime_for_test(&mut self, runtime: RelationalRuntime) {
        self.runtime = Arc::new(runtime);
    }

    pub fn prepare_captured_checkpoint_for_test(
        &self,
        checkpoint: RelationalNativeCheckpoint,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PreparedRelationalSection, RelationalSectionCheckpointDenial> {
        super::checkpoint::prepare_captured(&self.cold_source, checkpoint, execution)
    }
}
