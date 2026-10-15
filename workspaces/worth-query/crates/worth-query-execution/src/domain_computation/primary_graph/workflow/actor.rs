//! Native admission is the only author of canonical workflow actors.
mod dependencies;
pub(in crate::domain_computation) mod record;
mod revision_transport;
pub(in crate::domain_computation) use dependencies::WorkflowActorDependencies;

fn storage_denial(
    cause: worth_execution::ExecutionAllocationDenial,
    subject: &str,
) -> crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial {
    crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Allocation(cause)
        .into_attempt_denial(subject)
}
