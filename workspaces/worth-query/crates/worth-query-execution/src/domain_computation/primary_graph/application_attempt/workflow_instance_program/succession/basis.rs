//! The named predecessor's native pin and completion, not caller-selected Retained admission.
use super::super::super::{
    observe_adjacency, PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use super::*;
use crate::domain_computation::{
    primary_graph::workflow::definition::{
        reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture,
    },
    runtime_time::WorthQueryRuntimeClock,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;
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
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_instance_program) fn observe_succession_basis<
        Spec,
    >(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        predecessor: &PublishedWorkflowInstanceRef,
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        (CompiledWorkflowDefinition, WorkflowLineageCarry),
        WorthQueryApplicationAttemptDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if predecessor.branch() != self.lease.product().product_branch() {
            return Err(mismatch("succession predecessor belongs to another branch"));
        }
        let layout = self.lease.layout.workflow().clone();
        let published = PublishedWorkflowDefinitionRef::retained(
            predecessor.branch(),
            predecessor.definition_entity_id(),
            predecessor.definition_content_identity().clone(),
        );
        let (mut compiled, mut facts) = reconstruct_compiled_definition(
            execution,
            self.lease.handle(),
            self.lease.snapshot(),
            &layout,
            &published,
            installed.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Retained,
        )?;
        // Observation may restore a migration-resume position on the old instance.
        // A fresh lifecycle episode always begins at the actual definition start.
        let fresh = compiled.clone();
        let maximum = usize::try_from(
            installed
                .resources()
                .maximum_retained_transitions_per_instance(),
        )
        .unwrap_or(usize::MAX);
        let subject = self.admission.scope_entity_id();
        let mut observed = self.lease.handle().with_runtime(execution, |runtime| {
            super::super::super::workflow_instance_observation::observe_workflow_instance(execution, self.lease.handle(), runtime, self.lease.snapshot(), &layout,
                predecessor, subject, compiled.lineage(), &mut compiled, maximum, installed.resources().history_reconstruction_budget(),
                super::super::super::workflow_instance_observation::WorkflowInstanceObservationPurpose::Close)
        })?;
        if observed.live_membership.is_some() {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceNotCompleted,
                "succession requires a completed predecessor",
            ));
        }
        let (custody, successors) = self.lease.handle().with_runtime(execution, |runtime| {
            observed.ensure_history(execution, self.lease.handle(), runtime, self.lease.snapshot(), &layout, predecessor.entity_id(), maximum, &compiled)?;
            let custody = super::super::super::workflow_instance_observation::instance_binding::fence_owner_custody(execution, runtime, self.lease.snapshot(), &layout,
                predecessor.entity_id(), &observed.transitions, "succession predecessor has unsettled owner custody")?;
            let successors = observe_adjacency(runtime, execution, self.lease.snapshot(), layout.instance_succeeded_from_relation, predecessor.entity_id(),
                WorthQueryApplicationAdjacencyDirection::Incoming, 2)?.ok_or_else(|| mismatch("succession membership is unavailable"))?;
            Ok::<_, WorthQueryApplicationAttemptDenial>((custody, successors))
        })?;
        if !successors.is_empty() {
            self.new_commit_refusal = Some(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAlreadySucceeded,
                "predecessor already has a fresh successor",
            ));
        }
        facts.append(&mut observed.facts);
        facts.push(custody);
        facts.push(WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.instance_succeeded_from_relation,
            anchor: predecessor.entity_id(),
            direction: WorthQueryApplicationAdjacencyDirection::Incoming,
            maximum_work_units: 2,
            relations: successors,
        });
        let carry =
            self.observe_fresh_instance_capacity(execution, installed, &fresh, facts, clock)?;
        Ok((fresh, carry))
    }
}
