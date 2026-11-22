//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_workflow_definition_preparation_court_doors_for_test_doors {
    use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity;
    use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledWorkflowDefinitionContract;
    use crate::domain_computation::primary_graph::PreparedWorkflowDefinitionPublication;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::workflow::definition::preparation::WorkflowDefinitionPreparationDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
    where
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn prepare_workflow_publication_for_test<Capability, Operation, Input, Scope, Spec>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec>,
            expected_predecessor: crate::domain_computation::primary_graph::WorkflowDefinitionExpectedPredecessor,
            admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        ) -> Result<
            PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
            WorkflowDefinitionPreparationDenial,
        >
        where
            Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
            Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
            Spec: ApplicationWorkflowSpec<Schema = Schema>,
        {
            self.inner
                .prepare_workflow_publication_for_test::<Capability, Operation, Input, Scope, Spec>(
                    execution,
                    contract,
                    expected_predecessor,
                    admission,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_product_operation_shared_query_basis_court_doors_for_test_doors {
    use crate::basis::WorthQueryProductObservationLease;
    use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease;
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    use worth_query_execution_lineage::facade::primary_graph::product_operation::SelectedQueryBasisRetentionStop;
    use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::shared_query_basis::SharedSelectedProductOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        #[doc(hidden)]
        pub fn prepare_shared_query_basis_for_test(
            self,
        ) -> Result<SharedSelectedProductOperation<'runtime, Schema>, (Self, CompanionPreflightStop)>
        {
            let application = self.application;
            self.inner
                .prepare_shared_query_basis_for_test()
                .map_err(|(inner, stop)| (Self::from_inner(application, inner), stop))
        }
    }
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn retain_selected_query_basis_admitted_for_test(
            &self,
            shared: &SharedSelectedProductOperation<'_, Schema>,
        ) -> Result<
            (
                WorthQueryProductObservationLease,
                WorthQueryApplicationBasisLease,
            ),
            SelectedQueryBasisRetentionStop,
        > {
            self.inner
                .retain_selected_query_basis_admitted_for_test(shared)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_product_operation_context_court_doors_for_test_doors {
    use crate::basis::WorthQueryProductBranchAdmissionDenial;
    use crate::basis::WorthQueryProductBranchLease;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_runtime_world::facade::ProductBranchIdentity;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn select_product_branch_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            identity: &ProductBranchIdentity,
        ) -> Result<
            WorthQuerySelectedProductOperation<'_, Schema>,
            WorthQueryProductBranchAdmissionDenial,
        > {
            self.inner
                .select_product_branch_for_test(execution, identity)
                .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
        }
        #[doc(hidden)]
        pub fn on_product_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            product: WorthQueryProductBranchLease,
        ) -> Result<
            WorthQuerySelectedProductOperation<'_, Schema>,
            WorthQueryProductBranchAdmissionDenial,
        > {
            self.inner
                .on_product_for_test(execution, product)
                .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
        }
    }
    impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        #[doc(hidden)]
        pub fn into_parts_for_test(self) -> (
            &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryProductBranchLease,
            worth_query_execution_query::facade::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
        ){
            let application = self.application;
            let (_, product, basis) = self.inner.into_parts_for_test();
            (application, product, basis)
        }
        #[doc(hidden)]
        pub fn application_basis_for_test(&self) -> &worth_query_execution_query::facade::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease{
            self.inner.application_basis_for_test()
        }
        #[doc(hidden)]
        pub fn original_settled_producer_for_test(&self) -> Option<
            &std::sync::Arc<
                worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::OriginalSettledProducerObservation,
            >,
        >{
            self.inner.original_settled_producer_for_test()
        }
    }
    impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        #[doc(hidden)]
        pub fn retain_selection_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<Self, WorthQueryProductBranchAdmissionDenial> {
            self.inner
                .retain_selection_for_test(execution)
                .map(|inner| Self::from_inner(self.application, inner))
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_product_operation_query_retained_court_doors_for_test_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::query::WorthQueryProductQueryControls;
    use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial;
    use crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        #[doc(hidden)]
        pub fn retained_query_controls_for_test(
            self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            retained: WorthQuerySelectedProductOperation<'runtime, Schema>,
            query_name: &str,
            controls: WorthQueryProductQueryControls<'runtime>,
        ) -> Result<
            (
                &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
                WorthQueryApplicationQueryControls<'runtime, Schema>,
            ),
            WorthQueryApplicationQueryAdmissionDenial,
        > {
            let application = self.application;
            self.inner
                .retained_query_controls_for_test(
                    execution,
                    retained.into_inner(),
                    query_name,
                    controls,
                )
                .map(|(_, controls)| (application, controls))
        }
    }
}
