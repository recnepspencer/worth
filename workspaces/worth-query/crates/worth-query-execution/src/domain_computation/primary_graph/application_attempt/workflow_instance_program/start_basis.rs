//! Shared native definition, capacity and deadline admission for all fresh starts.
use super::{
    check_request_live, denial, observe_adjacency, reconstruct_compiled_definition, supersession,
    ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationWorkflowSpec,
    PublishedWorkflowDefinitionRef, WorkflowDefinitionCompilationPosture, WorkflowLineageCarry,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
    WorthQueryCompleteApplicationReadSet, WorthQueryInstalledApplicationWorkflowSpec,
    WorthQueryProjectedApplicationMutation, WorthQueryRuntimeClock,
};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
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
    pub(super) fn observe_workflow_start_basis<Spec>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        published: &PublishedWorkflowDefinitionRef,
        clock: &WorthQueryRuntimeClock,
    ) -> Result<
        (
            CompiledWorkflowDefinition,
            WorkflowLineageCarry,
            Option<supersession::WorkflowStartSupersession>,
        ),
        WorthQueryApplicationAttemptDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let layout = self.lease.layout.workflow().clone();
        let (compiled, mut compile_facts) = reconstruct_compiled_definition(
            execution,
            self.lease.handle(),
            self.lease.snapshot(),
            &layout,
            published,
            installed.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Current,
        )?;
        // A start writes only from the definition its branch holds current,
        // but a retry of one already recorded replays whatever changed since.
        let supersession = self
            .lease
            .handle()
            .with_runtime(execution, |runtime| {
                crate::domain_computation::primary_graph::application_discovery::current_definition(
                    runtime,
                    execution,
                    self.lease.snapshot(),
                    &layout,
                    published.branch(),
                    compiled.lineage(),
                )
            })?
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowLineageUnavailable,
                    self.admission.operation(),
                )
            })
            .map(|currentness| {
                supersession::WorkflowStartSupersession::of(published, currentness)
            })?;
        if let Some(supersession) = &supersession {
            self.new_commit_refusal = Some(denial(
                supersession.denial_kind(),
                self.admission.operation(),
            ));
        }
        let maximum_instances = usize::try_from(installed.resources().maximum_live_instances())
            .map_err(|_| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowLineageCapacityUnavailable,
                    self.admission.operation(),
                )
            })?;
        let instances = self.lease.handle().with_runtime(execution, |runtime| {
            observe_adjacency(
                runtime,
                execution,
                self.lease.snapshot(),
                layout.live_instance_lineage_relation,
                compiled.lineage(),
                WorthQueryApplicationAdjacencyDirection::Incoming,
                maximum_instances.saturating_mul(2).saturating_add(1),
            )
        })?;
        let declared_deadline = self.lease.handle().with_runtime(execution, |runtime| {
            super::super::workflow_deadline::definition_deadline(
                execution,
                runtime,
                self.lease.snapshot(),
                &layout,
                compiled.definition(),
                &mut compile_facts,
            )
        })?;
        // Every live instance pins its revision. A full lineage refuses a new
        // start once the commit knows it is no retry of a recorded one. A
        // lineage holding more than it can read, as a lowered ceiling can
        // leave it, is full too, so its recorded starts still replay.
        match instances {
            Some(instances) if instances.len() < maximum_instances => compile_facts.push(
                WorthQueryApplicationObservedFact::WorkflowInstanceCapacity {
                    relation_kind: layout.live_instance_lineage_relation,
                    lineage: compiled.lineage(),
                    maximum_instances,
                    instances,
                },
            ),
            _ => {
                let operation = self.admission.operation();
                self.new_commit_refusal.get_or_insert_with(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowLineageCapacityUnavailable,
                        operation,
                    )
                });
            }
        }
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        self.append_completed_facts(
            compile_facts,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )?;
        let carry = WorkflowLineageCarry {
            inherited_steps: 0,
            inherited_evidence_bytes: 0,
            deadline: super::super::workflow_deadline::start_deadline(
                clock,
                None,
                declared_deadline,
            )?,
        };
        Ok((compiled, carry, supersession))
    }
}
