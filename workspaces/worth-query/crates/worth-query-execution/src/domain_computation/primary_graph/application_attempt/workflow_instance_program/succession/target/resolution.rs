//! Resolve on the admitted mutation basis and compile its actual retained pin once.
use super::WorkflowSuccessionTarget;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
        WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
        WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
    },
    program_occurrence::WorthQueryInstalledProgramSupport,
    workflow::definition::{
        reconstruct_compiled_definition, CompiledWorkflowDefinition,
        WorkflowDefinitionCompilationPosture,
    },
};
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};
mod original;

pub(in crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::succession)
struct ResolvedPredecessor
{
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::succession) reference:
        PublishedWorkflowInstanceRef,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::succession) compiled:
        CompiledWorkflowDefinition,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::succession) facts:
        Vec<WorthQueryApplicationObservedFact>,
}
impl<
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Input,
        Scope,
    >
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::succession) fn resolve_succession_predecessor<
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        target: WorkflowSuccessionTarget<Schema>,
        support: &WorthQueryInstalledProgramSupport<Schema>,
    ) -> Result<ResolvedPredecessor, WorthQueryApplicationAttemptDenial> {
        let branch = self.lease.product().product_branch();
        match target {
            WorkflowSuccessionTarget::Published(reference) => {
                if reference.branch() != branch {
                    return Err(super::super::mismatch(
                        "succession predecessor belongs to another branch",
                    ));
                }
                let pin = PublishedWorkflowDefinitionRef::retained(
                    branch,
                    reference.definition_entity_id(),
                    reference.definition_content_identity().clone(),
                );
                let (compiled, facts) = self.compile_succession_pin(execution, installed, &pin)?;
                Ok(ResolvedPredecessor {
                    reference,
                    compiled,
                    facts,
                })
            }
            WorkflowSuccessionTarget::Original(original) => {
                if original.subject_entity_id() != self.admission.scope_entity_id() {
                    return Err(WorthQueryApplicationAttemptDenial::new(
                        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                        "original workflow intent belongs to another admitted subject",
                    ));
                }
                let mut observed = self.lease.handle().with_runtime(execution, |runtime| {
                    original::observe::<Schema, Spec>(
                        execution,
                        runtime,
                        self.lease.snapshot(),
                        self.lease.layout.workflow(),
                        branch,
                        &original,
                        support,
                    )
                })?;
                let pin = PublishedWorkflowDefinitionRef::retained(
                    branch,
                    observed.definition,
                    observed.content.clone(),
                );
                let (compiled, facts) = self.compile_succession_pin(execution, installed, &pin)?;
                observed.facts.extend(facts);
                let path = observed
                    .resume
                    .unwrap_or_else(|| compiled.start().path().to_owned());
                let reference = PublishedWorkflowInstanceRef::observed_current(
                    branch,
                    observed.instance,
                    observed.definition,
                    observed.content,
                    observed.original_revision,
                    path,
                );
                Ok(ResolvedPredecessor {
                    reference,
                    compiled,
                    facts: observed.facts,
                })
            }
        }
    }
    fn compile_succession_pin<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        pin: &PublishedWorkflowDefinitionRef,
    ) -> Result<
        (
            CompiledWorkflowDefinition,
            Vec<WorthQueryApplicationObservedFact>,
        ),
        WorthQueryApplicationAttemptDenial,
    > {
        reconstruct_compiled_definition(
            execution,
            self.lease.handle(),
            self.lease.snapshot(),
            self.lease.layout.workflow(),
            pin,
            installed.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Retained,
        )
    }
}
