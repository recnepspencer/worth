use super::{
    WorthQueryRuntimeError, WorthQueryRuntimeFacadeFamily, WorthQuerySharedReadContext,
    WorthQuerySharedReadCounters, WorthQueryWorkspace,
};

impl WorthQueryWorkspace {
    pub fn shared_read_context(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQuerySharedReadContext, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(WorthQueryRuntimeFacadeFamily::SharedRead)?;
        self.runtime.mint_shared_read_context(execution)
    }

    pub fn shared_read_counters(&self) -> WorthQuerySharedReadCounters {
        self.runtime.shared_read_counters()
    }

    pub fn record_shared_read_hot_path_lock_for_certification(&self) {
        self.runtime
            .record_shared_read_hot_path_lock_for_certification();
    }
}
