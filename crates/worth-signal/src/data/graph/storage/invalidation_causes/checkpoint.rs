use std::collections::BTreeMap;

use serde::Deserialize;

use super::CanonicalCauseSetStore;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;

#[derive(Deserialize)]
struct CanonicalCauseSetStoreWire {
    #[serde(default)]
    generation: u32,
    #[serde(default)]
    sets: Vec<Vec<ResolvedDependencyCause>>,
    #[serde(default)]
    slot_generations: Vec<u32>,
    #[serde(default)]
    next_output_commit_ordinal: u64,
    #[serde(default)]
    published_output_commits: BTreeMap<u64, ProducedAspectDelta>,
}

impl<'de> Deserialize<'de> for CanonicalCauseSetStore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = CanonicalCauseSetStoreWire::deserialize(deserializer)?;
        let deserialized_quarantine = wire.sets.iter().any(|set| !set.is_empty());
        let mut free_indices = wire
            .sets
            .iter()
            .enumerate()
            .filter_map(|(index, set)| set.is_empty().then_some(index as u32))
            .collect::<Vec<_>>();
        free_indices.reverse();
        let slot_generations = (0..wire.sets.len())
            .map(|index| {
                wire.slot_generations
                    .get(index)
                    .copied()
                    .unwrap_or(wire.generation)
            })
            .collect();
        let max_published_ordinal = wire
            .published_output_commits
            .iter()
            .map(|(key, delta)| (*key).max(delta.output_commit_ordinal.0))
            .max()
            .unwrap_or_default();
        let mut store = Self {
            generation: wire.generation,
            sets: wire.sets.into_iter().collect(),
            slot_generations,
            free_indices: free_indices.into_iter().collect(),
            next_output_commit_ordinal: wire.next_output_commit_ordinal.max(max_published_ordinal),
            published_output_commits: wire.published_output_commits.into_iter().collect(),
            occupied_set_count: 0,
            output_commit_reference_counts: Default::default(),
            retained_custody: None,
            deserialized_quarantine,
            #[cfg(test)]
            published_order_probe: Vec::new(),
            #[cfg(test)]
            last_compaction_slot_visits: 0,
        };
        store.rebuild_derived_metadata();
        Ok(store)
    }
}

impl CanonicalCauseSetStore {
    pub(crate) fn readmit_graph_instance(&mut self, graph_instance: u64) {
        for cause in self.sets.iter_mut().flatten() {
            cause.key.graph_instance = graph_instance;
            cause.binding_axes.graph_instance = graph_instance;
        }
    }

    pub(crate) fn requires_readmission(&self) -> bool {
        self.deserialized_quarantine
    }

    pub(crate) fn complete_readmission(&mut self) {
        self.deserialized_quarantine = false;
    }
}

pub(crate) fn serialize_canonical_cause_sets<S>(
    store: &CanonicalCauseSetStore,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut canonical = store.clone();
    canonical.readmit_graph_instance(0);
    serde::Serialize::serialize(&canonical, serializer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialization_reconstructs_allocator_metadata_instead_of_trusting_it() {
        let payload = serde_json::json!({
            "generation": 0,
            "sets": [],
            "slot_generations": [],
            "free_indices": [999],
            "next_output_commit_ordinal": 0,
            "published_output_commits": {}
        });
        let mut store: CanonicalCauseSetStore = serde_json::from_value(payload).unwrap();
        let cause = crate::data::proof::invalidation::binding::ResolvedDependencyCause::new(
            1,
            crate::data::handle::NodeId::new(1, 0),
            crate::data::proof::invalidation::binding::DependencyRevision(0),
            crate::data::handle::NodeId::new(0, 0),
            crate::data::aspect::Aspect::new(0),
            None,
            0,
            crate::data::proof::invalidation::binding::OutputCommitOrdinal(1),
            1,
            crate::data::proof::PartitionScopeSet::default(),
        );

        let id = store.insert([cause]);

        assert_eq!(store.allocated_slot_count(), 1);
        assert_eq!(store.get(id).unwrap().len(), 1);
    }
}
