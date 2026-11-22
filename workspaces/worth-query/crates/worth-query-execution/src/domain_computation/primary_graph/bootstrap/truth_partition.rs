use super::WorthQueryPrimaryGraphBootstrap;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema> {
    /// Binds the single primary Relational partition to its cross-runtime
    /// semantic role before publication.
    pub fn semantic_truth_partition(
        mut self,
        role: worth_foundational::facade::TruthPartitionRole,
    ) -> Self {
        self.graph.bind_truth_partition(role);
        self
    }
}
