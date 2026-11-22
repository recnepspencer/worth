use super::CompiledWorkflowDefinition;
use worth_query_installation::facade::WorthQueryWorkflowGlobalActionDependency;

impl CompiledWorkflowDefinition {
    pub fn global_actions(&self) -> &[WorthQueryWorkflowGlobalActionDependency] {
        &self.publication.global_actions
    }
}
