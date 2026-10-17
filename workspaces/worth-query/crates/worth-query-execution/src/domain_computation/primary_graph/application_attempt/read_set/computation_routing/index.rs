//! Immutable ordinal alignment and call keys, built with each completed read.
use super::*;
use worth_execution::ChargedBytes;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ComputationFactIndex {
    pub(super) ordinals: BTreeMap<usize, WorthQueryApplicationFactKey>,
    calls: BTreeMap<ComputationRead, Vec<WorthQueryApplicationFactKey>>,
    pub(super) observation_work: Option<u64>,
}
impl ComputationFactIndex {
    pub(super) fn build<'a>(
        facts: &BTreeMap<WorthQueryApplicationFactKey, SealedComputationFact>,
        ordered: impl Iterator<Item = &'a WorthQueryApplicationFactKey>,
    ) -> Self {
        let ordinals = ordered
            .enumerate()
            .filter(|(_, key)| facts.contains_key(*key))
            .map(|(ordinal, key)| (ordinal, key.clone()))
            .collect();
        let mut calls: BTreeMap<_, Vec<_>> = BTreeMap::new();
        let mut observation_work = Some(0_u64);
        for (key, fact) in facts {
            super::super::super::super::fact_verification_observation::sealed_computation_index();
            observation_work = observation_work
                .zip(super::super::fact_observation::observation_work_bound(key))
                .and_then(|(sum, work)| sum.checked_add(work));
            for read in fact.readers.reads() {
                calls.entry(read).or_default().push(key.clone());
            }
        }
        Self {
            ordinals,
            calls,
            observation_work,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn keys(
        &self,
        read: ComputationRead,
    ) -> impl Iterator<Item = &WorthQueryApplicationFactKey> {
        self.calls.get(&read).into_iter().flatten()
    }
    pub(super) fn charged_bytes(&self) -> Option<u64> {
        let mut bytes = u64::try_from(size_of::<Self>() + 2 * size_of::<usize>()).ok()?;
        for key in self.ordinals.values() {
            bytes = bytes
                .checked_add(
                    (11 * size_of::<(usize, WorthQueryApplicationFactKey)>()
                        + 16 * size_of::<usize>()) as u64,
                )?
                .checked_add(key.additional_charged_bytes())?;
        }
        for keys in self.calls.values() {
            bytes = bytes
                .checked_add(
                    (11 * size_of::<(ComputationRead, Vec<WorthQueryApplicationFactKey>)>()
                        + 16 * size_of::<usize>()) as u64,
                )?
                .checked_add(
                    (keys.capacity() * size_of::<WorthQueryApplicationFactKey>()) as u64,
                )?;
            for key in keys {
                bytes = bytes.checked_add(key.additional_charged_bytes())?;
            }
        }
        Some(bytes)
    }
}
