//! Original ordinals and persistent per-call dependency sets.
use super::super::super::retained_decision_facts::StoreDenial;
use super::*;
use worth_execution::{ExecutionAllocationPolicy, ExecutionOrderedMap};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ComputationFactIndex {
    pub(super) ordinals: std::sync::Arc<ExecutionOrderedMap<usize, WorthQueryApplicationFactKey>>,
    pub(super) calls: ExecutionOrderedMap<ComputationRead, Vec<WorthQueryApplicationFactKey>>,
    pub(super) observation_work: Option<u64>,
}
impl ComputationFactIndex {
    pub(super) fn build(
        facts: &BTreeMap<WorthQueryApplicationFactKey, SealedComputationFact>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        let mut ordinals = ExecutionOrderedMap::default();
        let mut grouped: BTreeMap<_, Vec<_>> = BTreeMap::new();
        let mut observation_work = Some(0u64);
        for (key, fact) in facts {
            super::super::super::super::fact_verification_observation::sealed_computation_index();
            observation_work = observation_work
                .zip(super::super::fact_observation::observation_work_bound(key))
                .and_then(|(a, b)| a.checked_add(b));
            ordinals
                .insert_cloned_in(&fact.ordinal, key, policy)
                .map_err(super::reattribution::storage_denial)?;
            for read in fact.readers.reads() {
                grouped.entry(read).or_default().push(key.clone());
            }
        }
        let mut calls = ExecutionOrderedMap::default();
        for (read, keys) in grouped {
            calls
                .insert_cloned_in(&read, &keys, policy)
                .map_err(super::reattribution::storage_denial)?;
        }
        Ok(Self {
            ordinals: std::sync::Arc::new(ordinals),
            calls,
            observation_work,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn keys(
        &self,
        read: ComputationRead,
    ) -> impl Iterator<Item = &WorthQueryApplicationFactKey> {
        self.calls.get(&read).into_iter().flatten()
    }
    pub(super) fn charged_bytes(&self) -> Option<u64> {
        (size_of::<Self>() as u64)
            .checked_add(self.ordinals.retained_bytes())?
            .checked_add(self.calls.retained_bytes())
    }
}
