use std::collections::BTreeMap;
use std::path::Path;

use super::durable_frame::{read_u16, read_u64};
use crate::integrity_observation::child_expectation::{ChildExpectation, ChildScope};
use crate::integrity_observation::{
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIntegrityOutcome,
    OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause, OfflineUnknownPhysicalReason,
};
use worth_foundational::{
    PhysicalArtifactFamily, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
};

#[derive(Default)]
pub(crate) struct ArenaAccounting {
    roots: BTreeMap<(u64, String), ArenaRanges>,
    routed_generations: BTreeMap<String, Vec<(u64, u64, u64, u64, u64)>>,
    geometry: BTreeMap<u64, (u64, u64)>,
    forgotten: std::collections::BTreeSet<String>,
    historical_coverage: Vec<(String, u64, u64, u64, u64, u64, u64)>,
}

mod ranges;
use ranges::ArenaRanges;

impl ArenaAccounting {
    pub(crate) fn held_coverage(
        &mut self,
        path: String,
        offset: u64,
        length: u64,
        extent: u64,
        extent_generation: u64,
        first: u64,
        end: u64,
    ) {
        self.historical_coverage.push((
            path,
            offset,
            offset + length,
            extent,
            extent_generation,
            first,
            end,
        ));
    }
    pub(crate) fn forgotten(&mut self, path: String) {
        self.forgotten.insert(path);
    }
    /// A validated routing leaf is authoritative even when a later payload
    /// read fails or collides with a separately protected historical source.
    pub(crate) fn observe_routed_expectation(&mut self, root: u64, expected: &ChildExpectation) {
        let ChildScope::ExtentManifest {
            allocated_bytes,
            extent,
            ..
        } = expected.scope
        else {
            return;
        };
        self.roots
            .entry((root, expected.path.clone()))
            .or_default()
            .routed
            .push((expected.offset, expected.offset + allocated_bytes));
        self.routed_generations
            .entry(expected.path.clone())
            .or_default()
            .push((
                expected.offset,
                expected.offset + allocated_bytes,
                extent,
                expected.generation,
                root,
            ));
    }
    /// Consume only independently validated routing and free-membership bytes.
    pub(crate) fn observe(&mut self, root: u64, expected: &ChildExpectation, bytes: &[u8]) {
        match &expected.scope {
            ChildScope::FreeSpace { .. } => {
                self.geometry
                    .insert(root, (read_u64(bytes, 200), read_u64(bytes, 208)));
            }
            ChildScope::Tree { level: 0, .. }
                if expected.family == PhysicalArtifactFamily::FreeSpaceMembershipBlock =>
            {
                let count = usize::from(read_u16(bytes, 66));
                for entry in bytes[88..88 + count * 40].chunks_exact(40) {
                    if entry[0] == 2 {
                        let arena = read_u64(entry, 8);
                        let offset = read_u64(entry, 16);
                        let length = read_u64(entry, 24);
                        let path = format!("families/records/arenas/arena-{arena:016x}.data");
                        self.roots
                            .entry((root, path))
                            .or_default()
                            .free
                            .push((offset, offset + length));
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn finish(
        mut self,
        root: &Path,
        walk: &mut BoundedMediaWalk,
    ) -> Vec<OfflineArtifactObservation> {
        let mut observations = Vec::new();
        let newest_root = self
            .roots
            .keys()
            .map(|(generation, _)| *generation)
            .max()
            .unwrap_or(0);
        let selected_free = self
            .roots
            .iter()
            .filter_map(|((generation, path), ranges)| {
                (*generation == newest_root).then_some((path.clone(), ranges.free.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        for ((generation, path), ranges) in &mut self.roots {
            ranges
                .historical
                .extend(self.historical_coverage.iter().filter_map(
                    |(owner, start, end, _, _, first, last)| {
                        (owner == path && *first <= *generation && *generation < *last)
                            .then_some((*start, *end))
                    },
                ));
            if *generation < newest_root {
                // Once WAL/checkpoint retirement evidence is reclaimed, a
                // predecessor's gap may be later free. This is coverage, not
                // proof of the missing historical transition: a gap that needs
                // it is Unknown rather than falsely Intact.
                if let Some(free) = selected_free.get(path) {
                    ranges.uncertain.extend(free.iter().copied());
                }
            }
            if *generation == newest_root {
                if let Some(all_routes) = self.routed_generations.get(path) {
                    ranges
                        .routed
                        .extend(all_routes.iter().map(|&(start, end, _, _, _)| (start, end)));
                }
            }
        }
        for ((generation, path), mut ranges) in self.roots {
            if self.forgotten.contains(&path) && ranges.routed.is_empty() {
                continue;
            }
            let physical = root.join(&path);
            let geometry = self.geometry.get(&generation).copied();
            if ranges.routed.is_empty()
                && matches!(physical.try_exists(), Ok(false))
                && geometry.is_some_and(|(capacity, alignment)| {
                    ranges.geometry_damage(capacity, alignment).is_none()
                        && ranges.inspect(capacity).is_none()
                        && !ranges.has_unproven_gap(capacity)
                })
            {
                continue;
            }
            if ranges.routed.is_empty()
                && matches!(physical.try_exists(), Ok(false))
                && geometry.is_some_and(|(capacity, alignment)| {
                    ranges.geometry_damage(capacity, alignment).is_none()
                        && ranges.inspect(capacity).is_none()
                        && ranges.has_unproven_gap(capacity)
                })
            {
                let outcome = OfflineIntegrityOutcome::Unknown(
                    OfflineUnknownPhysicalReason::ParentScopeUnavailable,
                );
                walk.record_outcome(&outcome);
                observations.push(OfflineArtifactObservation::new(
                    path.clone(),
                    PhysicalArtifactFamily::ExtentArenaFrame.into(),
                    PhysicalArtifactIdentity::new(format!("arena-accounting:{generation}:{path}"))
                        .unwrap(),
                    PhysicalArtifactGeneration::encoded(generation).unwrap(),
                    None,
                    outcome,
                ));
                continue;
            }
            let acquired = walk.acquire_range(&physical, 4, 0, 0);
            let outcome = match acquired {
                Err(outcome) => outcome,
                Ok(acquired) => {
                    let length = acquired.byte_length as u64;
                    let identity_overlap = self
                        .routed_generations
                        .get(&path)
                        .and_then(|routes| routed_identity_overlap(generation, routes));
                    let held_overlap = held_identity_overlap(
                        generation,
                        &path,
                        self.routed_generations.get(&path),
                        &self.historical_coverage,
                    );
                    let geometry_damage = geometry.and_then(|(capacity, alignment)| {
                        if length > capacity {
                            Some((capacity, length))
                        } else {
                            ranges
                                .geometry_damage(capacity, alignment)
                                .or_else(|| ranges.inspect(capacity))
                        }
                    });
                    let uncertain_gap =
                        !ranges.uncertain.is_empty() && ranges.has_unproven_gap(length);
                    let outcome = held_overlap
                        .or(identity_overlap)
                        .or(geometry_damage)
                        .map_or_else(
                            || {
                                if uncertain_gap {
                                    OfflineIntegrityOutcome::Unknown(
                                        OfflineUnknownPhysicalReason::ParentScopeUnavailable,
                                    )
                                } else {
                                    OfflineIntegrityOutcome::Intact
                                }
                            },
                            |(start, end)| {
                                crate::integrity_observation::record_walk::damage(
                                    OfflinePhysicalDamageCause::ScopeMismatch,
                                    Some((start, end.saturating_sub(start).max(1))),
                                    OfflinePhysicalBlastRadius::Artifact,
                                )
                            },
                        );
                    outcome
                }
            };
            walk.record_outcome(&outcome);
            observations.push(OfflineArtifactObservation::new(
                path.clone(),
                PhysicalArtifactFamily::ExtentArenaFrame.into(),
                PhysicalArtifactIdentity::new(format!("arena-accounting:{generation}:{path}"))
                    .unwrap(),
                PhysicalArtifactGeneration::encoded(generation).unwrap(),
                // Accounting belongs to this root's graph, not one frame range.
                None,
                outcome,
            ));
        }
        observations
    }
}

fn routed_identity_overlap(root: u64, routes: &[(u64, u64, u64, u64, u64)]) -> Option<(u64, u64)> {
    // A physical range may be released and legitimately reused by a later
    // root.  Only routes coexisting in this root can collide.
    let mut routes = routes
        .iter()
        .filter_map(|&(start, end, extent, generation, owner)| {
            (owner == root).then_some((start, end, extent, generation))
        })
        .collect::<Vec<_>>();
    routes.sort_unstable();
    routes.dedup();
    for pair in routes.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Some((pair[1].0, pair[0].1.min(pair[1].1)));
        }
    }
    None
}

fn held_identity_overlap(
    generation: u64,
    path: &str,
    routes: Option<&Vec<(u64, u64, u64, u64, u64)>>,
    holds: &[(String, u64, u64, u64, u64, u64, u64)],
) -> Option<(u64, u64)> {
    let active = holds
        .iter()
        .filter(|(owner, _, _, _, _, first, last)| {
            owner == path && *first <= generation && generation < *last
        })
        .collect::<Vec<_>>();
    for (index, hold) in active.iter().enumerate() {
        let &(_, start, end, extent, extent_generation, _, _) = *hold;
        for &(route_start, route_end, route_extent, route_generation, root) in
            routes.into_iter().flatten()
        {
            if root != generation || start >= route_end || route_start >= end {
                continue;
            }
            if (route_start, route_end, route_extent, route_generation)
                != (start, end, extent, extent_generation)
            {
                return Some((start.max(route_start), end.min(route_end)));
            }
        }
        for other in active.iter().skip(index + 1) {
            let &(_, other_start, other_end, other_extent, other_generation, _, _) = *other;
            if start < other_end
                && other_start < end
                && (start, end, extent, extent_generation)
                    != (other_start, other_end, other_extent, other_generation)
            {
                return Some((start.max(other_start), end.min(other_end)));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests;
