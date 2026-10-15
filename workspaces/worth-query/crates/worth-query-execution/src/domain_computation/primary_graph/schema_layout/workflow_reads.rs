//! Query read-index custody installed with this exact graph layout.
use super::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::application_discovery::WorkflowReadIndexes;
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn workflow_reads(
        &self,
    ) -> Option<WorkflowReadIndexes> {
        self.workflow_reads
    }
    pub(in crate::domain_computation::primary_graph) fn set_workflow_reads(
        &mut self,
        indexes: WorkflowReadIndexes,
    ) {
        self.workflow_reads = Some(indexes);
    }
}
