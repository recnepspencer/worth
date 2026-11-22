//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_derived_relation_doors {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::derived_view::WorthQueryRuleReadableCollectionLifecycleForTest;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn rule_readable_collection_lifecycle_for_test(
            &self,
        ) -> Vec<WorthQueryRuleReadableCollectionLifecycleForTest> {
            self.inner.rule_readable_collection_lifecycle_for_test()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_operation_control_doors {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::WorthQueryApplicationAttemptRegistrationPauseForTest;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::WorthQueryApplicationCandidatePreparationPauseForTest;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::WorthQueryManagedSourceRevalidationPauseForTest;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::WorthQuerySourceFactComparisonPauseForTest;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::WorthQuerySourcePublicationValidationPauseForTest;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn pause_after_managed_source_revalidation_for_test(
            &self,
            attempts: std::num::NonZeroUsize,
        ) -> WorthQueryManagedSourceRevalidationPauseForTest {
            self.inner
                .pause_after_managed_source_revalidation_for_test(attempts)
        }
        #[doc(hidden)]
        pub fn program_activation_entity_for_test(
            &self,
        ) -> Option<worth_relational::facade::identity::EntityId> {
            self.inner.program_activation_entity_for_test()
        }
        #[doc(hidden)]
        pub fn observe_application_branch_commit_contention_for_test(&self) -> worth_query_execution_query::facade::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitContentionForTest{
            self.inner
                .observe_application_branch_commit_contention_for_test()
        }
        #[doc(hidden)]
        pub fn pause_after_application_candidate_preparation_for_test(
            &self,
            attempts: std::num::NonZeroUsize,
        ) -> WorthQueryApplicationCandidatePreparationPauseForTest {
            self.inner
                .pause_after_application_candidate_preparation_for_test(attempts)
        }
        #[doc(hidden)]
        pub fn pause_after_application_attempt_registration_for_test(
            &self,
            attempts: std::num::NonZeroUsize,
        ) -> WorthQueryApplicationAttemptRegistrationPauseForTest {
            self.inner
                .pause_after_application_attempt_registration_for_test(attempts)
        }
        #[doc(hidden)]
        pub fn world_operation_control_for_test(
            &self,
        ) -> worth_runtime_world::facade::RuntimeWorldOperationControlForTest {
            self.inner.world_operation_control_for_test()
        }
        #[doc(hidden)]
        pub fn signal_operation_control_for_test(
            &self,
        ) -> Result<
            worth_signal::facade::branch::SignalOwnerOperationControlForTest,
            worth_runtime_bridge::facade::BridgeConditionalDenial,
        > {
            self.inner.signal_operation_control_for_test()
        }
        #[doc(hidden)]
        pub fn pause_product_currentness_for_test(
            &self,
        ) -> worth_runtime_world::facade::RuntimeWorldProductCurrentnessPauseForTest {
            self.inner.pause_product_currentness_for_test()
        }
        #[doc(hidden)]
        pub fn pause_product_branch_currentness_for_test(
            &self,
            branch: crate::basis::WorthQueryProductBranch,
        ) -> worth_runtime_world::facade::RuntimeWorldProductCurrentnessPauseForTest {
            self.inner.pause_product_branch_currentness_for_test(branch)
        }
        #[doc(hidden)]
        pub fn world_active_publication_attempts_for_test(&self) -> usize {
            self.inner.world_active_publication_attempts_for_test()
        }
        #[doc(hidden)]
        pub fn active_application_attempts_for_test(&self) -> usize {
            self.inner.active_application_attempts_for_test()
        }
        #[doc(hidden)]
        pub fn active_live_consumers_for_test(&self) -> usize {
            self.inner.active_live_consumers_for_test()
        }
        #[doc(hidden)]
        pub fn world_retention_snapshot_for_test(
            &self,
        ) -> worth_runtime_world::facade::RuntimeWorldRetentionSnapshot {
            self.inner.world_retention_snapshot_for_test()
        }
        #[doc(hidden)]
        pub fn world_history_snapshot_for_test(
            &self,
        ) -> worth_runtime_world::facade::RuntimeWorldHistorySnapshot {
            self.inner.world_history_snapshot_for_test()
        }
        #[doc(hidden)]
        pub fn with_owner_cleanup_capacity_exhausted_for_test<Output>(
            &self,
            work: impl FnOnce() -> Output,
        ) -> Output {
            self.inner
                .with_owner_cleanup_capacity_exhausted_for_test::<Output>(work)
        }
        #[doc(hidden)]
        pub fn indexed_product_branch_count_for_test(&self) -> usize {
            self.inner.indexed_product_branch_count_for_test()
        }
        #[doc(hidden)]
        pub fn with_program_retirement_barrier_for_test<Output>(
            &self,
            revision: &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
            work: impl FnOnce() -> Output,
        ) -> Output {
            self.inner
                .with_program_retirement_barrier_for_test::<Output>(revision, work)
        }
        #[doc(hidden)]
        pub fn with_program_fork_reservation_for_test<Output>(
            &self,
            revision: &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
            work: impl FnOnce() -> Output,
        ) -> Output {
            self.inner
                .with_program_fork_reservation_for_test::<Output>(revision, work)
        }
        #[doc(hidden)]
        pub fn installed_conditional_definition_count_for_test(&self) -> usize {
            self.inner.installed_conditional_definition_count_for_test()
        }
        #[doc(hidden)]
        pub fn installed_conditional_target_reference_count_for_test(&self) -> usize {
            self.inner
                .installed_conditional_target_reference_count_for_test()
        }
        #[doc(hidden)]
        pub fn owned_signal_active_node_count_for_test(&self) -> usize {
            self.inner.owned_signal_active_node_count_for_test()
        }
        #[doc(hidden)]
        pub fn pause_after_source_publication_validation_for_test(
            &self,
            attempts: std::num::NonZeroUsize,
        ) -> WorthQuerySourcePublicationValidationPauseForTest {
            self.inner
                .pause_after_source_publication_validation_for_test(attempts)
        }
        #[doc(hidden)]
        pub fn pause_before_source_fact_comparison_for_test(
            &self,
            attempts: std::num::NonZeroUsize,
        ) -> WorthQuerySourceFactComparisonPauseForTest {
            self.inner
                .pause_before_source_fact_comparison_for_test(attempts)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_oracle_observation_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_relational::facade::runtime::EntityReadRecord;
    use worth_relational::facade::runtime::RelationReadRecord;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::oracle_observation::WorthQueryOracleHistoryPageForTest;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn application_oracle_graph_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<
            (
                Vec<EntityReadRecord>,
                Vec<RelationReadRecord>,
                worth_relational::facade::symbols::SymbolTableSnapshot,
            ),
            RelationalExecutionDenialCause,
        > {
            self.inner.application_oracle_graph_for_test(execution)
        }
        #[doc(hidden)]
        pub fn application_oracle_head_for_test(
            &self,
        ) -> worth_foundational::facade::RecordedProductBranchReference {
            self.inner.application_oracle_head_for_test()
        }
        #[doc(hidden)]
        pub fn application_oracle_ancestry_for_test(&self) -> WorthQueryOracleHistoryPageForTest {
            self.inner.application_oracle_ancestry_for_test()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_external_dispatch_attempt_court_doors_for_test_doors {
    use crate::domain_computation::application_aftermath::external_effect::WorthQueryAdmittedExternalDispatchAttempt;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
    use worth_query_execution_lineage::facade::primary_graph::application_runtime::WorthQueryExternalDispatchAdmissionDenial;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        #[doc(hidden)]
        pub fn admit_external_dispatch_attempt_for_test(
            &self,
            committed: WorthQueryCommittedDispatchOutboxObservation,
        ) -> Result<
            WorthQueryAdmittedExternalDispatchAttempt<'static>,
            WorthQueryExternalDispatchAdmissionDenial,
        > {
            self.inner
                .admit_external_dispatch_attempt_for_test(committed)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_actor_retention_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_installation::facade::ApplicationSchema;
    impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
        #[doc(hidden)]
        pub fn run_actor_retention_pass_for_test(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        ) -> Result<
            (usize, usize),
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
        > {
            self.inner.run_actor_retention_pass_for_test(phase)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_cold_symbols_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_installation::facade::ApplicationSchema;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        #[doc(hidden)]
        pub fn place_symbol_pages_cold_for_test(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        ) -> Result<
            worth_relational::facade::runtime::RelationalColdSymbolPlacementForTest,
            crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
        > {
            self.inner.place_symbol_pages_cold_for_test(phase)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_correction_history_indexes_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_relational::facade::indexes::DerivedIndexMaintenanceDenial;
    use worth_relational::facade::indexes::DerivedIndexMaintenanceOutcome;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn release_correction_history_indexes_for_test(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        ) -> Result<usize, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial>
        {
            self.inner
                .release_correction_history_indexes_for_test(phase)
        }
        #[doc(hidden)]
        pub fn rebuild_correction_history_indexes_for_test(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            branch: crate::basis::WorthQueryProductBranch,
        ) -> Result<DerivedIndexMaintenanceOutcome, Box<DerivedIndexMaintenanceDenial>> {
            self.inner
                .rebuild_correction_history_indexes_for_test(phase, branch)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_inspection_materialization_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_installation::facade::ApplicationSchema;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn cancel_after_next_inspection_materialization_for_test(
            &self,
            source: worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource,
        ) -> bool {
            self.inner
                .cancel_after_next_inspection_materialization_for_test(source)
        }
    }
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        #[doc(hidden)]
        pub fn configure_conditional_history_for_test(&self, enabled: bool, limit: usize) -> bool {
            self.inner
                .configure_conditional_history_for_test(enabled, limit)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_managed_cleanup_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::provider::managed_cleanup_control::WorthQueryManagedCleanupRegistrationDenial;
    use crate::domain_computation::primary_graph::provider::managed_cleanup_control::WorthQueryManagedCleanupScheduleForTest;
    use worth_query_installation::facade::ApplicationSchema;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn schedule_next_managed_cleanup_on_worker_for_test(
            &self,
        ) -> Result<
            WorthQueryManagedCleanupScheduleForTest,
            WorthQueryManagedCleanupRegistrationDenial,
        > {
            self.inner
                .schedule_next_managed_cleanup_on_worker_for_test()
        }
        pub fn schedule_managed_cleanups_on_worker_for_test(
            &self,
            count: std::num::NonZeroUsize,
        ) -> Result<
            WorthQueryManagedCleanupScheduleForTest,
            WorthQueryManagedCleanupRegistrationDenial,
        > {
            self.inner
                .schedule_managed_cleanups_on_worker_for_test(count)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_native_field_write_doors {
    use crate::basis::WorthQueryProductBranch;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        #[doc(hidden)]
        #[allow(clippy::too_many_arguments)]
        pub fn publish_native_field_write_for_test<Entity, Aspect, Field, Value>(
            &self,
            branch: WorthQueryProductBranch,
            record: RelationalBridgeRecordIdentityParts,
            field: Field,
            value: Value,
            request: &WorthQueryRequestScope,
        ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryAdvancementDenial>
        where
            Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
                Entity = Entity,
                Aspect = Aspect,
                Value = Value,
            >,
        {
            self.inner
                .publish_native_field_write_for_test::<Entity, Aspect, Field, Value>(
                    branch, record, field, value, request,
                )
        }
    }
}
