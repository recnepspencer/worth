use super::SignalGraph;

impl SignalGraph {
    /// Producer provenance is lost permanently on allocation, cloning, or restore.
    pub(crate) fn has_initial_empty_origin(&self) -> bool {
        self.initial_empty_origin
            && self.arena.nodes.is_empty()
            && self.arena.active_nodes == 0
            && self.conditional_dependency_versions.is_empty()
            && self.authorization_policy_identities.len() == 0
    }
}
