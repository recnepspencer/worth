//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_cleanup_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn retained_accepted_for_cleanup_test_for_test(&self, correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity) -> Option<
            std::sync::Arc<
                crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
            >,
        > {
            self.inner.retained_accepted_for_cleanup_test_for_test(correlation)
        }
        #[doc(hidden)]
        pub fn stale_publication_claim_for_cleanup_test_for_test(&self, accepted: &std::sync::Arc<
                crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
            >) -> (
            crate::domain_computation::application_aftermath::WorthQueryInboundPublicationClaim,
            bool,
        ) {
            self.inner.stale_publication_claim_for_cleanup_test_for_test(accepted)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_recover_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundAdmissionDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundReceiptPosture as Posture;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn progress_retained_inbound_occurrence_for_test(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, correlation_token: [u8; 32], request: &WorthQueryRequestScope) -> Result<Posture, Denial> {
            self.inner.progress_retained_inbound_occurrence_for_test(execution, phase, correlation_token, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_transport_binding_court_doors_for_test_doors {
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::transport_binding::*;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        #[doc(hidden)]
        pub fn resolve_installed_transport_completion_binding_for_test(&self, owner: &WorthQueryCommittedDispatchOutboxObservation) -> Result<
            WorthQueryInstalledTransportCompletionBinding,
            WorthQueryTransportCompletionBindingDenial,
        > {
            self.inner.resolve_installed_transport_completion_binding_for_test(owner)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_operation_control_coupled_participant_doors {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::operation_control::coupled_participant::WorthQueryCoupledParticipantCorruptionForTest;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S> WorthQueryPrimaryGraphApplicationRuntime<S> {
        #[doc(hidden)]
        pub fn supply_private_membership_deletion_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.supply_private_membership_deletion_for_test()
        }
        #[doc(hidden)]
        pub fn replay_next_coupled_continuation_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.replay_next_coupled_continuation_for_test()
        }
        #[doc(hidden)]
        pub fn destroy_another_writers_fresh_record_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.destroy_another_writers_fresh_record_for_test()
        }
        #[doc(hidden)]
        pub fn destroy_the_authored_prefixs_fresh_record_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.destroy_the_authored_prefixs_fresh_record_for_test()
        }
        #[doc(hidden)]
        pub fn continue_next_coupled_route_with_its_effect_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.continue_next_coupled_route_with_its_effect_for_test()
        }
        #[doc(hidden)]
        pub fn continue_next_coupled_route_unchanged_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.continue_next_coupled_route_unchanged_for_test()
        }
        #[doc(hidden)]
        pub fn continue_next_coupled_route_with_a_changed_patch_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.continue_next_coupled_route_with_a_changed_patch_for_test()
        }
        #[doc(hidden)]
        pub fn withhold_next_coupled_output_witness_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.withhold_next_coupled_output_witness_for_test()
        }
        #[doc(hidden)]
        pub fn break_next_coupled_assembly_without_a_subject_for_test(&self) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.break_next_coupled_assembly_without_a_subject_for_test()
        }
        #[doc(hidden)]
        pub fn corrupt_next_coupled_participant_for_test(&self, birth: bool) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.corrupt_next_coupled_participant_for_test(birth)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_publication_custody_court_doors_for_test_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_publication::installed_transport::InstalledTransportCompletion;
    use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
    use crate::domain_computation::primary_graph::OutstandingDispatchInFlightLease;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_publication::custody::InstalledTransportCustodyDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn has_retained_installed_transport_completion_for_test(&self, correlation: &ExternalEffectCorrelationIdentity) -> bool {
            self.inner.has_retained_installed_transport_completion_for_test(correlation)
        }
        #[doc(hidden)]
        pub fn record_installed_transport_completion_for_test(&self, evidence: InstalledTransportCompletion, in_flight: OutstandingDispatchInFlightLease) -> Result<(), InstalledTransportCustodyDenial> {
            self.inner.record_installed_transport_completion_for_test(evidence, in_flight)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_publication_custody_resume_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::application_runtime::inbound_publication::recovered::RecordedCompletionContinuationAdvanceExt as _;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_publication::custody::InstalledTransportResumeOutcome;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn resume_installed_transport_completion_for_test(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, correlation: &ExternalEffectCorrelationIdentity, request: &WorthQueryRequestScope) -> InstalledTransportResumeOutcome {
            self.inner.resume_installed_transport_completion_for_test(execution, phase, correlation, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_provider_history_retirement_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn history_retained_for_test(&self, request_execution: &worth_execution::ExecutionRequest<'_, '_>) -> (usize, usize, usize, usize) {
            let world = self.inner.runtime.product_runtime.owner.inspection_port();
            let history = world
                .history_snapshot()
                .expect("an installed product World is available");
            let retention = world
                .retention_snapshot()
                .expect("an installed product World is available");
            (
                history.installed_commits(),
                history.metadata().total_occupancy(),
                retention.unique_pins(),
                self.inner
                    .runtime
                    .primary_provider
                    .graph
                    .with_runtime(request_execution, |runtime| {
                        runtime.retired_branch_root_count()
                    }),
            )
        }
        #[doc(hidden)]
        pub fn completed_evidence_retained_for_test(&self) -> usize {
            self.inner.runtime.primary_provider.completed_evidence_retained()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_provider_recovered_program_source_delivery_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::provider::recovered_program_source_delivery::WorthQueryRecoveredProgramSourceDeliveryDenialForTest;
    use worth_query_execution_query::facade::domain_computation::primary_graph::provider::recovered_program_source_delivery::WorthQueryRecoveredProgramSourceDeliveryForTest;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn consume_next_recovered_program_source_delivery_for_test(&self) -> Result<
            WorthQueryRecoveredProgramSourceDeliveryForTest<'_>,
            WorthQueryRecoveredProgramSourceDeliveryDenialForTest,
        > {
            self.inner.consume_next_recovered_program_source_delivery_for_test()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_provider_application_attempt_state_commit_preparation_relational_commit_evidence_seal_postcommit_currentness_coupled_deletions_walk_observation_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn coupled_commit_patches_walked_on_this_thread_for_test(&self) -> u64 {
            worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::coupled_deletions::walk_observation::walked()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_execution_contact_observation_doors {
    use worth_relational::facade::identity::EntityId;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn coupled_producer_keys_on_this_thread_for_test(&self) -> Vec<(&'static str, EntityId, [u8; 32], Option<[u8; 32]>)> {
            self.inner.coupled_producer_keys_on_this_thread_for_test()
        }
        #[doc(hidden)]
        pub fn coupled_producer_origins_on_this_thread_for_test(&self) -> Vec<(String, EntityId, [bool; 4])> {
            self.inner.coupled_producer_origins_on_this_thread_for_test()
        }
        #[doc(hidden)]
        pub fn producer_contacts_on_this_thread_for_test(&self) -> u64 {
            self.inner.producer_contacts_on_this_thread_for_test()
        }
        #[doc(hidden)]
        pub fn producer_contacts_at_root_on_this_thread_for_test(&self, root: EntityId) -> u64 {
            self.inner.producer_contacts_at_root_on_this_thread_for_test(root)
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
    Schema: ApplicationSchema + 'static, {
        #[doc(hidden)]
        #[allow(clippy::too_many_arguments)]
        pub fn deliver_output_checkpoint_with_finish_for_test(&self, phase: &WorthQueryAdvancementPhase<'_>, producer_identity: &str, receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt, delivery: Delivery, ready_backing: crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking, selected: bool, prepared: Option<PreparedSelectedDelivery<'_>>, finish: impl FnOnce(
                Checkpoint,
                Option<WorthQueryOutputDemandDenial>,
            )
                -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
            self.inner.deliver_output_checkpoint_with_finish_for_test(phase, producer_identity, receipt, delivery, ready_backing, selected, prepared, finish)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_caller_pass_observation_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn producer_publications_at_root_on_this_thread_for_test(&self, root: worth_relational::facade::identity::EntityId) -> u64 {
            self.inner.producer_publications_at_root_on_this_thread_for_test(root)
        }
        #[doc(hidden)]
        pub fn producer_recorded_publications_at_root_on_this_thread_for_test(&self, root: worth_relational::facade::identity::EntityId) -> u64 {
            self.inner.producer_recorded_publications_at_root_on_this_thread_for_test(root)
        }
        #[doc(hidden)]
        #[doc(hidden)]
        #[doc(hidden)]
        pub fn interleave_before_owned_reobservation_for_test(&self, action: impl FnOnce() + Send + 'static) {
            self.inner.interleave_before_owned_reobservation_for_test(action)
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
    Schema: ApplicationSchema + 'static, {
        #[doc(hidden)]
        #[allow(clippy::too_many_arguments)]
        pub fn advance_program_output_demand_for_test<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, request_scope: &WorthQueryRequestScope, delivery_branch: crate::basis::WorthQueryProductBranch, disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static, {
            self.inner.advance_program_output_demand_for_test::<Family>(execution, phase, demand, principal, request_scope, delivery_branch, disclosure)
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
        pub fn attempt_recovery_refresh_for_test<Family>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, retained: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, current: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, redisclosed: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>, limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits, receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt) -> Result<(), WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static, {
            self.inner.attempt_recovery_refresh_for_test::<Family>(phase, retained, current, redisclosed, limits, receipt)
        }
    }
}
