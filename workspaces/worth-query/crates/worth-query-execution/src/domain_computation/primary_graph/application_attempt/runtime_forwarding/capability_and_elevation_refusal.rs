//! Thin forwarding shims for `application_attempt` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_capability_revocation {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementDenialCommitExt as _;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryCapabilityRevocationProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn compare_and_commit_capability_revocation<Operation, Input, Scope>(
            &self,
            program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationCommitOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_capability_revocation::<Operation, Input, Scope>(
                    program,
                    idempotency,
                )
        }
        pub(crate) fn compare_and_commit_capability_revocation_for_program<
            Operation,
            Input,
            Scope,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationCommitOutcome
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_capability_revocation_for_program::<Operation, Input, Scope>(
                    execution,
                    phase,
                    program,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_delegation_activation {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementDenialCommitExt as _;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryDelegationActivationProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn compare_and_commit_capability_delegation<Operation, Input, Scope>(
            &self,
            program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationCommitOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_capability_delegation::<Operation, Input, Scope>(
                    program,
                    idempotency,
                )
        }
        pub(crate) fn compare_and_commit_capability_delegation_for_program<
            Operation,
            Input,
            Scope,
        >(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationCommitOutcome
        where
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_capability_delegation_for_program::<Operation, Input, Scope>(
                    execution,
                    phase,
                    program,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_elevation_refusal {
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::*;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_definition_program::*;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_instance_program::*;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_proposal_program::*;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::*;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn refuse_elevation_request<Operation, Input, Scope>(
            &self,
            candidate: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
            denial: WorthQueryAdvancementDenial,
        ) -> WorthQueryElevationRequestOutcome {
            self.inner
                .refuse_elevation_request::<Operation, Input, Scope>(candidate, denial)
        }
        pub(crate) fn refuse_elevation_approval<Operation, Input, Scope>(
            &self,
            candidate: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
            denial: WorthQueryAdvancementDenial,
        ) -> WorthQueryElevationApprovalOutcome {
            self.inner
                .refuse_elevation_approval::<Operation, Input, Scope>(candidate, denial)
        }
        pub(crate) fn refuse_elevation_close<Operation, Input, Scope>(
            &self,
            candidate: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
            denial: WorthQueryAdvancementDenial,
        ) -> WorthQueryElevationCloseOutcome {
            self.inner
                .refuse_elevation_close::<Operation, Input, Scope>(candidate, denial)
        }
        pub(crate) fn refuse_mandatory_review<Operation, Input, Scope>(
            &self,
            candidate: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
            denial: WorthQueryAdvancementDenial,
        ) -> WorthQueryMandatoryReviewOutcome {
            self.inner
                .refuse_mandatory_review::<Operation, Input, Scope>(candidate, denial)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_entry {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEffectProgram;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub(crate) fn compare_and_commit_conditional_operation<Operation, Input, Scope>(
            &self,
            execution: &worth_execution::ExecutionRequest<'_, '_>,
            phase: &WorthQueryAdvancementPhase<'_>,
            program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
            idempotency: WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationCommitOutcome
        where
            Operation: 'static,
            Input: Clone + Send + Sync + 'static,
        {
            self.inner
                .compare_and_commit_conditional_operation::<Operation, Input, Scope>(
                    execution,
                    phase,
                    program,
                    idempotency,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_external_dispatch {
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::*;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use std::sync::Arc;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport;
    use crate::domain_computation::application_aftermath::WorthQueryPerformedExternalRedispatch;
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryEffectAuthority;
    use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
    where
        Schema: ApplicationSchema,
    {
        pub fn install_external_effect_transport(
            &self,
            transport: Arc<dyn WorthQueryExternalEffectTransport>,
        ) -> Result<(), WorthQueryExternalTransportInstallationDenial> {
            self.inner.install_external_effect_transport(transport)
        }
        pub fn has_external_effect_transport(&self) -> bool {
            self.inner.has_external_effect_transport()
        }
        pub fn redispatch_admitted_external_effect<Operation, Input, Scope>(
            &self,
            phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
            handle: &WorthQueryRecoveryHandle,
            authority: &WorthQueryRecoveryEffectAuthority,
            admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        ) -> Result<WorthQueryPerformedExternalRedispatch, WorthQueryExternalRedispatchDenial>
        {
            self.inner
                .redispatch_admitted_external_effect::<Operation, Input, Scope>(
                    phase, handle, authority, admission,
                )
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_attempt_provider_execution_phase_prepared_entry_custody {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::carrier::CommitCarrier;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::ReadmittedCommitProgram;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[inline]
        pub(crate) fn validate_commit_program<Program: CommitCarrier<Schema>>(
            &self,
            program: Program,
        ) -> Result<ReadmittedCommitProgram<'_, Schema, Program>, Program::Refusal> {
            self.inner.validate_commit_program::<Program>(program)
        }
    }
}
