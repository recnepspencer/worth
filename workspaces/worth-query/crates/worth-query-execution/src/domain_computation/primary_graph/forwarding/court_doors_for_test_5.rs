//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_admission_court_doors_for_test_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        #[doc(hidden)]
        pub fn admit_required_output_demand_for_test<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>, {
            self.inner.admit_required_output_demand_for_test::<Family>(execution, source_result, limits)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_scheduling_progression_court_doors_for_test_doors {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::WorthQuerySelectedApplicationProducer;
    use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryObservedSource;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static, {
        #[doc(hidden)]
        pub fn schedule_selected_output_producer_for_test<Query>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, selected: &WorthQuerySelectedApplicationProducer, branch: crate::basis::WorthQueryProductBranch, observed_source: &WorthQueryObservedSource<Query>, performed_source: Option<
                &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRetainedOutputDemandSource,
            >) -> Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial> {
            self.inner.schedule_selected_output_producer_for_test::<Query>(execution, phase, selected, branch, observed_source, performed_source)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_currentness_court_doors_for_test_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::output_lineage::RetainedOutputCurrentnessRead;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
        #[doc(hidden)]
        pub fn require_current_read_for_test(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, relational: &worth_relational::facade::runtime::RelationalRuntime, read: &RetainedOutputCurrentnessRead, receipt: Option<&WorthQueryApplicationCommitReceipt>) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner.require_current_read_for_test(execution, relational, read, receipt)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_registry_required_work_runtime_doors {
    use worth_query_execution_lineage::facade::primary_graph::application_output_demand::registry::join_observation::StableJoinPauseForTest;
    use worth_query_execution_lineage::facade::primary_graph::application_output_demand::registry::WorthQueryDiscoveredRetentionControlDenial;
    use worth_query_execution_lineage::facade::primary_graph::application_output_demand::registry::WorthQueryDiscoveredRetentionReservationForTest;
    use worth_relational::facade::identity::EntityId as Root;
    use worth_runtime_world::facade::CompositeCommitIdentity as Commit;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn joined_stable_roots_for_test(&self) -> Vec<Root> {
            self.inner.joined_stable_roots_for_test()
        }
        #[doc(hidden)]
        pub fn retained_source_selections_for_test(&self) -> Vec<(Root, Commit, Commit)> {
            self.inner.retained_source_selections_for_test()
        }
        #[doc(hidden)]
        pub fn pause_stable_join_for_test(&self, root: Root) -> StableJoinPauseForTest {
            self.inner.pause_stable_join_for_test(root)
        }
        #[doc(hidden)]
        pub fn registry_row_keys_for_test(&self) -> Vec<(String, Root, u64, [u8; 32])> {
            self.inner.registry_row_keys_for_test()
        }
        #[doc(hidden)]
        pub fn registry_successor_claims_for_test(&self) -> usize {
            self.inner.registry_successor_claims_for_test()
        }
        #[doc(hidden)]
        pub fn stable_ready_sources_for_test(&self) -> Vec<(Root, Commit, [u8; 32])> {
            self.inner.stable_ready_sources_for_test()
        }
        #[doc(hidden)]
        pub fn required_refresh_stops_for_test(&self) -> Vec<(Root, WorthQueryOutputDemandDenialKind)> {
            self.inner.required_refresh_stops_for_test()
        }
        #[doc(hidden)]
        pub fn has_required_row_for_test(&self, root: Root) -> bool {
            self.inner.has_required_row_for_test(root)
        }
        #[doc(hidden)]
        pub fn registry_row_count_for_test(&self) -> usize {
            self.inner.registry_row_count_for_test()
        }
        #[doc(hidden)]
        pub fn whole_registry_rows_visited_for_test(&self) -> Option<u64> {
            self.inner.whole_registry_rows_visited_for_test()
        }
        #[doc(hidden)]
        pub fn leave_one_readiness_attempt_for_test(&self) {
            self.inner.leave_one_readiness_attempt_for_test()
        }
        #[doc(hidden)]
        pub fn required_row_stops_for_root_for_test(&self, root: Root) -> Vec<(String, Option<WorthQueryOutputDemandDenialKind>)> {
            self.inner.required_row_stops_for_root_for_test(root)
        }
        #[doc(hidden)]
        pub fn queued_required_work_for_test(&self) -> usize {
            self.inner.queued_required_work_for_test()
        }
        #[doc(hidden)]
        pub fn published_required_rows_for_test(&self) -> Vec<(String, usize, bool, String)> {
            self.inner.published_required_rows_for_test()
        }
        #[doc(hidden)]
        pub fn queued_required_rows_for_root_for_test(&self, root: Root) -> Vec<String> {
            self.inner.queued_required_rows_for_root_for_test(root)
        }
        #[doc(hidden)]
        pub fn entered_required_selection_is_frozen_for_test(&self) -> Option<bool> {
            self.inner.entered_required_selection_is_frozen_for_test()
        }
        #[doc(hidden)]
        pub fn undelivered_publication_counts_for_test(&self) -> (usize, usize) {
            self.inner.undelivered_publication_counts_for_test()
        }
    }
    impl<S: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn refuse_next_discovered_retention_reservation_for_test(&self) -> Result<
            WorthQueryDiscoveredRetentionReservationForTest,
            WorthQueryDiscoveredRetentionControlDenial,
        > {
            self.inner.refuse_next_discovered_retention_reservation_for_test()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_resolution_selected_product_admitted_court_doors_for_test_doors {
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationIdentityScalarValueBinding;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledPrincipalBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial;
    use worth_query_execution_lineage::facade::primary_graph::resolution::WorthQueryPrincipalResolutionMode;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema>
    WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema, {
        #[doc(hidden)]
        pub fn resolve_authenticated_principal_admitted_for_test<
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, installed_binding: &WorthQueryInstalledPrincipalBinding<
                Schema,
                Binding,
                Mapping,
                Principal,
                PrincipalIdentity,
                PrincipalIdentityBinding,
            >, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, scope: &WorthQueryRequestScope, mode: WorthQueryPrincipalResolutionMode) -> Result<
            WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
            WorthQueryPrincipalResolutionDenial,
        >
        where
            PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
            PrincipalIdentity: 'static, {
            self.inner.resolve_authenticated_principal_admitted_for_test::<Binding, Mapping, Principal, PrincipalIdentity, PrincipalIdentityBinding>(execution, installed_binding, external, scope, mode)
        }
    }
}

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
    Schema: ApplicationSchema, {
        #[doc(hidden)]
        pub fn prepare_workflow_publication_for_test<Capability, Operation, Input, Scope, Spec>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec>, expected_predecessor: crate::domain_computation::primary_graph::WorkflowDefinitionExpectedPredecessor, admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>) -> Result<
            PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
            WorkflowDefinitionPreparationDenial,
        >
        where
            Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
            Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
            Spec: ApplicationWorkflowSpec<Schema = Schema>, {
            self.inner.prepare_workflow_publication_for_test::<Capability, Operation, Input, Scope, Spec>(execution, contract, expected_predecessor, admission)
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
        pub fn prepare_shared_query_basis_for_test(self) -> Result<SharedSelectedProductOperation<'runtime, Schema>, (Self, CompanionPreflightStop)> {
            let application = self.application;
            self.inner.prepare_shared_query_basis_for_test()
                .map_err(|(inner, stop)| (Self::from_inner(application, inner), stop))
        }
    }
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn retain_selected_query_basis_admitted_for_test(&self, shared: &SharedSelectedProductOperation<'_, Schema>) -> Result<
            (
                WorthQueryProductObservationLease,
                WorthQueryApplicationBasisLease,
            ),
            SelectedQueryBasisRetentionStop,
        > {
            self.inner.retain_selected_query_basis_admitted_for_test(shared)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_product_operation_context_court_doors_for_test_doors {
    use crate::basis::WorthQueryProductBranchAdmissionDenial;
    use crate::basis::WorthQueryProductBranchLease;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_runtime_world::facade::ProductBranchIdentity;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn select_product_branch_for_test(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, identity: &ProductBranchIdentity) -> Result<
            WorthQuerySelectedProductOperation<'_, Schema>,
            WorthQueryProductBranchAdmissionDenial,
        > {
            self.inner.select_product_branch_for_test(execution, identity)
                .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
        }
        #[doc(hidden)]
        pub fn on_product_for_test(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, product: WorthQueryProductBranchLease) -> Result<
            WorthQuerySelectedProductOperation<'_, Schema>,
            WorthQueryProductBranchAdmissionDenial,
        > {
            self.inner.on_product_for_test(execution, product)
                .map(|inner| WorthQuerySelectedProductOperation::from_inner(self, inner))
        }
    }
    impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        #[doc(hidden)]
        pub fn into_parts_for_test(self) -> (
            &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryProductBranchLease,
            worth_query_execution_query::facade::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
        ) {
            let application = self.application;
            let (_, product, basis) = self.inner.into_parts_for_test();
            (application, product, basis)
        }
        #[doc(hidden)]
        pub fn application_basis_for_test(&self) -> &worth_query_execution_query::facade::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease {
            self.inner.application_basis_for_test()
        }
        #[doc(hidden)]
        pub fn original_settled_producer_for_test(&self) -> Option<
            &std::sync::Arc<
                worth_query_execution_query::facade::domain_computation::primary_graph::application_output_demand::OriginalSettledProducerObservation,
            >,
        > {
            self.inner.original_settled_producer_for_test()
        }
    }
    impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
        #[doc(hidden)]
        pub fn retain_selection_for_test(&self, execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<Self, WorthQueryProductBranchAdmissionDenial> {
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
        pub fn retained_query_controls_for_test(self, execution: &worth_execution::ExecutionRequest<'_, '_>, retained: WorthQuerySelectedProductOperation<'runtime, Schema>, query_name: &str, controls: WorthQueryProductQueryControls<'runtime>) -> Result<
            (
                &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
                WorthQueryApplicationQueryControls<'runtime, Schema>,
            ),
            WorthQueryApplicationQueryAdmissionDenial,
        > {
            let application = self.application;
            self.inner
                .retained_query_controls_for_test(execution, retained.into_inner(), query_name, controls)
                .map(|(_, controls)| (application, controls))
        }
    }
}
