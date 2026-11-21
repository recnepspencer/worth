//! Thin forwarding shims for `preparation_readmission` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_product_publication_preparation_readmission {
    use worth_query_execution_query::facade::domain_computation::product_publication::WorthQueryProductPreparationPending;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;
    use worth_runtime_world::facade::RuntimeWorldRecoveryDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn readmit_product_preparation_pending(&self, handle: &ProductUnpublishedRecoveryHandle) -> Result<WorthQueryProductPreparationPending, RuntimeWorldRecoveryDenial> {
            self.inner.readmit_product_preparation_pending(handle)
        }
    }
}
