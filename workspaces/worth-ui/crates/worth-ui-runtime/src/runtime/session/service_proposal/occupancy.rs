mod lease;
mod neighborhood;
mod release;
#[cfg(test)]
mod tests;

use lease::UiServiceProposalOccupancyKey;
pub(in crate::runtime) use lease::{
    UiServiceProposalOccupancyLease, UiServiceProposalOccupancyScopeIdentity,
};

const OCCUPANCY_LIMIT: usize = 384;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum UiServiceProposalConflictDisposition {
    #[cfg(test)]
    Occupied,
    #[cfg(test)]
    Superseded,
    #[cfg(test)]
    Coalesced,
    #[cfg(test)]
    CancelledBeforeEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(test)]
pub(in crate::runtime) enum UiServiceProposalConflictPolicy {
    RejectOccupied,
    #[cfg(test)]
    SupersedeBeforeEffect,
    #[cfg(test)]
    CoalesceExact,
    #[cfg(test)]
    CancelBeforeEffect,
}

#[derive(Debug)]
pub(super) struct UiServiceProposalOccupancyRecord {
    key: UiServiceProposalOccupancyKey,
    proposal: super::UiServiceProposalIdentity,
    #[cfg(test)]
    requirements: u8,
    #[cfg(test)]
    fact_references: u16,
    #[cfg(test)]
    mounted_work_references: u16,
    #[cfg(test)]
    coherence: super::UiServiceRequestCoherence,
    slot_generation: u64,
    before_effect_open: bool,
}

#[derive(Debug)]
pub(super) struct UiServiceProposalOccupancyTable {
    neighborhoods: neighborhood::UiServiceProposalOccupancyNeighborhoodIndex,
    next_slot_generation: u64,
    work_counters: UiServiceProposalOccupancyWorkCounters,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::runtime) struct UiServiceProposalOccupancyWorkCounters {
    proposal_requirements_visited: u64,
    unrelated_neighborhoods_touched: u64,
}

pub(super) struct UiServiceProposalOccupancyPlan {
    keys: Box<[UiServiceProposalOccupancyKey]>,
    displacement: Option<UiServiceProposalDisplacement>,
    coalesced: Option<super::UiServiceProposalIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiServiceProposalDisplacement {
    proposal: super::UiServiceProposalIdentity,
    disposition: UiServiceProposalConflictDisposition,
    released_leases: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiServiceProposalOccupancyDenial {
    Occupied(super::UiServiceProposalIdentity),
    AmbiguousConflict,
    CapacityExceeded,
    SlotGenerationExhausted,
    #[cfg(test)]
    BeforeEffectWindowClosed(super::UiServiceProposalIdentity),
}

impl UiServiceProposalOccupancyPlan {
    pub(super) const fn displacement(&self) -> Option<UiServiceProposalDisplacement> {
        self.displacement
    }
}

impl UiServiceProposalDisplacement {
    pub(in crate::runtime) const fn proposal(self) -> super::UiServiceProposalIdentity {
        self.proposal
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn disposition(self) -> UiServiceProposalConflictDisposition {
        self.disposition
    }

    pub(super) const fn released_leases(self) -> u16 {
        self.released_leases
    }
}

impl UiServiceProposalOccupancyTable {
    pub(super) fn new() -> Self {
        Self {
            neighborhoods: neighborhood::UiServiceProposalOccupancyNeighborhoodIndex::new(),
            next_slot_generation: 1,
            work_counters: UiServiceProposalOccupancyWorkCounters {
                proposal_requirements_visited: 0,
                unrelated_neighborhoods_touched: 0,
            },
        }
    }

    pub(super) fn plan(
        &mut self,
        candidate: &super::UiServiceProposalCandidate,
    ) -> Result<UiServiceProposalOccupancyPlan, UiServiceProposalOccupancyDenial> {
        let keys = candidate
            .family_proposals()
            .iter()
            .map(|family| UiServiceProposalOccupancyKey {
                application: candidate.application().clone(),
                semantic_surface: candidate.surface().semantic_surface(),
                family: family.family(),
                scope: family.scope(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        #[cfg(test)]
        let mut conflict = None;
        #[cfg(test)]
        let mut disposition = None;
        #[cfg(test)]
        let mut coalesced = false;
        self.work_counters.proposal_requirements_visited += keys.len() as u64;
        let neighborhood = self.neighborhoods.find(
            candidate.application(),
            candidate.surface().semantic_surface(),
        );
        for (key, _family) in keys.iter().zip(candidate.family_proposals()) {
            let existing = neighborhood.and_then(|neighborhood| {
                neighborhood
                    .records
                    .iter()
                    .find(|record| record.key == *key)
            });
            let Some(existing) = existing else {
                #[cfg(test)]
                if coalesced {
                    return Err(UiServiceProposalOccupancyDenial::AmbiguousConflict);
                }
                continue;
            };
            #[cfg(not(test))]
            {
                return Err(UiServiceProposalOccupancyDenial::Occupied(
                    existing.proposal,
                ));
            }
            #[cfg(test)]
            let next = match _family.conflict_policy() {
                UiServiceProposalConflictPolicy::RejectOccupied => {
                    return Err(UiServiceProposalOccupancyDenial::Occupied(
                        existing.proposal,
                    ));
                }
                #[cfg(test)]
                UiServiceProposalConflictPolicy::SupersedeBeforeEffect => {
                    if !existing.before_effect_open {
                        return Err(UiServiceProposalOccupancyDenial::BeforeEffectWindowClosed(
                            existing.proposal,
                        ));
                    }
                    UiServiceProposalConflictDisposition::Superseded
                }
                #[cfg(test)]
                UiServiceProposalConflictPolicy::CoalesceExact => {
                    coalesced = true;
                    UiServiceProposalConflictDisposition::Coalesced
                }
                #[cfg(test)]
                UiServiceProposalConflictPolicy::CancelBeforeEffect => {
                    if !existing.before_effect_open {
                        return Err(UiServiceProposalOccupancyDenial::BeforeEffectWindowClosed(
                            existing.proposal,
                        ));
                    }
                    UiServiceProposalConflictDisposition::CancelledBeforeEffect
                }
            };
            #[cfg(test)]
            if conflict.is_some_and(|current| current != existing.proposal)
                || disposition.is_some_and(|current| current != next)
            {
                return Err(UiServiceProposalOccupancyDenial::AmbiguousConflict);
            }
            #[cfg(test)]
            {
                conflict = Some(existing.proposal);
                disposition = Some(next);
            }
        }
        #[cfg(test)]
        if coalesced {
            let incumbent = conflict.ok_or(UiServiceProposalOccupancyDenial::AmbiguousConflict)?;
            if keys
                .iter()
                .zip(candidate.family_proposals())
                .any(|(key, family)| {
                    !neighborhood
                        .into_iter()
                        .flat_map(|neighborhood| &neighborhood.records)
                        .any(|record| {
                            record.key == *key
                                && record.proposal == incumbent
                                && record.requirements == family.requirements()
                                && record.fact_references == family.fact_references()
                                && record.mounted_work_references
                                    == family.mounted_work_references()
                                && record.coherence.eq(candidate.coherence())
                        })
                })
                || neighborhood.map_or(0, |neighborhood| {
                    neighborhood
                        .records
                        .iter()
                        .filter(|record| record.proposal == incumbent)
                        .count()
                }) != keys.len()
            {
                return Err(UiServiceProposalOccupancyDenial::AmbiguousConflict);
            }
            return Ok(UiServiceProposalOccupancyPlan {
                keys,
                displacement: None,
                coalesced: Some(incumbent),
            });
        }
        #[cfg(test)]
        let displacement = conflict.map(|proposal| UiServiceProposalDisplacement {
            proposal,
            disposition: disposition.expect("conflict disposition accompanies proposal"),
            released_leases: neighborhood
                .into_iter()
                .flat_map(|neighborhood| &neighborhood.records)
                .filter(|record| record.proposal == proposal)
                .count() as u16,
        });
        #[cfg(not(test))]
        let displacement: Option<UiServiceProposalDisplacement> = None;
        let released = displacement.map_or(0, |record| usize::from(record.released_leases));
        if self.neighborhoods.live_count() - released + keys.len() > OCCUPANCY_LIMIT {
            return Err(UiServiceProposalOccupancyDenial::CapacityExceeded);
        }
        self.next_slot_generation
            .checked_add(keys.len() as u64)
            .ok_or(UiServiceProposalOccupancyDenial::SlotGenerationExhausted)?;
        Ok(UiServiceProposalOccupancyPlan {
            keys,
            displacement,
            coalesced: None,
        })
    }

    pub(super) fn commit(
        &mut self,
        candidate: &super::UiServiceProposalCandidate,
        plan: UiServiceProposalOccupancyPlan,
    ) -> (
        Box<[UiServiceProposalOccupancyLease]>,
        Option<UiServiceProposalDisplacement>,
    ) {
        let proposal = candidate.identity();
        debug_assert!(plan.coalesced.is_none());
        if let Some(displacement) = plan.displacement {
            self.neighborhoods.release(
                candidate.application(),
                candidate.surface().semantic_surface(),
                displacement.proposal,
            );
        }
        let application = candidate.application().clone();
        let semantic_surface = candidate.surface().semantic_surface();
        let mut next_slot_generation = self.next_slot_generation;
        self.next_slot_generation += plan.keys.len() as u64;
        let mut leases = Vec::with_capacity(plan.keys.len());
        for (key, _family) in plan.keys.iter().cloned().zip(candidate.family_proposals()) {
            let slot_generation = next_slot_generation;
            next_slot_generation += 1;
            leases.push(UiServiceProposalOccupancyLease {
                key: key.clone(),
                proposal,
                slot_generation,
            });
            self.neighborhoods.record(
                application.clone(),
                semantic_surface,
                UiServiceProposalOccupancyRecord {
                    key,
                    proposal,
                    #[cfg(test)]
                    requirements: _family.requirements(),
                    #[cfg(test)]
                    fact_references: _family.fact_references(),
                    #[cfg(test)]
                    mounted_work_references: _family.mounted_work_references(),
                    #[cfg(test)]
                    coherence: candidate.coherence().clone(),
                    slot_generation,
                    before_effect_open: true,
                },
            );
        }
        (leases.into_boxed_slice(), plan.displacement)
    }

    pub(super) fn coalesced(
        plan: &UiServiceProposalOccupancyPlan,
    ) -> Option<super::UiServiceProposalIdentity> {
        plan.coalesced
    }

    pub(super) fn live_count(&self) -> usize {
        self.neighborhoods.live_count()
    }

    pub(super) fn neighborhood_count(&self) -> usize {
        self.neighborhoods.neighborhood_count()
    }

    /// Counters are read from the index rather than mirrored, so a sweep charged
    /// inside the index cannot be lost on the way out.
    pub(super) fn work_counters(&self) -> UiServiceProposalOccupancyWorkCounters {
        UiServiceProposalOccupancyWorkCounters {
            proposal_requirements_visited: self.work_counters.proposal_requirements_visited,
            unrelated_neighborhoods_touched: self.neighborhoods.foreign_neighborhoods_examined(),
        }
    }

    pub(super) fn proposal_count(&mut self) -> u16 {
        self.neighborhoods.proposal_count()
    }

    pub(super) fn before_effect_summary(&mut self) -> (Vec<super::UiServiceProposalIdentity>, u16) {
        self.neighborhoods.before_effect_summary()
    }

    pub(super) fn abandon_before_effect(
        &mut self,
        proposals: &[super::UiServiceProposalIdentity],
    ) -> u16 {
        self.neighborhoods.abandon_before_effect(proposals)
    }
}

impl UiServiceProposalOccupancyWorkCounters {
    pub(in crate::runtime) const fn proposal_requirements_visited(self) -> u64 {
        self.proposal_requirements_visited
    }

    pub(in crate::runtime) const fn unrelated_neighborhoods_touched(self) -> u64 {
        self.unrelated_neighborhoods_touched
    }
}
