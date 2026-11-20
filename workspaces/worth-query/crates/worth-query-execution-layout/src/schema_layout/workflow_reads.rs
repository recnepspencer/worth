//! Query read-index custody installed with this exact graph layout.
use super::WorthQueryPrimaryGraphLayout;
use crate::workflow_read_indexes::WorkflowReadIndexes;
impl WorthQueryPrimaryGraphLayout {
    pub fn workflow_reads(&self) -> Option<WorkflowReadIndexes> {
        self.workflow_reads
    }
    pub fn set_workflow_reads(&mut self, indexes: WorkflowReadIndexes) {
        self.workflow_reads = Some(indexes);
    }
}
