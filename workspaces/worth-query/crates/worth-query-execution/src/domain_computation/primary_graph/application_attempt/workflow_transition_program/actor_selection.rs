//! Exact native role selection shared by admission and later read owners.
use super::*;
mod approval;
mod portable;
mod sources;
use super::actor_context::{Command, WorthQueryWorkflowActorContext};
use crate::domain_computation::primary_graph::{
    workflow::{
        actor::{record::WorkflowActorFields, WorkflowActorDependencies},
        definition::{CompiledWorkflowDefinition, CompiledWorkflowNode},
        instance::WorkflowTransitionLocator,
    },
    WorthQueryApplicationObservedFact, WorthQueryPrimaryGraphIntegrationHandle,
    WorthQueryPrimaryGraphLayout,
};
use approval::approval_actor;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRecordedActorRequirement as Requirement,
    ApplicationRecordedActorComparison as Comparison,
    ApplicationWorkflowRecordedActorSource as Source, ApplicationWorkflowRecordedRole as Role,
};
use worth_relational::facade::{identity::EntityId, snapshots::SnapshotHandle};

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorkflowRecordedActorObservation {
    requirement: Requirement,
    actor: Option<EntityId>,
    record: Option<EntityId>,
    dependencies: WorkflowActorDependencies,
}
impl WorkflowRecordedActorObservation {
    pub(in crate::domain_computation) fn remains_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        match self
            .dependencies
            .remains_current(execution, runtime, snapshot)
        {
            Ok(false) => Ok(false),
            Ok(true) => match self.record {
                None => Ok(self.actor.is_none()),
                Some(record) => match self.dependencies.recorded_actor(
                    execution,
                    runtime,
                    snapshot,
                    record,
                    self.requirement.role(),
                ) {
                    Ok(actor) => Ok(actor == self.actor),
                    Err(cause) => currentness_denial(cause),
                },
            },
            Err(cause) => currentness_denial(cause),
        }
    }
    pub(in crate::domain_computation) fn requirement(&self) -> Requirement {
        self.requirement
    }
    pub(in crate::domain_computation) fn dependencies(&self) -> &WorkflowActorDependencies {
        &self.dependencies
    }
    pub(in crate::domain_computation) fn matches(&self, principal: EntityId) -> bool {
        self.actor
            .is_some_and(|actor| match self.requirement.comparison() {
                Comparison::SamePrincipal => actor == principal,
                Comparison::DifferentPrincipal => actor != principal,
            })
    }
}
impl WorthQueryWorkflowActorContext {
    pub(in crate::domain_computation) fn observe(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &WorthQueryPrimaryGraphIntegrationHandle,
        snapshot: &SnapshotHandle,
        graph: &WorthQueryPrimaryGraphLayout,
        resource: EntityId,
        requirement: Requirement,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorkflowRecordedActorObservation, WorthQueryApplicationAttemptDenial> {
        if !requirement.is_valid() || self.source != requirement.source() {
            return Err(affinity(
                "workflow recorded role differs from installed binding",
            ));
        }
        let layout = graph.workflow();
        let published = super::super::PublishedWorkflowDefinitionRef::retained(
            self.instance.branch(),
            self.instance.definition_entity_id(),
            self.instance.definition_content_identity().clone(),
        );
        let (compiled, mut facts) = reconstruct_compiled_definition(
            execution,
            handle,
            snapshot,
            layout,
            &published,
            &self.revision,
            &self.spec,
            &self.support,
            usize::from(self.resources.maximum_definition_nodes()),
            usize::from(self.resources.maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Retained,
        )?;
        handle.with_runtime(execution, |runtime| {
            super::super::workflow_instance_observation::original_scope(
                execution,
                runtime,
                snapshot,
                layout,
                &self.instance,
                resource,
                compiled.lineage(),
                &mut facts,
            )
        })?;
        let (proposal, transition, approval) = self.select_original_sources(
            execution,
            handle,
            snapshot,
            layout,
            &compiled,
            requirement,
            request,
            &mut facts,
        )?;
        let mut record = match requirement.source() {
            Source::BoundProposal => proposal,
            Source::BoundTransition => transition.map(|transition| transition.entity()),
            Source::BoundApproval => None,
        };
        let actor = handle.with_runtime(execution, |runtime| match requirement.source() {
            Source::BoundProposal => match proposal {
                Some(entity) => layout
                    .proposal
                    .proposer
                    .observe(
                        runtime,
                        execution,
                        snapshot,
                        entity,
                        layout.proposal.entity_kind,
                        &mut facts,
                    )
                    .and_then(|actor| crate::domain_computation::primary_graph::workflow::actor::record::validate_actor_applicability(actor, true)),
                None => Ok(None),
            },
            Source::BoundTransition => match transition {
                Some(transition) => {
                    super::super::workflow_instance_observation::observe_retained_transition(
                        execution, runtime, snapshot, layout, transition, &mut facts,
                    )?;
                    let fields = match requirement.role() {
                        Role::Advancer => &layout.transition.advancer,
                        Role::Executor => &layout.transition.executor,
                        _ => return Err(affinity("invalid transition role")),
                    };
                    let actor = fields.observe(
                        runtime,
                        execution,
                        snapshot,
                        transition.entity(),
                        layout.transition.entity_kind,
                        &mut facts,
                    )?;
                    let required = requirement.role() == Role::Advancer
                        || transition.settlement().operation_receipt_identity().is_some();
                    crate::domain_computation::primary_graph::workflow::actor::record::validate_actor_applicability(actor, required)
                }
                None => Ok(None),
            },
            Source::BoundApproval => {
                let (actor, entity) = approval_actor(
                    execution,
                    runtime,
                    snapshot,
                    layout,
                    approval,
                    proposal,
                    match &self.command {
                        Command::Operation { required } => {
                            match (approval, required.original_sources.approval_parent) {
                                (Some(_), Some(parent)) => parent,
                                (None, None) => self.instance.entity_id(), // no record is read
                                _ => {
                                    return Err(affinity(
                                        "original approval parent pairing is incomplete",
                                    ))
                                }
                            }
                        }
                        Command::Approval { .. } => self.instance.entity_id(),
                    },
                    &mut facts,
                )?;
                record = entity;
                Ok(actor)
            }
        })?;
        let dependencies = handle.with_runtime(execution, |runtime| {
            WorkflowActorDependencies::capture(
                execution, runtime, snapshot, layout, &facts, request,
            )
        })?;
        Ok(WorkflowRecordedActorObservation {
            requirement,
            actor,
            record,
            dependencies,
        })
    }
}
fn unique<'a>(
    mut nodes: impl Iterator<Item = &'a CompiledWorkflowNode>,
) -> Result<&'a CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    match (nodes.next(), nodes.next()) {
        (Some(node), None) => Ok(node),
        _ => Err(affinity("workflow actor source is not singular")),
    }
}
fn affinity(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

fn currentness_denial(
    cause: WorthQueryApplicationAttemptDenial,
) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    if let Some(native) = cause.relational_execution_cause() {
        return Err(native);
    }
    if let Some(allocation) = cause.allocation_denial()
        && let worth_execution::ExecutionAllocationDenialClass::Request(stop) =
            allocation.classify()
    {
        return Err(stop.into());
    }
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
    match cause.kind() {
        WorthQueryApplicationAttemptDenialKind::Cancelled => Err(Native::Cancelled),
        WorthQueryApplicationAttemptDenialKind::DeadlineExceeded => Err(Native::DeadlineElapsed),
        _ => Ok(false),
    }
}
