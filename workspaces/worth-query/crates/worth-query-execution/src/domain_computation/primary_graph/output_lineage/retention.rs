use super::WorthQueryApplicationOutputLineage;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn release_occurrence(
        &mut self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        self.live_occurrences.remove(&occurrence);
        let mut retained = self.live_occurrences.clone();
        let mut frontier = retained.iter().copied().collect::<Vec<_>>();
        while let Some(child) = frontier.pop() {
            if let Some(parent) = self
                .origins
                .get(&child)
                .map(|coordinate| coordinate.occurrence)
            {
                if retained.insert(parent) {
                    frontier.push(parent);
                }
            }
        }
        self.by_source.retain(|_, versions| {
            versions.retain(|indexed, _| retained.contains(indexed));
            !versions.is_empty()
        });
        self.origins.retain(|child, _| retained.contains(child));
    }
}
