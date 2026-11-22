//! Crate-boundary internal port implementation for
//! `CompleteReadSetWorkflowInternalPort`. Never exported on
//! `worth-query-execution::facade` or Host.

use worth_execution::ExecutionRequest;
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

use super::complete_read_set_methods_port::CompleteReadSetWorkflowInternalPort;
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

impl<Schema, Operation, Input, Scope>
    CompleteReadSetWorkflowInternalPort<Schema, Operation, Input, Scope>
    for WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::materialize_workflow_definition_retirement::<Capability, Spec>(
            self,
            execution,
            installed,
            published,
            selected_occurrence,
        )
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::materialize_workflow_definition_publication::<Capability, Spec>(
            self,
            execution,
            bound,
            expected_predecessor,
        )
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::into_required_workflow_birth::<Binding, Role, Spec>(
            self, execution, installed, published, identities, clock,
        )
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::materialize_workflow_instance_cancellation::<Capability, Spec>(
            self,
            execution,
            installed,
            instance,
            cancel_key_identity,
        )
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::materialize_workflow_instance_migration::<Capability, Spec>(
            self,
            execution,
            installed,
            succession,
            source,
            target,
            resume_at,
            start_key_identity,
            clock,
        )
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::into_required_workflow_succession::<Binding, Role, Spec>(
            self,
            execution,
            installed,
            predecessor,
            identities,
            support,
            clock,
        )
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::materialize_workflow_instance_start::<Capability, Spec>(
            self,
            execution,
            installed,
            published,
            start_key_identity,
            clock,
        )
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::materialize_workflow_proposal::<Spec>(
            self,
            execution,
            installed,
            instance,
            input_identity,
            source_identity,
            captured,
            clock,
        )
    }

    fn authorize_advance<Capability, Spec>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::authorize_advance::<Capability, Spec>(self, installed)
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::materialize_admitted_workflow_transition::<Spec>(
            self,
            execution,
            installed,
            instance,
            request_kind,
            clock,
        )
    }

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
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::materialize_workflow_approval::<Capability, Spec>(
            self, execution, workflow, installed, instance, required, proposal, decision, clock,
        )
    }

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
    ) -> Result<WorkflowOperationEvidence, WorthQueryApplicationAttemptDenial> {
        Self::resolve_operation_evidence(
            self, execution, compiled, layout, instance, selected, progress, operation, facts,
        )
    }

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
    > {
        Self::materialize_operation_requirement(
            self,
            execution,
            layout,
            compiled,
            instance,
            selected,
            live_membership,
            retire_live_membership,
            facts,
            authority_facts,
            progress,
            operation,
            allowance,
        )
    }
}
