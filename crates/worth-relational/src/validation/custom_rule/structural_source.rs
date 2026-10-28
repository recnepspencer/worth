//! Genuine observation-to-view source custody shared by structural readers.
use super::StructuralRelationView;
use crate::validation::engine::InvariantSourceAffinity;

impl StructuralRelationView<'_> {
    pub(crate) fn with_structural_source(
        mut self,
        runtime: &crate::validation::engine::InvariantRuntimeView<'_>,
        observation: &crate::validation::engine::InvariantObservation<'_>,
        committed: bool,
    ) -> Self {
        let version = self.aggregate_read_parts().0.version_id();
        self.structural_source = Some((
            runtime.selected_registry.clone(),
            InvariantSourceAffinity {
                observation: observation.observation_identity(),
                committed,
                version,
                proposal: observation.proposal_identity().cloned(),
            },
        ));
        self
    }
}
