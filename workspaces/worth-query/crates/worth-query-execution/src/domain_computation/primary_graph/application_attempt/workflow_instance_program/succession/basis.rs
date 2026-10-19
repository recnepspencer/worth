//! The named predecessor's native pin and completion, not caller-selected Retained admission.
use super::super::super::{
    observe_adjacency, PublishedWorkflowInstanceRef, WorthQueryApplicationAdjacencyDirection,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use super::{
    mismatch, ApplicationSchema, ApplicationWorkflowSpec, CompiledWorkflowDefinition,
    WorkflowLineageCarry, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;
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
        target: super::WorkflowSuccessionTarget<Schema>,
        support: &crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport<Schema>,
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        (
            CompiledWorkflowDefinition,
            WorkflowLineageCarry,
            Option<super::CompletedOutputExpectation>,
            PublishedWorkflowInstanceRef,
        ),
        WorthQueryApplicationAttemptDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let resolved =
            self.resolve_succession_predecessor(execution, installed, target, support)?;
        let predecessor = resolved.reference;
        let mut compiled = resolved.compiled;
        let mut facts = resolved.facts;
        let layout = self.lease.layout.workflow().clone();
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
        let observed = self.lease.handle().with_runtime(execution, |runtime| {
            super::super::super::workflow_instance_observation::observe_workflow_instance(execution, self.lease.handle(), runtime, self.lease.snapshot(), &layout,
                &predecessor, subject, compiled.lineage(), &mut compiled, maximum, installed.resources().history_reconstruction_budget(),
                super::super::super::workflow_instance_observation::WorkflowInstanceObservationPurpose::Close)
        })?;
        let (source, successors) = self.lease.handle().with_runtime(execution, |runtime| {
            let source = observed.into_completed_source(
                execution,
                runtime,
                self.lease.snapshot(),
                &layout,
                predecessor.entity_id(),
            )?;
            let successors = observe_adjacency(
                runtime,
                execution,
                self.lease.snapshot(),
                layout.instance_succeeded_from_relation,
                predecessor.entity_id(),
                WorthQueryApplicationAdjacencyDirection::Incoming,
                2,
            )?
            .ok_or_else(|| mismatch("succession membership is unavailable"))?;
            Ok::<_, WorthQueryApplicationAttemptDenial>((source, successors))
        })?;
        if !successors.is_empty() {
            self.new_commit_refusal = Some(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAlreadySucceeded,
                "predecessor already has a fresh successor",
            ));
        }
        let output_match = self
            .admission
            .allowed_graph_contract()
            .body()
            .and_then(|body| body.succession())
            .and_then(|s| s.output_match())
            .map(|matching| super::CompletedOutputExpectation {
                declaration: matching.clone(),
                target: source.output_matching(&compiled, matching),
            });
        facts.extend(source.into_facts());
        facts.push(WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.instance_succeeded_from_relation,
            anchor: predecessor.entity_id(),
            direction: WorthQueryApplicationAdjacencyDirection::Incoming,
            maximum_work_units: 2,
            relations: successors,
        });
        let carry =
            self.observe_fresh_instance_capacity(execution, installed, &fresh, facts, clock)?;
        Ok((fresh, carry, output_match, predecessor))
    }
}
