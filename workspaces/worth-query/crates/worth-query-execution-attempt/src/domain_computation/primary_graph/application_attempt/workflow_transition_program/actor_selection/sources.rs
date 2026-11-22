//! Exact original recorded sources; current command eligibility stays in its owner.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation as native;
impl WorthQueryWorkflowActorContext {
    pub(super) fn select_original_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &WorthQueryPrimaryGraphIntegrationHandle,
        snapshot: &SnapshotHandle,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        compiled: &CompiledWorkflowDefinition,
        requirement: Requirement,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        facts: &mut Vec<WorthQueryApplicationObservedFact>,
    ) -> Result<
        (
            Option<EntityId>,
            Option<WorkflowTransitionLocator>,
            Option<WorkflowTransitionLocator>,
        ),
        WorthQueryApplicationAttemptDenial,
    > {
        match &self.command {
            Command::Approval { required, proposal } => {
                if required.instance() != self.instance.entity_id()
                    || required.operation() != self.operation
                    || proposal.branch() != self.instance.branch()
                    || proposal.instance_entity_id() != self.instance.entity_id()
                    || proposal.definition_entity_id() != self.instance.definition_entity_id()
                {
                    return Err(affinity("workflow approval actor context changed"));
                }
                let node =
                    super::super::approval_decision::actor_approval_node(compiled, required)?;
                let source = unique(compiled.approval_proposal_sources(node))?;
                let (transition, identity, parent) = handle.with_runtime(execution, |runtime| {
                    let (transition, identity) = native::original_transition(
                        execution,
                        runtime,
                        snapshot,
                        layout,
                        proposal.entity_id(),
                        facts,
                    )?;
                    let parent = native::original_parent(
                        execution,
                        runtime,
                        snapshot,
                        layout,
                        transition.entity(),
                        facts,
                    )?;
                    Ok::<_, WorthQueryApplicationAttemptDenial>((transition, identity, parent))
                })?;
                if parent != self.instance.entity_id()
                    || transition.settlement().node() != source.entity()
                    || proposal.node_path() != source.path()
                {
                    return Err(affinity("proposal is not the original typed source"));
                }
                self.validate_proposal(
                    execution,
                    handle,
                    snapshot,
                    layout,
                    source,
                    transition,
                    &identity,
                    proposal.entity_id(),
                    proposal.identity(),
                    proposal.input_identity(),
                    request,
                    facts,
                )?;
                let approval = if requirement.source() == Source::BoundApproval {
                    handle.with_runtime(execution, |runtime| {
                        native::recorded_approval(
                            execution,
                            runtime,
                            snapshot,
                            layout,
                            self.instance.entity_id(),
                            required.transition_identity(),
                            node,
                            required.occurrence(),
                            usize::try_from(
                                self.resources.maximum_retained_transitions_per_instance(),
                            )
                            .unwrap_or(usize::MAX),
                            self.resources.history_reconstruction_budget(),
                            compiled,
                            facts,
                        )
                    })?
                } else {
                    None
                };
                Ok((Some(proposal.entity_id()), Some(transition), approval))
            }
            Command::Operation { required } => {
                if required.instance != self.instance.entity_id()
                    || required.branch != self.instance.branch()
                    || required.operation != self.operation
                    || required.binding.as_deref() != Some(self.binding.as_str())
                {
                    return Err(affinity("workflow operation actor context changed"));
                }
                // Local Back/termination bodies have no proposal or performed
                // source. Positive recorded-actor requirements must fail on
                // that absence, never inherit a convenient unrelated record.
                let Some(originals) = &required.original_sources else {
                    return Ok((None, None, None));
                };
                let node = unique(compiled.nodes_with_path(&required.node_path).iter())?;
                let source = unique(
                    compiled
                        .operation_input_sources(node.entity())
                        .chain(compiled.captured_operation_input_sources(node.entity())),
                )?;
                let (transition, identity, parent) = handle.with_runtime(execution, |runtime| {
                    let (transition, identity) = native::original_transition(
                        execution,
                        runtime,
                        snapshot,
                        layout,
                        originals.proposal,
                        facts,
                    )?;
                    let parent = native::original_parent(
                        execution,
                        runtime,
                        snapshot,
                        layout,
                        transition.entity(),
                        facts,
                    )?;
                    Ok::<_, WorthQueryApplicationAttemptDenial>((transition, identity, parent))
                })?;
                if transition != originals.transition
                    || identity != originals.transition_identity
                    || parent != originals.parent_instance
                    || transition.settlement().node() != source.entity()
                {
                    return Err(affinity("original workflow source pairing changed"));
                }
                if let Some(expected) = &required.captured_input {
                    let actual = handle.with_runtime(execution, |runtime| {
                        crate::domain_computation::primary_graph::workflow::proposal::observe_captured_workflow_input(
                            execution, runtime, snapshot, layout, originals.proposal, &originals.proposal_identity,
                            &required.node_path, expected.resource, facts,
                        )
                    })?;
                    if &actual != expected || actual.input != required.input_identity {
                        return Err(affinity("recorded actor captured target changed"));
                    }
                } else if required.input_identity != originals.proposal_input_identity {
                    return Err(affinity("homogeneous operation input changed"));
                }
                self.validate_proposal(
                    execution,
                    handle,
                    snapshot,
                    layout,
                    source,
                    transition,
                    &identity,
                    originals.proposal,
                    &originals.proposal_identity,
                    &originals.proposal_input_identity,
                    request,
                    facts,
                )?;
                let mut approvals = compiled.approval_authority_sources(node.entity());
                match (approvals.next(), approvals.next(), originals.approval) {
                    (None, None, None) => {}
                    (Some(source), None, Some(approval))
                        if approval.settlement().node() == source.entity() => {}
                    _ => {
                        return Err(affinity(
                            "original approval source differs from typed binding",
                        ))
                    }
                }
                Ok((
                    Some(originals.proposal),
                    Some(transition),
                    originals.approval,
                ))
            }
        }
    }
    fn validate_proposal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &WorthQueryPrimaryGraphIntegrationHandle,
        snapshot: &SnapshotHandle,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        source: &CompiledWorkflowNode,
        transition: WorkflowTransitionLocator,
        transition_identity: &str,
        proposal: EntityId,
        identity: &str,
        input_identity: &[u8; 32],
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        facts: &mut Vec<WorthQueryApplicationObservedFact>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::Operation { operation, input_type, .. } = source.kind()
            else { return Err(affinity("proposal source is not an operation")); };
        let input = handle.with_runtime(execution, |runtime|
            crate::domain_computation::primary_graph::workflow::proposal::observe_workflow_operation_input(
                execution, runtime, snapshot, layout, transition.entity(), transition_identity,
                operation, input_type, source.path(), request))?;
        if input.proposal != proposal
            || input.proposal_identity != identity
            || &input.input_identity != input_identity
        {
            return Err(affinity("original proposal canonical pairing changed"));
        }
        facts.extend(input.facts);
        Ok(())
    }
}
