//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow::instance::transition::admission::selection`: the items below and every impl they own.

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowCondition {
    pub(in crate::domain_computation::primary_graph) condition:
        crate::domain_computation::primary_graph::vocabulary::application_attempt::condition::CompiledWorkflowCondition,
}
