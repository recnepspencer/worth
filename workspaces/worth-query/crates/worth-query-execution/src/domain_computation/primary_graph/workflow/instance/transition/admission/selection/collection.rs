use worth_relational::facade::identity::EntityId;

use super::{
    denial, select_transition, CompiledWorkflowDefinition, CompiledWorkflowNodeKind,
    SelectedWorkflowAssessment, SelectedWorkflowTransition, SelectedWorkflowTransitionKind,
    WorkflowTransitionProgressBasis, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};

pub(in crate::domain_computation::primary_graph) fn select_assessment_collection(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    progress_basis: &WorkflowTransitionProgressBasis,
    node_path: &str,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let [node] = compiled.nodes_with_path(node_path) else {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
            node_path,
        ));
    };
    let CompiledWorkflowNodeKind::Assessment {
        query,
        parameter_type,
        result_type,
        binding,
        subject,
        ..
    } = node.kind()
    else {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
            node_path,
        ));
    };
    let progress = progress_basis.progress();
    if node.entity() == progress.head() {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
            "current assessment uses ordinary workflow advance",
        ));
    }
    let mut sources = compiled.assessment_subject_sources(node.entity());
    let Some(source) = sources.next() else {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            node_path,
        ));
    };
    if sources.next().is_some() || progress.latest_transition(source.entity()).is_none() {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "assessment proposal source is not ready",
        ));
    }
    select_transition(
        compiled,
        instance,
        node,
        progress.next_occurrence(),
        SelectedWorkflowTransitionKind::Assessment(SelectedWorkflowAssessment {
            query: query.clone(),
            parameter_type: parameter_type.clone(),
            result_type: result_type.clone(),
            binding: binding.clone(),
            subject: subject.clone(),
        }),
        Some(progress_basis.clone()),
    )
}
