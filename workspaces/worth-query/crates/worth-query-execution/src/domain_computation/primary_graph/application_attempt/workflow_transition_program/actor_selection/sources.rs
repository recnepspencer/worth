//! Exact original recorded sources; eligibility remains in the workflow owner.
use super::*;
impl WorthQueryWorkflowActorContext {
    pub(super) fn select_original_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &WorthQueryPrimaryGraphIntegrationHandle,
        snapshot: &SnapshotHandle,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        compiled: &CompiledWorkflowDefinition,
        observed: &mut super::super::super::workflow_instance_observation::ObservedWorkflowInstance,
        requirement: Requirement,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        facts: &mut Vec<WorthQueryApplicationObservedFact>,
    ) -> Result<(Option<EntityId>, Option<WorkflowTransitionLocator>, Option<WorkflowTransitionLocator>), WorthQueryApplicationAttemptDenial> {
        let progress = observed.progress_basis.progress();
        let selected = match &self.command {
            Command::Approval { required, proposal } => {
                if required.instance() != self.instance.entity_id()
                    || required.operation() != self.operation
                    || proposal.branch() != self.instance.branch()
                    || proposal.instance_entity_id() != self.instance.entity_id()
                    || proposal.definition_entity_id() != self.instance.definition_entity_id()
                {
                    return Err(affinity("workflow approval actor context changed"));
                }
                let node = super::approval_decision::actor_approval_node(&compiled, required)?;
                let source = unique(compiled.approval_proposal_sources(node))?;
                let transition = progress.latest_transition(source.entity());
                let proposal_entity = match transition {
                    Some(transition) => {
                        let (entity, identity, source_facts) = handle.with_runtime(execution, |runtime|
                            super::super::workflow_instance_observation::observe_retained_workflow_proposal_identity(
                                execution, runtime, snapshot, layout, transition, request))?;
                        facts.extend(source_facts);
                        let crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::Operation { operation, input_type, .. } = source.kind()
                            else { return Err(affinity("workflow proposal source is not an operation")); };
                        let source_identity = progress.latest_transition_identity(source.entity())
                            .ok_or_else(|| affinity("workflow proposal transition identity absent"))?;
                        let input = handle.with_runtime(execution, |runtime|
                            crate::domain_computation::primary_graph::workflow::proposal::observe_workflow_operation_input(
                                execution, runtime, snapshot, layout, transition.entity(), source_identity,
                                operation, input_type, source.path(), request))?;
                        if input.proposal != entity || input.proposal_identity != identity
                            || &input.input_identity != proposal.input_identity() {
                            return Err(affinity("workflow proposal canonical pairing changed"));
                        }
                        facts.extend(input.facts);
                        if proposal.node_path() != source.path()
                            || entity != proposal.entity_id()
                            || identity != proposal.identity()
                        {
                            return Err(affinity(
                                "workflow actor proposal is not the consumed occurrence",
                            ));
                        }
                        Some(entity)
                    }
                    None => None,
                };
                let latest = progress.latest_transition(node);
                let mut approval = latest.filter(|entry| entry.settlement().occurrence() == required.occurrence());
                if approval.is_none() && requirement.source() == Source::BoundApproval
                    && latest.is_some_and(|entry| entry.settlement().occurrence() > required.occurrence()) {
                    handle.with_runtime(execution, |runtime| observed.ensure_history(
                        execution, handle, runtime, snapshot, layout, self.instance.entity_id(),
                        usize::try_from(self.resources.maximum_retained_transitions_per_instance()).unwrap_or(usize::MAX), compiled))?;
                    approval = observed.transitions.iter().find(|entry|
                        entry.settlement.node() == node && entry.settlement.occurrence() == required.occurrence())
                        .map(|entry| WorkflowTransitionLocator::new(entry.entity, entry.settlement));
                }
                (proposal_entity, transition, approval)
            }
            Command::Operation { required } => {
                if required.instance != self.instance.entity_id()
                    || required.branch != self.instance.branch()
                    || required.operation != self.operation
                    || required.binding.as_deref() != Some(self.binding.as_str())
                {
                    return Err(affinity("workflow operation actor context changed"));
                }
                let mut nodes = compiled.nodes_with_path(&required.node_path).iter();
                let (Some(node), None) = (nodes.next(), nodes.next()) else {
                    return Err(affinity("workflow operation node changed"));
                };
                let source = unique(compiled.operation_input_sources(node.entity()))?;
                let originals = &required.original_sources;
                let transition = progress.latest_transition(source.entity());
                if transition != Some(originals.transition)
                    || progress.latest_transition_identity(source.entity())
                        != Some(originals.transition_identity.as_str())
                {
                    return Err(affinity(
                        "original workflow input occurrence is no longer consumed",
                    ));
                }
                let proposal = match transition {
                    Some(transition) => {
                        let (entity, proposal_identity, source_facts) = handle.with_runtime(execution, |runtime|
                            super::super::workflow_instance_observation::observe_retained_workflow_proposal_identity(
                                execution, runtime, snapshot, layout, transition, request))?;
                        if entity != originals.proposal
                            || proposal_identity != originals.proposal_identity
                        {
                            return Err(affinity("original workflow actor proposal changed"));
                        }
                        facts.extend(source_facts);
                        let crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::Operation { operation, input_type, .. } = source.kind()
                            else { return Err(affinity("workflow operation input source changed")); };
                        let input = handle.with_runtime(execution, |runtime|
                            crate::domain_computation::primary_graph::workflow::proposal::observe_workflow_operation_input(
                                execution, runtime, snapshot, layout, transition.entity(),
                                progress.latest_transition_identity(source.entity()).ok_or_else(|| affinity("workflow input identity absent"))?,
                                operation, input_type, source.path(), request))?;
                        if &input.input_identity != &required.input_identity {
                            return Err(affinity("workflow actor input changed"));
                        }
                        facts.extend(input.facts);
                        Some(entity)
                    }
                    None => None,
                };
                let mut approvals = compiled.approval_authority_sources(node.entity());
                let approval = match (approvals.next(), approvals.next()) {
                    (None, None) => None,
                    (Some(source), None) => progress.latest_transition(source.entity()),
                    _ => return Err(affinity("workflow approval source is not singular")),
                };
                if approval != originals.approval {
                    return Err(affinity("original workflow approval occurrence changed"));
                }
                (proposal, transition, approval)
            }
        };
        Ok(selected)
    }
}
