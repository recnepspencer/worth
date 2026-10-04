use super::WorthQueryApplicationObservedFact as Fact;

impl Fact {
    /// Combines observations of the same dependency at one native revision.
    /// Adjacency endpoints and comparison limits describe each read boundary;
    /// they may differ even when both reads observed the same structural truth.
    pub(in crate::domain_computation::primary_graph) fn merge_same_source_fact(
        &mut self,
        duplicate: Self,
    ) -> bool {
        if self.dependency_key() != duplicate.dependency_key() {
            return false;
        }
        match (self, duplicate) {
            (
                Fact::SourceAdjacencyRevision {
                    native_revision: first_revision,
                    comparison_work_limit: first_limit,
                    endpoints: first_endpoints,
                    ..
                },
                Fact::SourceAdjacencyRevision {
                    native_revision: second_revision,
                    comparison_work_limit: second_limit,
                    endpoints: second_endpoints,
                    ..
                },
            ) if *first_revision == second_revision => {
                *first_limit = (*first_limit).max(second_limit);
                first_endpoints.extend(second_endpoints);
                first_endpoints.sort();
                first_endpoints.dedup();
                true
            }
            (existing, duplicate) => *existing == duplicate,
        }
    }
}
