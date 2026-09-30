use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use worth_execution::ExecutionResourceLease;

use crate::{
    product_operation_contract::WorthServerStoredProductOperation, WorthServerAdmission,
    WorthServerCompletedProductOperation, WorthServerExecutedProductReadBatch,
    WorthServerOperationRegistry, WorthServerProductAdapterRegistry,
    WorthServerProductOperationInput, WorthServerProductOperationRuntime,
    WorthServerProductOperationSurfaceDenial, WorthServerProductReadBatchStop,
    WorthServerProductSessionRegistry, WorthServerQueryHandoffConfig,
};

#[derive(Clone, Debug)]
pub struct WorthServerWorthNativeProductOperationFacade {
    admission: WorthServerAdmission,
    runtime: WorthServerProductOperationRuntime,
}

impl WorthServerWorthNativeProductOperationFacade {
    pub(crate) fn new(
        admission: WorthServerAdmission,
        operation_registry: WorthServerOperationRegistry,
        product_adapter_registry: WorthServerProductAdapterRegistry,
        query_handoff_config: WorthServerQueryHandoffConfig,
        product_session_registry: WorthServerProductSessionRegistry,
        retry_store: Arc<Mutex<HashMap<String, WorthServerStoredProductOperation>>>,
        counters: Arc<crate::diagnostics::WorthServerCounters>,
    ) -> Self {
        Self {
            admission,
            runtime: WorthServerProductOperationRuntime::new(
                operation_registry,
                product_adapter_registry,
                query_handoff_config,
                product_session_registry,
                retry_store,
                counters,
            ),
        }
    }

    pub fn execute(
        &self,
        input: WorthServerProductOperationInput,
    ) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial>
    {
        self.runtime
            .execute_from_worth_native(&self.admission, input)
    }

    pub fn execute_mutation(
        &self,
        command: super::WorthServerWorthNativeProductMutationCommand,
    ) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial>
    {
        self.execute(command.into_input())
    }

    pub fn execute_shared_read_batch(
        &self,
        inputs: Vec<WorthServerProductOperationInput>,
    ) -> Result<WorthServerExecutedProductReadBatch, WorthServerProductOperationSurfaceDenial> {
        self.runtime
            .execute_shared_read_batch_from_worth_native(&self.admission, inputs)
    }

    pub fn execute_shared_read_batch_with_lease(
        &self,
        inputs: Vec<WorthServerProductOperationInput>,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<WorthServerExecutedProductReadBatch, WorthServerProductReadBatchStop> {
        self.runtime
            .execute_shared_read_batch_from_worth_native_with_lease(&self.admission, inputs, lease)
    }

    pub fn resolve_durable_mutation(
        &self,
        recovery: &crate::WorthServerDurableProductMutationRecoveryHandle,
    ) -> Result<
        crate::WorthServerDurableProductMutationConclusion,
        WorthServerProductOperationSurfaceDenial,
    > {
        self.runtime
            .resolve_durable_mutation(&self.admission, recovery)
    }
}
