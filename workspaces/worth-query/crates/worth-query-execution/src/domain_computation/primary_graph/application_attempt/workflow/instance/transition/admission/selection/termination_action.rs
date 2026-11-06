//! A global action binds the current occurrence, not a manufactured graph edge.
use worth_query_installation::facade::WorthQueryWorkflowGlobalActionDependency;
use {
    super::denial, super::select_current_node, super::CompiledWorkflowDefinition, super::EntityId,
    super::SelectedWorkflowOperation, super::SelectedWorkflowTransition,
    super::SelectedWorkflowTransitionKind, super::WorkflowTransitionProgressBasis,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial,
    crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind,
};

pub(in crate::domain_computation::primary_graph) fn select_termination_action_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    basis: &WorkflowTransitionProgressBasis,
    requested: &SelectedWorkflowOperation,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let source = select_current_node(compiled, basis.progress())?;
    let action = compiled
        .global_actions()
        .iter()
        .find(|action| {
            let WorthQueryWorkflowGlobalActionDependency::Terminate {
                identifier,
                input_type,
                binding,
                requirements,
            } = action;
            requested.operation == *identifier
                && requested.input_type == *input_type
                && requested.binding.as_deref() == Some(binding)
                && &requested.workflow_execution_requirements == requirements
        })
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                "requested binding is not a declared global termination action",
            )
        })?;
    let occurrence = basis.progress().next_occurrence();
    let (identity, identity_bytes) = crate::domain_computation::primary_graph::application_attempt::workflow::instance::transition::admission::selection::identity::business_termination_identity(
        compiled,
        instance,
        source,
        occurrence,
        basis.progress().back_edge_iterations(),
        action,
    )?;
    Ok(SelectedWorkflowTransition {
        node: source.entity(),
        node_path: source.path().to_owned(),
        occurrence,
        identity,
        identity_bytes,
        progress_basis: Some(basis.clone()),
        kind: SelectedWorkflowTransitionKind::Operation(requested.clone()),
    })
}
