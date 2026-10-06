use worth_execution::ExecutionResourceLease;

use super::WorthServerProductOperationRuntime;
use crate::product_adapter::execution_pipeline::{
    execute_shared_read_batch_from_worth_native,
    execute_shared_read_batch_with_lease_from_worth_native,
};
use crate::{
    WorthServerAdmission, WorthServerExecutedProductReadBatch, WorthServerProductOperationInput,
    WorthServerProductOperationSurfaceDenial, WorthServerProductReadBatchStop,
};

impl WorthServerProductOperationRuntime {
    pub fn execute_shared_read_batch_from_worth_native(
        &self,
        admission: &WorthServerAdmission,
        inputs: Vec<WorthServerProductOperationInput>,
    ) -> Result<WorthServerExecutedProductReadBatch, WorthServerProductOperationSurfaceDenial> {
        let executed = execute_shared_read_batch_from_worth_native(
            &self.operation_registry,
            &self.adapter_registry,
            &self.query_handoff_config,
            admission,
            inputs,
        )?;
        self.record_artifacts(&executed);
        Ok(executed)
    }

    pub fn execute_shared_read_batch_from_worth_native_with_lease(
        &self,
        admission: &WorthServerAdmission,
        inputs: Vec<WorthServerProductOperationInput>,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<WorthServerExecutedProductReadBatch, WorthServerProductReadBatchStop> {
        let executed = execute_shared_read_batch_with_lease_from_worth_native(
            &self.operation_registry,
            &self.adapter_registry,
            &self.query_handoff_config,
            admission,
            inputs,
            lease,
        )?;
        self.record_artifacts(&executed);
        Ok(executed)
    }

    fn record_artifacts(&self, executed: &WorthServerExecutedProductReadBatch) {
        for operation in executed.operations() {
            if let Some(artifact) = operation.result_artifact() {
                self.counters
                    .record_product_result_artifact(artifact.body().byte_len());
            }
        }
    }
}
