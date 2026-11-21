//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_cleanup_court_doors_for_test_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
        WorthQueryPrimaryGraphApplicationRuntime<Schema>
    {
        #[doc(hidden)]
        pub fn retained_accepted_for_cleanup_test_for_test(&self, correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity) -> Option<
            std::sync::Arc<
                crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
            >,
        >{
            self.inner
                .retained_accepted_for_cleanup_test_for_test(correlation)
        }
        #[doc(hidden)]
        pub fn stale_publication_claim_for_cleanup_test_for_test(
            &self,
            accepted: &std::sync::Arc<
                crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
            >,
        ) -> (
            crate::domain_computation::application_aftermath::WorthQueryInboundPublicationClaim,
            bool,
        ) {
            self.inner
                .stale_publication_claim_for_cleanup_test_for_test(accepted)
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
        pub fn progress_retained_inbound_occurrence_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            correlation_token: [u8; 32],
            request: &WorthQueryRequestScope,
        ) -> Result<Posture, Denial> {
            self.inner.progress_retained_inbound_occurrence_for_test(
                execution,
                phase,
                correlation_token,
                request,
            )
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
        Schema: ApplicationSchema,
    {
        #[doc(hidden)]
        pub fn resolve_installed_transport_completion_binding_for_test(
            &self,
            owner: &WorthQueryCommittedDispatchOutboxObservation,
        ) -> Result<
            WorthQueryInstalledTransportCompletionBinding,
            WorthQueryTransportCompletionBindingDenial,
        > {
            self.inner
                .resolve_installed_transport_completion_binding_for_test(owner)
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
        pub fn supply_private_membership_deletion_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.supply_private_membership_deletion_for_test()
        }
        #[doc(hidden)]
        pub fn replay_next_coupled_continuation_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.replay_next_coupled_continuation_for_test()
        }
        #[doc(hidden)]
        pub fn destroy_another_writers_fresh_record_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.destroy_another_writers_fresh_record_for_test()
        }
        #[doc(hidden)]
        pub fn destroy_the_authored_prefixs_fresh_record_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner
                .destroy_the_authored_prefixs_fresh_record_for_test()
        }
        #[doc(hidden)]
        pub fn continue_next_coupled_route_with_its_effect_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner
                .continue_next_coupled_route_with_its_effect_for_test()
        }
        #[doc(hidden)]
        pub fn continue_next_coupled_route_unchanged_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.continue_next_coupled_route_unchanged_for_test()
        }
        #[doc(hidden)]
        pub fn continue_next_coupled_route_with_a_changed_patch_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner
                .continue_next_coupled_route_with_a_changed_patch_for_test()
        }
        #[doc(hidden)]
        pub fn withhold_next_coupled_output_witness_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner.withhold_next_coupled_output_witness_for_test()
        }
        #[doc(hidden)]
        pub fn break_next_coupled_assembly_without_a_subject_for_test(
            &self,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
            self.inner
                .break_next_coupled_assembly_without_a_subject_for_test()
        }
        #[doc(hidden)]
        pub fn corrupt_next_coupled_participant_for_test(
            &self,
            birth: bool,
        ) -> WorthQueryCoupledParticipantCorruptionForTest {
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
        pub fn has_retained_installed_transport_completion_for_test(
            &self,
            correlation: &ExternalEffectCorrelationIdentity,
        ) -> bool {
            self.inner
                .has_retained_installed_transport_completion_for_test(correlation)
        }
        #[doc(hidden)]
        pub fn record_installed_transport_completion_for_test(
            &self,
            evidence: InstalledTransportCompletion,
            in_flight: OutstandingDispatchInFlightLease,
        ) -> Result<(), InstalledTransportCustodyDenial> {
            self.inner
                .record_installed_transport_completion_for_test(evidence, in_flight)
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
        pub fn resume_installed_transport_completion_for_test(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            correlation: &ExternalEffectCorrelationIdentity,
            request: &WorthQueryRequestScope,
        ) -> InstalledTransportResumeOutcome {
            self.inner.resume_installed_transport_completion_for_test(
                execution,
                phase,
                correlation,
                request,
            )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_provider_history_retirement_doors {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn history_retained_for_test(
            &self,
            request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        ) -> (usize, usize, usize, usize) {
            let world = self
                .inner
                .runtime
                .product_runtime
                .owner_ref()
                .inspection_port();
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
            self.inner
                .runtime
                .primary_provider
                .completed_evidence_retained()
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
        pub fn consume_next_recovered_program_source_delivery_for_test(
            &self,
        ) -> Result<
            WorthQueryRecoveredProgramSourceDeliveryForTest<'_>,
            WorthQueryRecoveredProgramSourceDeliveryDenialForTest,
        > {
            self.inner
                .consume_next_recovered_program_source_delivery_for_test()
        }
    }
}
