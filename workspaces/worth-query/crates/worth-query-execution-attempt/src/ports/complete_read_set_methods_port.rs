//! Crate-boundary internal port for `WorthQueryCompleteApplicationReadSet`
//! workflow materialization and fact retention methods consumed by
//! `worth-query-execution-query` and `worth-query-execution`. Never exported on
//! `worth-query-execution::facade` or Host.

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput, WorthQueryPreserveOutput,
};
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::application_attempt::workflow_definition_program::{
    retirement::PreparedWorkflowDefinitionRetirement, PreparedWorkflowDefinitionPublication,
    WorkflowDefinitionExpectedPredecessor,
};
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::{
    birth::RequiredWorkflowBirth, cancellation::PreparedWorkflowInstanceCancellation,
    succession::RequiredWorkflowSuccession, PreparedWorkflowInstanceStart,
    WorkflowSuccessionTarget,
};
use crate::domain_computation::primary_graph::application_attempt::workflow_proposal_program::PreparedWorkflowProposal;
use crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::{
    operation::evidence::WorkflowOperationEvidence, PreparedWorkflowAdvance,
    RequiredWorkflowApproval, WorkflowApprovalDecision, WorkflowTransitionRequestKind,
};
use crate::domain_computation::primary_graph::application_attempt::{
    PublishedWorkflowProposalRef, WorkflowStepAllowance, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationObservedFact, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::invariant_projection::BorrowedOperationProjection;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport;
use crate::domain_computation::primary_graph::workflow::definition::preparation::BoundWorkflowDefinitionContract;
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;
use crate::domain_computation::primary_graph::workflow::instance::{
    SelectedWorkflowOperation, SelectedWorkflowTransition, WorkflowInstanceProgress,
    WorkflowSuccession,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::primary_graph::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef, WorkflowCapturedInputs,
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;
use crate::ports::WorkflowVocabularyPort;

pub trait CompleteReadSetFactAppendInternalPort {
    fn append_completed_facts<Facts>(
        &mut self,
        additional: Facts,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Facts: IntoIterator<Item = WorthQueryApplicationObservedFact>,
        Facts::IntoIter: ExactSizeIterator;
}

impl<Schema, Operation, Input, Scope, Phase> CompleteReadSetFactAppendInternalPort
    for WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    fn append_completed_facts<Facts>(
        &mut self,
        additional: Facts,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Facts: IntoIterator<Item = WorthQueryApplicationObservedFact>,
        Facts::IntoIter: ExactSizeIterator,
    {
        Self::append_completed_facts(self, additional, policy)
    }
}

pub trait CompleteReadSetProjectedInternalPort<Schema, Operation> {
    fn captured_inputs_remain_equal(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<bool, WorthQueryApplicationAttemptDenial>;

    fn complete_compiled_body_projection(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        projection: BorrowedOperationProjection<Schema, Operation>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;
}

impl<Schema, Operation, Input, Scope> CompleteReadSetProjectedInternalPort<Schema, Operation>
    for WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    fn captured_inputs_remain_equal(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<bool, WorthQueryApplicationAttemptDenial> {
        Self::captured_inputs_remain_equal(self, execution, current)
    }

    fn complete_compiled_body_projection(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        projection: BorrowedOperationProjection<Schema, Operation>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::complete_compiled_body_projection(self, execution, projection, allocation)
    }
}

pub trait CompleteReadSetWorkflowInternalPort<Schema, Operation, Input, Scope>: Sized
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    fn materialize_workflow_definition_retirement<Capability, Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        published: PublishedWorkflowDefinitionRef,
        selected_occurrence: &WorthQueryProductBranchReadIdentity,
    ) -> Result<
        PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    fn materialize_workflow_definition_publication<Capability, Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        bound: BoundWorkflowDefinitionContract<Schema, Spec>,
        expected_predecessor: WorkflowDefinitionExpectedPredecessor,
    ) -> Result<
        PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    fn into_required_workflow_birth<Binding, Role, Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        published: &PublishedWorkflowDefinitionRef,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        clock: &WorthQueryRuntimeClock,
    ) -> Result<RequiredWorkflowBirth, WorthQueryApplicationAttemptDenial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryCreateOutput,
        >,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    fn materialize_workflow_instance_cancellation<Capability, Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        cancel_key_identity: [u8; 32],
    ) -> Result<
        PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    #[allow(clippy::too_many_arguments)]
    fn materialize_workflow_instance_migration<Capability, Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        succession: WorkflowSuccession,
        source: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
        start_key_identity: [u8; 32],
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    #[allow(clippy::too_many_arguments)]
    fn into_required_workflow_succession<Binding, Role, Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        predecessor: WorkflowSuccessionTarget<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        support: &WorthQueryInstalledProgramSupport<Schema>,
        clock: &WorthQueryRuntimeClock,
    ) -> Result<RequiredWorkflowSuccession, WorthQueryApplicationAttemptDenial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
        >,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    fn materialize_workflow_instance_start<Capability, Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        published: PublishedWorkflowDefinitionRef,
        start_key_identity: [u8; 32],
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    #[allow(clippy::too_many_arguments)]
    fn materialize_workflow_proposal<Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        input_identity: [u8; 32],
        source_identity: Option<[u8; 32]>,
        captured: WorkflowCapturedInputs,
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    fn authorize_advance<Capability, Spec>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    fn materialize_admitted_workflow_transition<Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        request_kind: WorkflowTransitionRequestKind,
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    #[allow(clippy::too_many_arguments)]
    fn materialize_workflow_approval<Capability, Spec>(
        self,
        execution: &ExecutionRequest<'_, '_>,
        workflow: &dyn WorkflowVocabularyPort<Schema, Spec>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>;

    #[allow(clippy::too_many_arguments)]
    fn resolve_operation_evidence(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        compiled: &CompiledWorkflowDefinition,
        layout: &WorthQueryWorkflowLayout,
        instance: &PublishedWorkflowInstanceRef,
        selected: &SelectedWorkflowTransition,
        progress: &WorkflowInstanceProgress,
        operation: &SelectedWorkflowOperation,
        facts: &mut Vec<WorthQueryApplicationObservedFact>,
    ) -> Result<WorkflowOperationEvidence, WorthQueryApplicationAttemptDenial>;

    #[allow(clippy::too_many_arguments)]
    fn materialize_operation_requirement(
        self,
        execution: &ExecutionRequest<'_, '_>,
        layout: &WorthQueryWorkflowLayout,
        compiled: &CompiledWorkflowDefinition,
        instance: PublishedWorkflowInstanceRef,
        selected: SelectedWorkflowTransition,
        live_membership: RelationId,
        retire_live_membership: bool,
        facts: Vec<WorthQueryApplicationObservedFact>,
        authority_facts: Vec<WorthQueryApplicationObservedFact>,
        progress: &WorkflowInstanceProgress,
        operation: SelectedWorkflowOperation,
        allowance: WorkflowStepAllowance,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >;
}
