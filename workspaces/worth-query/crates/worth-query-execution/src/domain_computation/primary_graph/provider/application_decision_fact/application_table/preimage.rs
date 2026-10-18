//! Targeted preimage views retain their transient projection allocation.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
use worth_execution::{ExecutionAllocationPolicy, ExecutionMemoryReservation};

pub(in crate::domain_computation::primary_graph) struct ProjectedPreImageCandidates {
    facts: Vec<Decision>,
    _memory: Option<ExecutionMemoryReservation>,
    examined: usize,
}
impl ProjectedPreImageCandidates {
    pub(in crate::domain_computation::primary_graph) fn iter(
        &self,
    ) -> impl Iterator<Item = &Decision> {
        self.facts.iter()
    }
    pub(in crate::domain_computation::primary_graph) fn examined(&self) -> usize {
        self.examined
    }
}
impl ApplicationFactTable {
    pub(in crate::domain_computation::primary_graph) fn preimage_candidates(
        &self,
        demand: &worth_query_installation::facade::InstalledPreImageDemand,
        mutation: &worth_relational::facade::mvcc::ValidatedMutationFootprint,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<ProjectedPreImageCandidates, StoreDenial> {
        let mut count = Some(self.count);
        if let Some(prefix) = &self.prefix {
            count = Some(0usize);
            for record in mutation.records() {
                let worth_relational::facade::transactions::RecordRef::Entity(entity) = record
                else {
                    continue;
                };
                for locus in demand.loci() {
                    count = count.and_then(|n| {
                        n.checked_add(prefix.projection.preimage_ordinals(*entity, locus).len())
                    });
                }
            }
        }
        // B-tree set slack plus both vector growth and current-body views.
        let bytes = count
            .and_then(|n| {
                (n as u64).checked_mul(
                    (11 * std::mem::size_of::<usize>()
                        + 16 * std::mem::size_of::<usize>()
                        + 2 * std::mem::size_of::<Decision>()) as u64,
                )
            })
            .ok_or(StoreDenial::Representability)?;
        let memory = policy.reserve_memory(bytes);
        policy.check_live()?;
        let memory = memory?;
        let Some(prefix) = &self.prefix else {
            return Ok(ProjectedPreImageCandidates {
                facts: self.values().collect(),
                _memory: memory,
                examined: self.count,
            });
        };
        let mut selected = std::collections::BTreeSet::new();
        for record in mutation.records() {
            policy.check_live()?;
            let worth_relational::facade::transactions::RecordRef::Entity(entity) = record else {
                continue;
            };
            for locus in demand.loci() {
                selected.extend(
                    prefix
                        .projection
                        .preimage_ordinals(*entity, locus)
                        .iter()
                        .copied(),
                );
            }
        }
        let facts: Vec<_> = selected
            .into_iter()
            .filter_map(|ordinal| self.get(prefix.projection.locator(ordinal)?))
            .collect();
        policy.check_live()?;
        Ok(ProjectedPreImageCandidates {
            examined: facts.len(),
            facts,
            _memory: memory,
        })
    }
}
