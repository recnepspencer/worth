//! Held-name resolution over the branch-local derived name index.
//!
//! A live held name answers from its own index entry. A name that is no
//! longer live at the basis is followed forward only along the edges
//! recorded at the commit where it ended, and each reached name repeats the
//! same rule at the same basis. Nothing outside that lineage path is read.

use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::{
    PersistentNameKey, PersistentNameResolution, PersistentNameResolutionDenial,
    PersistentNameResolutionFootprint, PersistentNameResolutionMetrics,
    PersistentNameResolutionOutcome, PersistentNameSelection, PersistentNameTarget,
};
use worth_foundational::facade::AggregateIdentity;

use crate::lineage::index::{LiveName, NameEntry, NameState, PersistentNameBranchIndex};

/// One live descendant and the birth side it was named with.
struct LiveCandidate<'index> {
    live: &'index LiveName,
    side: Option<&'index [u8]>,
}

type Resolution = Result<PersistentNameResolution, PersistentNameResolutionDenial>;

impl PersistentNameBranchIndex {
    /// Resolves `held` at `basis` under `selection`.
    ///
    /// Complexity: O(1) probes for an empty filter, a pruned basis, an unknown
    /// name, or a name live at `basis`; otherwise O(p log p) in the forward lineage path `p`
    /// from `held` to its live or retired frontier at `basis`. Each probe also
    /// pays O(log t) in that one name's own timeline. Total names and
    /// unrelated commits never enter the cost.
    pub fn resolve(
        &self,
        held: PersistentNameKey,
        basis: CommitId,
        selection: &PersistentNameSelection,
    ) -> PersistentNameResolutionOutcome {
        let mut metrics = PersistentNameResolutionMetrics::default();
        let mut ended = Ended::default();
        let mut live = Vec::new();
        let result =
            self.resolve_counted(held, basis, selection, &mut metrics, &mut ended, &mut live);
        PersistentNameResolutionOutcome {
            result,
            metrics,
            footprint: PersistentNameResolutionFootprint {
                live,
                ended_lineages: ended.lineages.into_iter().collect(),
                ended_groups: ended.groups.into_iter().collect(),
            },
        }
    }

    fn resolve_counted(
        &self,
        held: PersistentNameKey,
        basis: CommitId,
        selection: &PersistentNameSelection,
        metrics: &mut PersistentNameResolutionMetrics,
        footprint: &mut Ended,
        frontier: &mut Vec<PersistentNameTarget>,
    ) -> Resolution {
        let empty_filter = match selection {
            PersistentNameSelection::BoundedByCarriers { carriers } => carriers.is_empty(),
            PersistentNameSelection::KeepSide { discriminator } => discriminator.is_empty(),
            PersistentNameSelection::Exact | PersistentNameSelection::AllDescendants => false,
        };
        if empty_filter {
            return Err(PersistentNameResolutionDenial::EmptySelectionFilter);
        }
        if let Some(retention_floor) = self.retention_floor()
            && basis < retention_floor
        {
            return Err(PersistentNameResolutionDenial::BasisPruned {
                basis,
                retention_floor,
            });
        }
        metrics.index_probes += 1;
        let unknown = PersistentNameResolutionDenial::UnknownName { name: held };
        let entry = self.names.get(&held).ok_or(unknown.clone())?;
        let (since, state) = entry.state_at(basis).ok_or(unknown)?;
        let mut live = BTreeMap::new();
        let retired_at = match state {
            NameState::Live(name) => {
                live.insert(held, candidate(entry, name));
                since
            }
            NameState::Ended => {
                let walk = ForwardWalk {
                    basis,
                    metrics,
                    footprint,
                };
                self.walk_forward(held, entry, since, &mut live, walk)
            }
        };
        frontier.extend(live.values().map(|found| found.live.target));
        select(&live, retired_at, selection)
    }

    /// Breadth-first walk from an ended name, collecting live descendants at
    /// `basis`; returns the latest commit at which a reached name ended.
    fn walk_forward<'index>(
        &'index self,
        held: PersistentNameKey,
        entry: &'index NameEntry,
        ended_at: CommitId,
        live: &mut BTreeMap<PersistentNameKey, LiveCandidate<'index>>,
        walk: ForwardWalk<'_>,
    ) -> CommitId {
        let ForwardWalk {
            basis,
            metrics,
            footprint,
        } = walk;
        let mut retired_at = ended_at;
        let mut visited = HashSet::from([held]);
        let mut frontier = VecDeque::from([(held, entry, ended_at)]);
        while let Some((key, entry, ended_at)) = frontier.pop_front() {
            metrics.event_visits += 1;
            footprint.lineages.extend(entry.lineage_ended_at(ended_at));
            if let PersistentNameKey::Group(group) = key {
                footprint.groups.insert(group);
            }
            retired_at = retired_at.max(ended_at);
            for successor in entry.edges_at(ended_at) {
                metrics.edge_visits += 1;
                if !visited.insert(*successor) {
                    continue;
                }
                metrics.index_probes += 1;
                let Some(next) = self.names.get(successor) else {
                    continue;
                };
                match next.state_at(basis) {
                    Some((_, NameState::Live(name))) => {
                        live.insert(*successor, candidate(next, name));
                    }
                    Some((since, NameState::Ended)) => {
                        frontier.push_back((*successor, next, since));
                    }
                    None => {}
                }
            }
        }
        retired_at
    }
}

/// What a forward walk reads at and records into.
struct ForwardWalk<'walk> {
    basis: CommitId,
    metrics: &'walk mut PersistentNameResolutionMetrics,
    footprint: &'walk mut Ended,
}

/// What ended along a walked path: record lineages and group names.
#[derive(Default)]
struct Ended {
    lineages: BTreeSet<LineageId>,
    groups: BTreeSet<AggregateIdentity>,
}

fn candidate<'index>(entry: &'index NameEntry, live: &'index LiveName) -> LiveCandidate<'index> {
    LiveCandidate {
        live,
        side: entry.side(),
    }
}

/// Applies `selection` to the live descendants, in canonical key order.
fn select(
    live: &BTreeMap<PersistentNameKey, LiveCandidate<'_>>,
    retired_at: CommitId,
    selection: &PersistentNameSelection,
) -> Resolution {
    if live.is_empty() {
        return Ok(PersistentNameResolution::Retired {
            at_commit: retired_at,
        });
    }
    let matches: Vec<(&PersistentNameKey, &LiveCandidate<'_>)> = match selection {
        PersistentNameSelection::Exact => {
            return single_or(live.iter().collect(), |keys| {
                PersistentNameResolutionDenial::Ambiguous { candidates: keys }
            });
        }
        PersistentNameSelection::AllDescendants => {
            let mut targets: Vec<_> = live.values().map(|found| found.live.target).collect();
            return Ok(match targets.len() {
                1 => PersistentNameResolution::Exact(targets.remove(0)),
                _ => PersistentNameResolution::Plural(targets),
            });
        }
        PersistentNameSelection::BoundedByCarriers { carriers } => live
            .iter()
            .filter(|(_, found)| carriers.iter().all(|c| found.live.carriers.contains(c)))
            .collect(),
        PersistentNameSelection::KeepSide { discriminator } => live
            .iter()
            .filter(|(_, found)| found.side == Some(discriminator.as_slice()))
            .collect(),
    };
    if matches.is_empty() {
        return Err(PersistentNameResolutionDenial::NoMatchingCandidate {
            live_candidates: live.keys().copied().collect(),
        });
    }
    single_or(matches, |keys| PersistentNameResolutionDenial::Ambiguous {
        candidates: keys,
    })
}

/// One match resolves `Exact`; several refuse with every candidate.
fn single_or(
    matches: Vec<(&PersistentNameKey, &LiveCandidate<'_>)>,
    refuse: impl FnOnce(Vec<PersistentNameKey>) -> PersistentNameResolutionDenial,
) -> Resolution {
    match matches.as_slice() {
        [(_, only)] => Ok(PersistentNameResolution::Exact(only.live.target)),
        _ => Err(refuse(matches.iter().map(|(key, _)| **key).collect())),
    }
}
