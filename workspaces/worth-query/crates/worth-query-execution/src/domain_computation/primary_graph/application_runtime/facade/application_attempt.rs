//! The runtime's consumer methods over `domain_computation::primary_graph::application_attempt`; each only calls its subsystem.
use crate::domain_computation::application_aftermath::ApplicationCommitCausality;
use crate::domain_computation::application_aftermath::WorthQueryAdmittedIdempotencyRead;
use crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport;
use crate::domain_computation::application_aftermath::WorthQueryPerformedExternalRedispatch;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryEffectAuthority;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEffectProgram;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyResolutionDenial;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApprovedElevation;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryCapabilityRevocationProgram;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryDelegationActivationProgram;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryElevationApprovalOutcome;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryElevationCloseOutcome;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryExternalRedispatchDenial;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryExternalTransportInstallationDenial;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryMandatoryReview;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryMandatoryReviewOutcome;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryRequestedElevation;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationReadAttempt, WorthQueryApplicationRetainedCommitOutcome,
    WorthQueryElevationApprovalProgram, WorthQueryElevationCloseProgram,
    WorthQueryElevationRequestOutcome, WorthQueryElevationRequestProgram,
    WorthQueryMandatoryReviewProgram, WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionSnapshot;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use std::sync::Arc;
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn resolve_admitted_elevation_approval_replay<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryElevationApprovalOutcome>,
        (
            WorthQueryApplicationIdempotencyResolutionDenial,
            WorthQueryRequestedElevation,
        ),
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::ElevationApprovalReplayOps<Schema>>::resolve_admitted_elevation_approval_replay(self, phase, admission, idempotency)
    }

    pub fn resolve_admitted_elevation_close_replay<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryElevationCloseOutcome>,
        (
            WorthQueryApplicationIdempotencyResolutionDenial,
            WorthQueryApprovedElevation,
        ),
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::ElevationCloseReplayOps<Schema>>::resolve_admitted_elevation_close_replay(self, phase, admission, idempotency)
    }

    pub fn resolve_admitted_guarded_workflow_operation_custody<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
        transition_identity: &[u8; 32],
        completed: Option<&WorthQueryApplicationCommitReceipt>,
    ) -> Result<
        WorthQueryGuardedWorkflowOperationCustody,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::IdempotencyResolutionOps<Schema>>::resolve_admitted_guarded_workflow_operation_custody(self, execution, admission, binding, transition_identity, completed)
    }

    pub fn resolve_admitted_application_idempotency<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::IdempotencyResolutionOps<Schema>>::resolve_admitted_application_idempotency(self, phase, admission, binding)
    }

    pub fn resolve_admitted_mandatory_review_replay<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryMandatoryReviewOutcome>,
        (
            WorthQueryApplicationIdempotencyResolutionDenial,
            WorthQueryMandatoryReview,
        ),
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::MandatoryReviewReplayOps<Schema>>::resolve_admitted_mandatory_review_replay(self, phase, admission, idempotency)
    }

    pub fn compare_and_commit_capability_revocation<Operation, Input, Scope>(
        &self,
        program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::CapabilityRevocationOps<Schema>>::compare_and_commit_capability_revocation(self, program, idempotency)
    }

    pub fn compare_and_commit_capability_delegation<Operation, Input, Scope>(
        &self,
        program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::DelegationActivationOps<Schema>>::compare_and_commit_capability_delegation(self, program, idempotency)
    }

    pub fn compare_and_commit_elevation_approval<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::ElevationApprovalOps<Schema>>::compare_and_commit_elevation_approval(self, program, idempotency)
    }

    pub fn compare_and_commit_elevation_close<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationCloseOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::ElevationCloseOps<
            Schema,
        >>::compare_and_commit_elevation_close(self, program, idempotency)
    }

    pub fn compare_and_commit_elevation_request<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::ElevationRequestOps<Schema>>::compare_and_commit_elevation_request(self, program, idempotency)
    }

    pub fn compare_and_commit_mandatory_review<Operation, Input, Scope>(
        &self,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryMandatoryReviewOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::MandatoryReviewOps<Schema>>::compare_and_commit_mandatory_review(self, program, idempotency)
    }

    pub fn compare_and_commit_application<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::ApplicationCommitOps<Schema>>::compare_and_commit_application(self, program, idempotency, allocation_policy)
    }

    #[doc(hidden)]
    pub fn compare_and_commit_application_in_advancement<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::ApplicationCommitOps<Schema>>::compare_and_commit_application_in_advancement(self, phase, program, idempotency, allocation_policy)
    }

    pub fn compare_and_commit_application_for_required_output_source<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        presented: &WorthQueryPresentedProgram<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        causality: ApplicationCommitCausality,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::OutputSourceOps<
            Schema,
        >>::compare_and_commit_application_for_required_output_source(
            self,
            execution,
            phase,
            presented,
            program,
            idempotency,
            causality,
            allocation_policy,
        )
    }

    pub fn compare_and_commit_application_for_program_output_producer<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        selected_program: Option<(
            &worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
            &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        )>,
        history: Option<HistoryProducerInvocation<'_>>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::OutputSourceOps<
            Schema,
        >>::compare_and_commit_application_for_program_output_producer(
            self,
            execution,
            phase,
            program,
            idempotency,
            selected_program,
            history,
        )
    }

    pub fn compare_and_commit_application_for_program_action<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        presented: &WorthQueryPresentedProgram,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        causality: ApplicationCommitCausality,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::OutputSourceOps<
            Schema,
        >>::compare_and_commit_application_for_program_action(
            self,
            execution,
            phase,
            presented,
            program,
            idempotency,
            causality,
            allocation_policy,
        )
    }

    pub fn install_external_effect_transport(
        &self,
        transport: Arc<dyn WorthQueryExternalEffectTransport>,
    ) -> Result<(), WorthQueryExternalTransportInstallationDenial> {
        <Self as crate::domain_computation::primary_graph::application_attempt::ExternalDispatchOps<Schema>>::install_external_effect_transport(self, transport)
    }

    pub fn has_external_effect_transport(&self) -> bool {
        <Self as crate::domain_computation::primary_graph::application_attempt::ExternalDispatchOps<Schema>>::has_external_effect_transport(self)
    }

    pub fn redispatch_admitted_external_effect<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,

        handle: &WorthQueryRecoveryHandle,
        authority: &WorthQueryRecoveryEffectAuthority,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryPerformedExternalRedispatch, WorthQueryExternalRedispatchDenial> {
        <Self as crate::domain_computation::primary_graph::application_attempt::ExternalDispatchOps<Schema>>::redispatch_admitted_external_effect(self, phase, handle, authority, admission)
    }

    pub fn begin_application_read_attempt<'request, 'authority, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        mut admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_attempt::ReadSetOps<
            Schema,
        >>::begin_application_read_attempt(self, execution, admission)
    }

    pub fn begin_projected_application_read_attempt<'scope, Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'scope>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'scope,
            'scope,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_attempt::ReadSetOps<
            Schema,
        >>::begin_projected_application_read_attempt(
            self,
            phase,
            admission,
            projection,
            allocation_policy,
        )
    }

    #[doc(hidden)]
    pub fn compare_and_commit_application_retained_in_advancement<Operation, Input, Scope>(
        &self,

        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationRetainedCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::application_attempt::RetainedCommitOps<
            Schema,
        >>::compare_and_commit_application_retained_in_advancement(
            self,
            phase,
            program,
            idempotency,
            allocation_policy,
        )
    }
}
