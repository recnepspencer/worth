use super::*;

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn observe_operation_approval_inputs(
    compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    graph_layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
    resource: worth_relational::facade::identity::EntityId,
    instance: &super::super::super::PublishedWorkflowInstanceRef,
    progress: &crate::domain_computation::primary_graph::workflow::instance::WorkflowInstanceProgress,
    approval: worth_relational::facade::identity::EntityId,
    target_operation: &str,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_facts: usize,
) -> Result<
    (
        Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
        crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    let observed = inputs::observe(
        compiled,
        layout,
        graph_layout,
        resource,
        instance,
        progress,
        approval,
        None,
        handle,
        snapshot,
        maximum_facts,
    )?;
    let mut facts = observed.facts;
    facts.extend(observed.currentness);
    let transition = progress
        .latest_transition(approval)
        .ok_or_else(|| affinity("approved transition is absent"))?;
    let evidence = observed
        .evidence
        .iter()
        .map(|entry| entry.evidence.entity)
        .collect::<Vec<_>>();
    let authority = handle.with_runtime(|runtime| {
        super::super::super::workflow_instance_observation::observe_retained_approval_binding(
            runtime,
            snapshot,
            layout,
            graph_layout,
            transition.entity(),
            observed.proposal_entity,
            &evidence,
            target_operation,
            &mut facts,
        )
    })?;
    if facts.len() > maximum_facts {
        return Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
            "workflow operation approval input facts",
        ));
    }
    Ok((facts, authority))
}
