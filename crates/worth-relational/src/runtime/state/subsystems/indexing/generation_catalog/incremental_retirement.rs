//! Publication delta candidates and a finite, fair cycle over older generations.

use std::collections::BTreeSet;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::sync::Arc;

use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::VersionId;
use crate::indexes::data::{DerivedIndexGeneration, DerivedIndexGenerationId};

use super::GenerationCatalog;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appending_generations_cannot_starve_a_previously_visited_held_prefix() {
        let mut catalog = GenerationCatalog::default();
        for id in 1..=40 {
            catalog.insert(super::super::tests::generation(id, 1, 0, id, 1, true));
        }
        assert!(catalog
            .next_retirement_visits(3)
            .contains(&DerivedIndexGenerationId(1)));
        let mut revisited = false;
        for round in 0..16 {
            for offset in 1..=10 {
                let id = 40 + round * 10 + offset;
                catalog.insert(super::super::tests::generation(id, 1, 0, id, 1, true));
            }
            let visits = catalog.next_retirement_visits(3);
            assert!(
                visits.len() <= 3,
                "held entries also consume the sweep quota"
            );
            if visits.contains(&DerivedIndexGenerationId(1)) {
                revisited = true;
                break;
            }
        }
        assert!(
            revisited,
            "newer arrivals must not postpone an old hold's next eligibility check"
        );
    }
}

impl GenerationCatalog {
    pub(in crate::runtime::state::subsystems::indexing) fn retire_after_publication(
        &mut self,
        publication: &CanonicalCommitEnvelope,
        previous_versions: &BTreeSet<VersionId>,
        maximum_older_visits: usize,
        mut custody: impl FnMut(&DerivedIndexGeneration) -> (bool, bool, bool),
    ) -> Vec<Arc<DerivedIndexGeneration>> {
        let mut candidates = BTreeSet::new();
        for version in previous_versions {
            if let Some(ids) = self.versions.get(version) {
                candidates.extend(ids.iter().copied());
            }
        }
        for id in self
            .commits
            .get(&publication.commit.commit_id)
            .into_iter()
            .flatten()
            .copied()
        {
            let Some(generation) = self.entries.get(&id) else {
                continue;
            };
            candidates.insert(id);
            if let Some(previous) = self
                .scope(generation.index_id, Some(&generation.source_branch_id))
                .and_then(|scope| scope.predecessor(id))
            {
                candidates.insert(previous);
            }
        }
        let older = self.next_retirement_visits(maximum_older_visits);
        self.work.enumerate_inventory(older.len());
        candidates.extend(older);
        let mut retired = Vec::new();
        for id in candidates {
            let Some(generation) = self.entries.get(&id) else {
                continue;
            };
            let (global, basis_held, source_is_live) = custody(generation);
            if self.selected_for_retention(id, generation, global, basis_held, source_is_live) {
                continue;
            }
            if let Some(generation) = self.entries.remove(&id) {
                self.remove_bindings(&generation);
                retired.push(generation);
            }
        }
        retired
    }

    fn next_retirement_visits(&mut self, maximum: usize) -> Vec<DerivedIndexGenerationId> {
        if maximum == 0 {
            return Vec::new();
        }
        if self.retirement_cycle_end.is_none() {
            self.retirement_cycle_end = self.entries.last_key_value().map(|(id, _)| *id);
        }
        let Some(end) = self.retirement_cycle_end else {
            return Vec::new();
        };
        let start = self.retirement_cursor.map(Excluded).unwrap_or(Unbounded);
        let visits: Vec<_> = self
            .entries
            .range((start, Included(end)))
            .take(maximum)
            .map(|(id, _)| *id)
            .collect();
        if let Some(last) = visits.last().copied() {
            self.retirement_cursor = Some(last);
        }
        let next = self.retirement_cursor.map(Excluded).unwrap_or(Unbounded);
        if self.entries.range((next, Included(end))).next().is_none() {
            self.retirement_cursor = None;
            self.retirement_cycle_end = None;
        }
        visits
    }
}
