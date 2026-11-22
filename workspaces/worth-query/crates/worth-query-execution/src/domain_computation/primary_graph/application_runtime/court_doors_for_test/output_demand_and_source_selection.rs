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
        Schema: ApplicationSchema + 'static,
    {
        #[doc(hidden)]
        pub fn admit_required_output_demand_for_test<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
        {
            self.inner.admit_required_output_demand_for_test::<Family>(
                execution,
                source_result,
                limits,
            )
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
        Schema: worth_query_installation::facade::ApplicationSchema + 'static,
    {
        #[doc(hidden)]
        pub fn schedule_selected_output_producer_for_test<Query>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            selected: &WorthQuerySelectedApplicationProducer,
            branch: crate::basis::WorthQueryProductBranch,
            observed_source: &WorthQueryObservedSource<Query>,
            performed_source: Option<
                &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRetainedOutputDemandSource,
            >,
        ) -> Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial> {
            self.inner
                .schedule_selected_output_producer_for_test::<Query>(
                    execution,
                    phase,
                    selected,
                    branch,
                    observed_source,
                    performed_source,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_output_demand_currentness_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::output_lineage::RetainedOutputCurrentnessRead;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
    use worth_query_installation::facade::ApplicationSchema;
    impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
        #[doc(hidden)]
        pub fn require_current_read_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            relational: &worth_relational::facade::runtime::RelationalRuntime,
            read: &RetainedOutputCurrentnessRead,
            receipt: Option<&WorthQueryApplicationCommitReceipt>,
        ) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner
                .require_current_read_for_test(execution, relational, read, receipt)
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
        pub fn required_refresh_stops_for_test(
            &self,
        ) -> Vec<(Root, WorthQueryOutputDemandDenialKind)> {
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
        pub fn required_row_stops_for_root_for_test(
            &self,
            root: Root,
        ) -> Vec<(String, Option<WorthQueryOutputDemandDenialKind>)> {
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
        WorthQueryPrimaryGraphApplicationRuntime<S>
    {
        pub fn refuse_next_discovered_retention_reservation_for_test(
            &self,
        ) -> Result<
            WorthQueryDiscoveredRetentionReservationForTest,
            WorthQueryDiscoveredRetentionControlDenial,
        > {
            self.inner
                .refuse_next_discovered_retention_reservation_for_test()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_resolution_selected_product_admitted_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_execution_lineage::facade::primary_graph::resolution::WorthQueryPrincipalResolutionMode;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
    use worth_query_execution_query::facade::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial;
    use worth_query_installation::facade::ApplicationIdentityScalarValueBinding;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledPrincipalBinding;
    impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema>
    where
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn resolve_authenticated_principal_admitted_for_test<
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            installed_binding: &WorthQueryInstalledPrincipalBinding<
                Schema,
                Binding,
                Mapping,
                Principal,
                PrincipalIdentity,
                PrincipalIdentityBinding,
            >,
            external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            scope: &WorthQueryRequestScope,
            mode: WorthQueryPrincipalResolutionMode,
        ) -> Result<
            WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
            WorthQueryPrincipalResolutionDenial,
        >
        where
            PrincipalIdentityBinding:
                ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
            PrincipalIdentity: 'static,
        {
            self.inner.resolve_authenticated_principal_admitted_for_test::<Binding, Mapping, Principal, PrincipalIdentity, PrincipalIdentityBinding>(execution, installed_binding, external, scope, mode)
        }
    }
}
