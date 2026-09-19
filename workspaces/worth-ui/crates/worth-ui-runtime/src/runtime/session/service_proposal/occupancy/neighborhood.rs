use std::collections::HashMap;

type NeighborhoodIndexKey = (
    crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    worth_ui_host_contract::UiSemanticSurfaceIdentity,
);

#[derive(Debug)]
pub(super) struct UiServiceProposalOccupancyNeighborhoodIndex {
    neighborhoods: HashMap<NeighborhoodIndexKey, UiServiceProposalOccupancyNeighborhood>,
    live_records: usize,
    /// Neighborhood entries examined outside the operation's own
    /// `(application, semantic surface)` key. Keyed access adds nothing; only a
    /// full sweep does. An ordinary reserve, commit, or release must therefore
    /// leave this at zero, and any future path that replaces a keyed lookup with
    /// a sweep becomes visible here instead of silently amplifying.
    foreign_neighborhoods_examined: u64,
}

#[derive(Debug)]
pub(super) struct UiServiceProposalOccupancyNeighborhood {
    pub(super) records: Vec<super::UiServiceProposalOccupancyRecord>,
}

impl UiServiceProposalOccupancyNeighborhoodIndex {
    pub(super) fn new() -> Self {
        Self {
            neighborhoods: HashMap::new(),
            live_records: 0,
            foreign_neighborhoods_examined: 0,
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(super) const fn foreign_neighborhoods_examined(&self) -> u64 {
        self.foreign_neighborhoods_examined
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(super) fn neighborhood_count(&self) -> usize {
        self.neighborhoods.len()
    }

    /// Every full sweep charges the neighborhoods it examines beyond the one an
    /// equivalent keyed access would have touched.
    fn charge_sweep(&mut self) {
        self.foreign_neighborhoods_examined = self
            .foreign_neighborhoods_examined
            .saturating_add(self.neighborhoods.len().saturating_sub(1) as u64);
    }

    pub(super) fn find(
        &self,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<&UiServiceProposalOccupancyNeighborhood> {
        self.neighborhoods
            .get(&(application.clone(), semantic_surface))
    }

    pub(super) fn find_mut(
        &mut self,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<&mut UiServiceProposalOccupancyNeighborhood> {
        self.neighborhoods
            .get_mut(&(application.clone(), semantic_surface))
    }

    pub(super) fn find_or_insert(
        &mut self,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> &mut UiServiceProposalOccupancyNeighborhood {
        self.neighborhoods
            .entry((application, semantic_surface))
            .or_insert_with(|| UiServiceProposalOccupancyNeighborhood {
                records: Vec::new(),
            })
    }

    pub(super) const fn live_count(&self) -> usize {
        self.live_records
    }

    pub(super) fn record(
        &mut self,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        record: super::UiServiceProposalOccupancyRecord,
    ) {
        self.find_or_insert(application, semantic_surface)
            .records
            .push(record);
        self.live_records += 1;
    }

    pub(super) fn release(
        &mut self,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        proposal: super::super::UiServiceProposalIdentity,
    ) -> u16 {
        let key = (application.clone(), semantic_surface);
        let Some(neighborhood) = self.neighborhoods.get_mut(&key) else {
            return 0;
        };
        let before = neighborhood.records.len();
        neighborhood
            .records
            .retain(|record| record.proposal != proposal);
        let released = before - neighborhood.records.len();
        if neighborhood.records.is_empty() {
            self.neighborhoods.remove(&key);
        }
        self.live_records -= released;
        released as u16
    }

    pub(super) fn all_records(
        &self,
    ) -> impl Iterator<Item = &super::UiServiceProposalOccupancyRecord> {
        self.neighborhoods
            .values()
            .flat_map(|neighborhood| neighborhood.records.iter())
    }

    pub(super) fn proposal_count(&mut self) -> u16 {
        self.charge_sweep();
        let mut proposals = Vec::new();
        for record in self.all_records() {
            if !proposals.contains(&record.proposal) {
                proposals.push(record.proposal);
            }
        }
        proposals.len() as u16
    }

    pub(super) fn before_effect_summary(
        &mut self,
    ) -> (Vec<super::super::UiServiceProposalIdentity>, u16) {
        self.charge_sweep();
        let mut proposals = Vec::new();
        let mut leases = 0_u16;
        for record in self
            .all_records()
            .filter(|record| record.before_effect_open)
        {
            leases += 1;
            if !proposals.contains(&record.proposal) {
                proposals.push(record.proposal);
            }
        }
        (proposals, leases)
    }

    pub(super) fn abandon_before_effect(
        &mut self,
        proposals: &[super::super::UiServiceProposalIdentity],
    ) -> u16 {
        self.charge_sweep();
        let mut released = 0_usize;
        for neighborhood in self.neighborhoods.values_mut() {
            let before = neighborhood.records.len();
            neighborhood
                .records
                .retain(|record| !proposals.contains(&record.proposal));
            released += before - neighborhood.records.len();
        }
        self.neighborhoods
            .retain(|_, neighborhood| !neighborhood.records.is_empty());
        self.live_records -= released;
        released as u16
    }
}

#[cfg(test)]
mod tests {
    use super::NeighborhoodIndexKey;

    #[test]
    fn index_key_is_compile_time_exact_application_generation_and_surface() {
        fn require_axes(
            _: &(
                crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
                worth_ui_host_contract::UiSemanticSurfaceIdentity,
            ),
        ) {
        }
        let _ = require_axes as fn(&NeighborhoodIndexKey);
    }
}
