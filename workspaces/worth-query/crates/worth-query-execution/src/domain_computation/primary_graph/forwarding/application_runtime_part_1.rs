//! Thin forwarding shims for `application_runtime` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_candidate_input_observation {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn custom_invariant_candidate_input_counters(&self) -> worth_relational::facade::runtime::RelationalCandidateInputCounters {
            self.inner.custom_invariant_candidate_input_counters()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_derived_relation {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn discard_rule_readable_collections(&self) {
            self.inner.discard_rule_readable_collections()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_execution_memory {
    use worth_execution::ExecutionResourceLease;
    use worth_execution::LeaseRequest;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::execution_memory::WorthQueryExecutionLeaseDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn request_execution_lease(&self, request: LeaseRequest) -> Result<ExecutionResourceLease<'_>, WorthQueryExecutionLeaseDenial> {
            self.inner.request_execution_lease(request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_external_dispatch_attempt {
    use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::external_dispatch_attempt::WorthQueryTerminalEffectRefusal;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn refuse_completed_external_effect(&self, record: &WorthQueryDispatchOutboxRecord) -> Result<(), WorthQueryTerminalEffectRefusal> {
            self.inner.refuse_completed_external_effect(record)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence {
    use std::sync::Arc;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
    use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceVerifier;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierInstallationDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn install_inbound_occurrence_verifier<Operation, Input>(&self, operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>, verifier: Arc<dyn WorthQueryInboundOccurrenceVerifier>) -> Result<WorthQueryInboundVerifierHandle, WorthQueryInboundVerifierInstallationDenial> {
            self.inner.install_inbound_occurrence_verifier::<Operation, Input>(operation, verifier)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_installed_package_export {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledPackageExportDenial;
    use worth_query_installation::facade::WorthQueryPortablePackageExportLimits;
    use worth_query_installation::facade::WorthQueryPortablePackageRecordSet;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn export_domain_package(&self, owner: &str, limits: WorthQueryPortablePackageExportLimits) -> Result<WorthQueryPortablePackageRecordSet, WorthQueryInstalledPackageExportDenial> {
            self.inner.export_domain_package(owner, limits)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_managed_cleanup {
    use crate::domain_computation::primary_graph::query_obligations::*;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_foundational::facade::ManagedCleanupObligationIdentity;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn managed_run_obligations(&self) -> Vec<QueryManagedRunObligation> {
            self.inner.managed_run_obligations()
        }
        pub fn discharge_managed_run(&self, phase: &WorthQueryAdvancementPhase<'_>, identity: worth_foundational::facade::ManagedRunObligationIdentity) -> Result<(), QueryObligationDenial> {
            self.inner.discharge_managed_run(phase, identity)
        }
        pub fn conditional_decision_obligations(&self) -> Vec<QueryConditionalDecision> {
            self.inner.conditional_decision_obligations()
        }
        pub fn managed_cleanup_obligations(&self) -> Vec<QueryManagedCleanupObligation> {
            self.inner.managed_cleanup_obligations()
        }
        pub fn discharge_managed_cleanup(&self, phase: &WorthQueryAdvancementPhase<'_>, identity: ManagedCleanupObligationIdentity) -> Result<(), QueryObligationDenial> {
            self.inner.discharge_managed_cleanup(phase, identity)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_mutation_partition {
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn issue_application_mutation_partition(&self) -> Option<worth_relational::facade::identity::PartitionId> {
            self.inner.issue_application_mutation_partition()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_operation_input_capture {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_declaration::facade::application_operation::ApplicationEncodedInput;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::operation_input_capture::WorthQueryOperationInputCaptureDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn capture_operation_input_in_advancement<InputBinding>(&self, phase: &WorthQueryAdvancementPhase<'_>, __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess, request: &WorthQueryRequestScope, original: &InputBinding::Value) -> Result<ApplicationEncodedInput<InputBinding>, WorthQueryOperationInputCaptureDenial>
        where
            InputBinding: ApplicationStructuredValueBinding,
            InputBinding::Value: Clone + serde::Serialize + Send + Sync, {
            self.inner.capture_operation_input_in_advancement::<InputBinding>(phase, __arg1, request, original)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_provider_session_preparation {
    use crate::domain_computation::WorthQueryPreparedProviderSession;
    use crate::domain_computation::WorthQueryProviderSessionFailure;
    use crate::domain_computation::WorthQueryProviderSessionGrant;
    use crate::domain_computation::WorthQueryProviderSessionReceiver;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn prepare_provider_session_grant<'run>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'run>, grant: WorthQueryProviderSessionGrant, receiver: impl FnOnce() -> Result<
                WorthQueryProviderSessionReceiver<'run>,
                WorthQueryProviderSessionFailure,
            >) -> Result<WorthQueryPreparedProviderSession<'run>, WorthQueryProviderSessionFailure> {
            self.inner.prepare_provider_session_grant(phase, grant, receiver)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_recovery_custody {
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial as Denial;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn maintain_recovery_claims(&self, phase: &WorthQueryAdvancementPhase<'_>) -> Result<(), Denial> {
            self.inner.maintain_recovery_claims(phase)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_storage_retention {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::storage_retention::WorthQueryStorageRetentionProgress;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::storage_retention::WorthQueryStorageRetentionDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn try_reclaim_retired_storage(&self, phase: &WorthQueryAdvancementPhase<'_>) -> Result<WorthQueryStorageRetentionProgress, WorthQueryStorageRetentionDenial> {
            self.inner.try_reclaim_retired_storage(phase)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_recovery_custody_completion {
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryTransitionDenial;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleBinding;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn settle_recorded_recovery_completion<Operation, Input, Scope>(&self, phase: &WorthQueryAdvancementPhase<'_>, handle: WorthQueryRecoveryHandle, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>) -> Result<WorthQueryRecoveryHandleBinding, WorthQueryRecoveryTransitionDenial> {
            self.inner.settle_recorded_recovery_completion::<Operation, Input, Scope>(phase, handle, admission)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_recovery_custody_readmission {
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial as Denial;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn mint_admitted_recovery_handle<Operation, Input, Scope, InputBinding>(&self, phase: &WorthQueryAdvancementPhase<'_>, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>, binding: WorthQueryApplicationIdempotencyBinding, input: &worth_query_declaration::facade::application_operation::ApplicationEncodedInput<
                InputBinding,
            >) -> Result<WorthQueryRecoveryHandle, Denial>
        where
            InputBinding:
                worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                    Value = Input,
                >,
            Input: Clone + serde::Serialize + Send + Sync + 'static, {
            self.inner.mint_admitted_recovery_handle::<Operation, Input, Scope, InputBinding>(phase, admission, binding, input)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_recovery_custody_settlement {
    use worth_execution::ExecutionRequest;
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial as Denial;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn settle_recovery_custody(&self, execution: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
            self.inner.settle_recovery_custody(execution)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_recovery_custody_source {
    use worth_runtime_world::facade::WorldProgramActivationHistory;
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial as Denial;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn admit_recovery_claim_sources(&self, originals: &WorldProgramActivationHistory, execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<(), Denial> {
            self.inner.admit_recovery_claim_sources(originals, execution)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_cleanup {
    use std::num::NonZeroUsize;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundAdmissionDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use crate::domain_computation::application_aftermath::WorthQueryInboundCleanupReport;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn cleanup_completed_inbound_occurrences(&self, handle: &WorthQueryInboundVerifierHandle, maximum_work: NonZeroUsize) -> Result<WorthQueryInboundCleanupReport, Denial> {
            self.inner.cleanup_completed_inbound_occurrences(handle, maximum_work)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_cost {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::cost::WorthQueryInboundCostObservation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn observe_inbound_cost(&self, handle: &WorthQueryInboundVerifierHandle) -> Option<WorthQueryInboundCostObservation> {
            self.inner.observe_inbound_cost(handle)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_maintenance {
    use std::num::NonZeroUsize;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundAdmissionDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::maintenance::WorthQueryInboundMaintenanceReport;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn maintain_inbound_occurrences(&self, handle: &WorthQueryInboundVerifierHandle, maximum_work: NonZeroUsize, request: &WorthQueryRequestScope) -> Result<WorthQueryInboundMaintenanceReport, Denial> {
            self.inner.maintain_inbound_occurrences(handle, maximum_work, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_observe {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::observe::WorthQueryInboundTerminalObservation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn observe_inbound_terminal(&self, handle: &WorthQueryInboundVerifierHandle, correlation_token: [u8; 32]) -> Option<WorthQueryInboundTerminalObservation> {
            self.inner.observe_inbound_terminal(handle, correlation_token)
        }
        pub fn observe_inbound_recovery_denial(&self, handle: &WorthQueryInboundVerifierHandle, correlation_token: [u8; 32]) -> Option<std::sync::Arc<worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::WorthQueryRecordedCompletionDenial>> {
            self.inner.observe_inbound_recovery_denial(handle, correlation_token)
        }
    }
}
