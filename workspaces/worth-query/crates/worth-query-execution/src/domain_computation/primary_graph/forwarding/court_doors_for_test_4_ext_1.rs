//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_provider_application_attempt_state_commit_preparation_relational_commit_evidence_seal_postcommit_currentness_coupled_deletions_walk_observation_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn coupled_commit_patches_walked_on_this_thread_for_test(&self) -> u64 {
            worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::coupled_deletions::walk_observation::walked()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_execution_contact_observation_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_relational::facade::identity::EntityId;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn coupled_producer_keys_on_this_thread_for_test(
            &self,
        ) -> Vec<(&'static str, EntityId, [u8; 32], Option<[u8; 32]>)> {
            self.inner.coupled_producer_keys_on_this_thread_for_test()
        }
        #[doc(hidden)]
        pub fn coupled_producer_origins_on_this_thread_for_test(
            &self,
        ) -> Vec<(String, EntityId, [bool; 4])> {
            self.inner
                .coupled_producer_origins_on_this_thread_for_test()
        }
        #[doc(hidden)]
        pub fn producer_contacts_on_this_thread_for_test(&self) -> u64 {
            self.inner.producer_contacts_on_this_thread_for_test()
        }
        #[doc(hidden)]
        pub fn producer_contacts_at_root_on_this_thread_for_test(&self, root: EntityId) -> u64 {
            self.inner
                .producer_contacts_at_root_on_this_thread_for_test(root)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_checkpoint_delivery_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandAdvance;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint as Checkpoint;
    use crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery as Delivery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::checkpoint_delivery::PreparedSelectedDelivery;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        #[doc(hidden)]
        #[allow(clippy::too_many_arguments)]
        pub fn deliver_output_checkpoint_with_finish_for_test(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            producer_identity: &str,
            receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
            delivery: Delivery,
            ready_backing: crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
            selected: bool,
            prepared: Option<PreparedSelectedDelivery<'_>>,
            finish: impl FnOnce(
                Checkpoint,
                Option<WorthQueryOutputDemandDenial>,
            )
                -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>,
        ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
            self.inner.deliver_output_checkpoint_with_finish_for_test(
                phase,
                producer_identity,
                receipt,
                delivery,
                ready_backing,
                selected,
                prepared,
                finish,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_caller_pass_observation_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn producer_publications_at_root_on_this_thread_for_test(
            &self,
            root: worth_relational::facade::identity::EntityId,
        ) -> u64 {
            self.inner
                .producer_publications_at_root_on_this_thread_for_test(root)
        }
        #[doc(hidden)]
        pub fn producer_recorded_publications_at_root_on_this_thread_for_test(
            &self,
            root: worth_relational::facade::identity::EntityId,
        ) -> u64 {
            self.inner
                .producer_recorded_publications_at_root_on_this_thread_for_test(root)
        }
        #[doc(hidden)]
        #[doc(hidden)]
        #[doc(hidden)]
        pub fn interleave_before_owned_reobservation_for_test(
            &self,
            action: impl FnOnce() + Send + 'static,
        ) {
            self.inner
                .interleave_before_owned_reobservation_for_test(action)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandAdvance;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema + 'static,
    {
        #[doc(hidden)]
        #[allow(clippy::too_many_arguments)]
        pub fn advance_program_output_demand_for_test<Family>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
            principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            request_scope: &WorthQueryRequestScope,
            delivery_branch: crate::basis::WorthQueryProductBranch,
            disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
        ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static,
        {
            self.inner.advance_program_output_demand_for_test::<Family>(
                execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_refresh_recovery_refresh_observation_doors {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn attempt_recovery_refresh_for_test<Family>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            retained: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            current: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            redisclosed: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
            receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        ) -> Result<(), WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static,
        {
            self.inner.attempt_recovery_refresh_for_test::<Family>(
                phase,
                retained,
                current,
                redisclosed,
                limits,
                receipt,
            )
        }
    }
}
