//! Select the single authored business route, never the historical cursor stack.
use super::{
    denial, identity::business_return_identity, select_current_node, SelectedWorkflowOperation,
    SelectedWorkflowTransition, SelectedWorkflowTransitionKind,
};
use worth_relational::facade::identity::EntityId;
use {crate::domain_computation::primary_graph::application_attempt::workflow::definition::CompiledWorkflowConnectionKind, crate::domain_computation::primary_graph::application_attempt::workflow::definition::CompiledWorkflowDefinition, crate::domain_computation::primary_graph::application_attempt::workflow::instance::WorkflowTransitionProgressBasis, crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial, crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind};

pub(in crate::domain_computation::primary_graph) fn select_back_action_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    basis: &WorkflowTransitionProgressBasis,
    requested: &SelectedWorkflowOperation,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let progress = basis.progress();
    let source = select_current_node(compiled, progress)?;
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::transition::admission::selection::navigation::deny_back_over_authorized_operation(compiled, progress, source)?;
    let (_, kind) = compiled.back_action(source.entity())?;
    let CompiledWorkflowConnectionKind::BackAction {
        operation,
        input_type,
        binding,
        requirements,
    } = kind
    else {
        unreachable!("Back route selected")
    };
    if requested.operation != *operation
        || requested.input_type != *input_type
        || requested.binding.as_deref() != Some(binding)
        || &requested.workflow_execution_requirements != requirements
    {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
            "requested binding does not own the head's authored Back action",
        ));
    }
    let occurrence = progress.next_occurrence();
    let (identity, identity_bytes) = business_return_identity(
        compiled,
        instance,
        source,
        occurrence,
        progress.back_edge_iterations(),
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
