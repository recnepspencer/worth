//! Selected-root-only derived index walk. Record identities are resolved through
//! independently validated C.5 routing and page membership, never by scanning.
mod bounds;
mod catalog_target;
mod directory;
mod resolver;

use std::{
    collections::{BTreeMap, VecDeque},
    path::Path,
};

use super::{
    families::{
        index::inspect_btree_node_payload, physical_fields::record_key,
        root_manifest::OfflineRootManifestFacts,
    },
    record_walk::damage,
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIntegrityOutcome as Outcome,
    OfflinePhysicalBlastRadius as Blast, OfflinePhysicalDamageCause as Cause,
};
use bounds::IndexKeyBounds;
use resolver::{resolve, RecordKey, ResolvedRecord};
use worth_foundational::{
    PhysicalArtifactFamily as Family, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
    PhysicalByteRange,
};

const MAXIMUM_ADMITTED_BTREE_HEIGHT: u8 = 5;

#[derive(Default)]
struct RootPointCost {
    observation_index: Option<usize>,
    expected_touches: Option<u64>,
    family_code: u16,
    complete: bool,
}

pub(crate) fn observe_selected_indexes(
    root: &Path,
    roots: &[OfflineRootManifestFacts],
    selected_generation: Option<u64>,
    store: Option<[u8; 16]>,
    walk: &mut BoundedMediaWalk,
) -> Vec<OfflineArtifactObservation> {
    let Some(selected) = roots
        .iter()
        .find(|candidate| Some(candidate.generation) == selected_generation)
    else {
        return Vec::new();
    };
    if selected.payload[400] != 1 {
        return Vec::new();
    }
    let Some(directory_record) = record_key(&selected.payload[408..432]) else {
        return Vec::new();
    };
    let mut observations = Vec::new();
    let directory = resolve(root, selected, directory_record, walk);
    let entries = match directory.as_ref() {
        Ok(record) => directory::inspect_directory(&record.payload, &selected.payload),
        Err(outcome) => Err(outcome.clone()),
    };
    let outcome = entries
        .as_ref()
        .map(|_| Outcome::Intact)
        .unwrap_or_else(|outcome| outcome.clone());
    observations.push(observation(
        selected,
        directory_record,
        Family::DerivedFamilyRootDirectory,
        directory.as_ref().ok(),
        outcome,
        448,
    ));
    let Ok(entries) = entries else {
        return observations;
    };
    let mut roots = BTreeMap::new();
    let mut queue: VecDeque<_> = entries
        .into_iter()
        .map(|entry| {
            roots.insert(
                entry.root,
                RootPointCost {
                    family_code: entry.family_code,
                    complete: true,
                    ..Default::default()
                },
            );
            (
                entry.root,
                entry.family_code,
                None,
                IndexKeyBounds::default(),
                entry.root,
            )
        })
        .collect();
    let mut seen = BTreeMap::new();
    while let Some((identity, family_code, expected_level, bounds, tree_root)) = queue.pop_front() {
        if let Some(previous_root) = seen.insert(identity, tree_root) {
            roots.get_mut(&previous_root).expect("known root").complete = false;
            roots.get_mut(&tree_root).expect("known root").complete = false;
            let outcome = damage(Cause::DuplicateIdentity, None, Blast::ReachableRootSubtree);
            observations.push(observation(
                selected,
                identity,
                Family::BTreeNode,
                None,
                outcome,
                448,
            ));
            continue;
        }
        if seen.len() as u64 > walk.maximum_entries() {
            for root in roots.values_mut() {
                root.complete = false;
            }
            let outcome = walk.entry_bound();
            observations.push(observation(
                selected,
                identity,
                Family::BTreeNode,
                None,
                outcome,
                448,
            ));
            break;
        }
        let record = resolve(root, selected, identity, walk);
        let fact = match record.as_ref() {
            Ok(record) => {
                inspect_btree_node_payload(&record.payload, family_code, walk.counters_mut())
            }
            Err(outcome) => Err(outcome.clone()),
        };
        let outcome = match fact.as_ref() {
            Ok(fact)
                if fact.level >= MAXIMUM_ADMITTED_BTREE_HEIGHT
                    || expected_level.is_some_and(|level| level != fact.level)
                    || fact.previous_sibling == Some(identity)
                    || fact.next_sibling == Some(identity)
                    || !bounds.admits(fact) =>
            {
                damage(Cause::Pointer, None, Blast::ReachableRootSubtree)
            }
            Ok(_) => Outcome::Intact,
            Err(outcome) => outcome.clone(),
        };
        let outcome = if outcome == Outcome::Intact && family_code == 1 {
            match fact.as_ref() {
                Ok(fact) if fact.level == 0 => {
                    catalog_target::verify_catalog_targets(root, selected, fact, store, walk)
                        .err()
                        .unwrap_or(outcome)
                }
                _ => outcome,
            }
        } else {
            outcome
        };
        let intact = outcome == Outcome::Intact;
        let observation_index = observations.len();
        if identity == tree_root && intact {
            let cost = roots.get_mut(&tree_root).expect("known root");
            cost.observation_index = Some(observation_index);
            cost.expected_touches = fact.as_ref().ok().map(|node| u64::from(node.level) + 1);
        }
        if !intact {
            roots.get_mut(&tree_root).expect("known root").complete = false;
        }
        observations.push(observation(
            selected,
            identity,
            Family::BTreeNode,
            record.as_ref().ok(),
            outcome,
            448,
        ));
        if intact {
            let fact = fact.unwrap();
            if fact.level > 0 {
                if queue.len().saturating_add(usize::from(fact.cell_count) + 1) as u64
                    > walk.maximum_entries()
                {
                    for root in roots.values_mut() {
                        root.complete = false;
                    }
                    let outcome = walk.entry_bound();
                    let last = observations
                        .last_mut()
                        .expect("node observation was emitted");
                    *last = last.clone().with_outcome(outcome);
                    break;
                }
                let child_level = fact.level - 1;
                let children = std::iter::once(fact.first_child.expect("interior has first child"))
                    .chain(fact.separator_children.iter().copied());
                queue.extend(
                    children
                        .zip(bounds.children(&fact))
                        .map(|(child, child_bounds)| {
                            (
                                child,
                                family_code,
                                Some(child_level),
                                child_bounds,
                                tree_root,
                            )
                        }),
                );
            }
        }
    }
    for cost in roots.values() {
        if let (true, Some(index), Some(touches)) =
            (cost.complete, cost.observation_index, cost.expected_touches)
        {
            observations[index] = observations[index]
                .clone()
                .with_expected_point_page_touches(
                    touches,
                    if cost.family_code == 1 {
                        "blob_catalog"
                    } else {
                        "dedupe_index"
                    },
                );
        }
    }
    observations
}

fn observation(
    selected: &OfflineRootManifestFacts,
    identity: RecordKey,
    family: Family,
    record: Option<&ResolvedRecord>,
    outcome: Outcome,
    root_pointer_offset: u64,
) -> OfflineArtifactObservation {
    let name = format!("index-record:{}:{:016x}", hex(&identity.0), identity.1);
    let path = record.map(|record| record.path.clone()).unwrap_or_else(|| {
        format!(
            "families/records/roots/root-{:016x}.manifest",
            selected.generation
        )
    });
    let range = match record {
        Some(record) => record
            .offset
            .and_then(|offset| PhysicalByteRange::new(offset, record.payload.len() as u64).ok()),
        None => PhysicalByteRange::new(root_pointer_offset, 24).ok(),
    };
    OfflineArtifactObservation::new(
        path,
        family.into(),
        PhysicalArtifactIdentity::new(name).expect("valid record identity"),
        PhysicalArtifactGeneration::encoded(selected.generation).unwrap(),
        range,
        outcome,
    )
}

fn hex(bytes: &[u8; 16]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(32);
    for byte in bytes {
        value.push(DIGITS[(byte >> 4) as usize] as char);
        value.push(DIGITS[(byte & 15) as usize] as char);
    }
    value
}
