use super::child_expectation::{ChildExpectation, ChildScope};
use super::families::{durable_frame::read_u32, root_manifest::OfflineRootManifestFacts};
use super::{
    BoundedMediaWalk, OfflineArtifactDuplicateEvidence, OfflineArtifactObservation,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflineUnknownPhysicalReason,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;
use worth_foundational::{PhysicalArtifactFamily as Family, PhysicalArtifactGeneration};

mod bootstrap;
mod emission;
mod historical_blob;
mod inspection;
mod origin;
mod root_children;
pub(crate) mod route_inventory;
use bootstrap::observe_bootstrap;
use emission::EmittedChildScopes;
use historical_blob::HistoricalBlobCollector;
pub(crate) use inspection::{damage, inspect_expected, project, shift_outcome};
pub(crate) use origin::TraversalOrigin;
pub(crate) use root_children::root_children;
use route_inventory::{note_route_failure, RouteInventory};

pub(crate) fn observe_records(
    root: &Path,
    store: Option<[u8; 16]>,
    roots: &[OfflineRootManifestFacts],
    current_generation: Option<u64>,
    retirements: super::retirement_evidence::RetirementEvidence,
    checkpoint: super::journal_walk::SelectedCheckpointEvidence,
    selected_records: &mut BTreeSet<Box<str>>,
    walk: &mut BoundedMediaWalk,
) -> Vec<OfflineArtifactObservation> {
    let mut observations = vec![observe_bootstrap(root, store, walk)];
    if walk.exhausted_reason().is_some() {
        return observations;
    }
    let mut queue = VecDeque::new();
    let mut arenas = super::families::extent_arena::ArenaAccounting::default();
    let mut historical_blobs = HistoricalBlobCollector::new(
        roots,
        current_generation,
        walk.maximum_entries(),
        checkpoint.clone(),
    );
    let mut blobs = super::blob_walk::BlobRecordWalk::new(walk.maximum_entries(), checkpoint)
        .with_selected_root_generation(current_generation);
    let mut selected_routes = RouteInventory::new();
    let mut historical_routes = RouteInventory::new();
    observations.extend(retirements.admit(
        root,
        roots,
        current_generation,
        &mut arenas,
        &mut queue,
        walk,
    ));
    let mut emitted = EmittedChildScopes::seed(&mut observations);
    for manifest in roots {
        queue.extend(
            root_children(manifest)
                .into_iter()
                .map(|child| (manifest.generation, child, TraversalOrigin::RootManifest)),
        );
    }
    let mut visited: BTreeMap<(u64, String, u64, TraversalOrigin), ChildExpectation> =
        BTreeMap::new();
    while let Some((root_generation, expected, origin)) = queue.pop_front() {
        let key = (
            root_generation,
            expected.path.clone(),
            expected.offset,
            origin,
        );
        if let Some(prior) = visited.get(&key) {
            if prior != &expected {
                let outcome = damage(Cause::ScopeMismatch, None, Blast::Artifact);
                if origin.selected_blob(root_generation, current_generation) {
                    blobs.note_outcome(&expected, &outcome);
                }
                historical_blobs.note(origin, root_generation, &expected, &outcome);
                note_route_failure(
                    origin,
                    root_generation,
                    current_generation,
                    &expected,
                    &mut selected_routes,
                    &mut historical_routes,
                );
                emitted.push(&mut observations, &expected, project(&expected, 0, outcome));
            }
            continue;
        }
        if visited.len() as u64 >= walk.maximum_entries() {
            let outcome = walk.entry_bound();
            if origin.selected_blob(root_generation, current_generation) {
                blobs.note_outcome(&expected, &outcome);
            }
            blobs.note_walk_stopped(&outcome);
            historical_blobs.note(origin, root_generation, &expected, &outcome);
            note_route_failure(
                origin,
                root_generation,
                current_generation,
                &expected,
                &mut selected_routes,
                &mut historical_routes,
            );
            emitted.push(&mut observations, &expected, project(&expected, 0, outcome));
            break;
        }
        visited.insert(key, expected.clone());
        let path = root.join(&expected.path);
        let arena_frame = matches!(
            expected.scope,
            ChildScope::ExtentManifest { .. } | ChildScope::ExtentChunk { .. }
        );
        let acquired = if !path.try_exists().unwrap_or(true) {
            walk.counters_mut().missing_artifacts += 1;
            Err(damage(Cause::MissingArtifact, None, Blast::Artifact))
        } else if arena_frame {
            walk.acquire_range(
                &path,
                4,
                expected.offset,
                expected.length.expect("arena frame length"),
            )
        } else {
            walk.acquire(&path, 4)
        };
        let mut alias = None;
        let mut length = 0;
        let outcome = match acquired {
            Err(outcome) => outcome,
            Ok(acquired) => {
                alias = acquired
                    .physical_alias_of
                    .as_ref()
                    .map(|path| super::unknown_artifact::relative_path(root, path));
                if let Some(first_path) = &alias {
                    let outcome =
                        Outcome::Unknown(OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected);
                    if origin.selected_blob(root_generation, current_generation) {
                        blobs.note_outcome(&expected, &outcome);
                    }
                    historical_blobs.note(origin, root_generation, &expected, &outcome);
                    note_route_failure(
                        origin,
                        root_generation,
                        current_generation,
                        &expected,
                        &mut selected_routes,
                        &mut historical_routes,
                    );
                    emitted.push(
                        &mut observations,
                        &expected,
                        project(&expected, acquired.byte_length, outcome).with_duplicate(
                            OfflineArtifactDuplicateEvidence::PhysicalAlias {
                                first_path: first_path.clone().into(),
                            },
                        ),
                    );
                    continue;
                }
                if let ChildScope::Page { pages, .. } = expected.scope {
                    let expected_bytes =
                        u64::from(pages) * u64::from(read_u32(&expected.format, 2));
                    if acquired.byte_length as u64 != expected_bytes {
                        let outcome = damage(
                            Cause::Framing,
                            Some((0, acquired.byte_length.max(1) as u64)),
                            Blast::Artifact,
                        );
                        walk.record_outcome(&outcome);
                        emitted.push(&mut observations, &expected, project(&expected, 0, outcome));
                        continue;
                    }
                }
                let end = expected
                    .length
                    .and_then(|length| expected.offset.checked_add(length))
                    .unwrap_or(acquired.byte_length as u64);
                let start = if arena_frame { 0 } else { expected.offset };
                let read_end = if arena_frame {
                    expected.length.unwrap()
                } else {
                    end
                };
                let range = usize::try_from(start)
                    .ok()
                    .zip(usize::try_from(read_end).ok())
                    .and_then(|(start, end)| acquired.bytes.get(start..end));
                match range {
                    None => damage(
                        Cause::Truncation,
                        Some((
                            acquired.byte_length as u64,
                            end.saturating_sub(acquired.byte_length as u64).max(1),
                        )),
                        Blast::Artifact,
                    ),
                    Some(bytes) => {
                        length = bytes.len();
                        let result = inspect_expected(bytes, &expected, walk);
                        match result {
                            Err(outcome) => shift_outcome(outcome, expected.offset),
                            Ok(children) => {
                                if origin == TraversalOrigin::RootManifest
                                    && expected.family == Family::FreeSpaceHeader
                                {
                                    if Some(root_generation) == current_generation {
                                        selected_routes.note_free_space_header(bytes);
                                    } else if current_generation
                                        .and_then(|value| value.checked_sub(1))
                                        == Some(root_generation)
                                    {
                                        historical_routes.note_free_space_header(bytes);
                                    }
                                }
                                if origin == TraversalOrigin::RootManifest
                                    && expected.family == Family::RootRoutingBlock
                                {
                                    if Some(root_generation) == current_generation {
                                        selected_routes.observe(&expected, bytes);
                                    } else if current_generation
                                        .and_then(|value| value.checked_sub(1))
                                        == Some(root_generation)
                                    {
                                        historical_routes.observe(&expected, bytes);
                                    }
                                    for child in &children {
                                        arenas.observe_routed_expectation(root_generation, child);
                                    }
                                }
                                if origin == TraversalOrigin::RootManifest {
                                    arenas.observe(root_generation, &expected, bytes);
                                }
                                if expected.family == Family::ExtentChunkFrame
                                    && origin.selected_blob(root_generation, current_generation)
                                {
                                    blobs.observe_valid_extent_chunk(
                                        &expected,
                                        bytes,
                                        store,
                                        walk.counters_mut(),
                                    );
                                }
                                if expected.family == Family::ExtentChunkFrame {
                                    historical_blobs.observe(
                                        origin,
                                        root_generation,
                                        &expected,
                                        bytes,
                                        store,
                                        walk.counters_mut(),
                                    );
                                }
                                if queue.len().saturating_add(children.len()) as u64
                                    > walk.maximum_entries()
                                {
                                    walk.entry_bound()
                                } else {
                                    queue.extend(
                                        children
                                            .into_iter()
                                            .map(|child| (root_generation, child, origin)),
                                    );
                                    Outcome::Intact
                                }
                            }
                        }
                    }
                }
            }
        };
        if origin.selected_blob(root_generation, current_generation) {
            blobs.note_outcome(&expected, &outcome);
        }
        historical_blobs.note(origin, root_generation, &expected, &outcome);
        if outcome != Outcome::Intact {
            note_route_failure(
                origin,
                root_generation,
                current_generation,
                &expected,
                &mut selected_routes,
                &mut historical_routes,
            );
        }
        walk.record_outcome(&outcome);
        let mut observation = project(&expected, length, outcome);
        if let Some(first_path) = alias {
            observation =
                observation.with_duplicate(OfflineArtifactDuplicateEvidence::PhysicalAlias {
                    first_path: first_path.into(),
                });
        }
        emitted.push(&mut observations, &expected, observation);
    }
    let selected_anchor = current_generation.and_then(|generation| {
        roots
            .iter()
            .find(|root| root.generation == generation)
            .map(|root| root.tier_epoch_anchor)
    });
    let epoch_mismatch = selected_anchor
        .is_some_and(|anchor| anchor.is_some() != selected_routes.tier_epoch_start().is_some());
    if selected_routes.tier_mismatch()
        || epoch_mismatch
        || selected_routes.tier_epoch_start().is_some()
        || selected_anchor.flatten().is_some()
    {
        let generation = current_generation.and_then(PhysicalArtifactGeneration::encoded);
        if let Some(observation) = observations.iter_mut().find(|observation| {
            observation.family().declared() == Some(Family::FreeSpaceHeader)
                && Some(observation.generation()) == generation
        }) {
            let outcome = if selected_routes.tier_mismatch() || epoch_mismatch {
                damage(Cause::ScopeMismatch, None, Blast::Artifact)
            } else {
                // Header bytes alone cannot activate a tier epoch without
                // an independently selected Intent+Completed WAL pair.
                Outcome::Unknown(OfflineUnknownPhysicalReason::WalCoverageUnavailable)
            };
            *observation = observation.clone().with_outcome(outcome.clone());
            walk.record_outcome(&outcome);
        }
    }
    let arena_observations = arenas.finish(root, walk);
    // Historical roots remain independently reported, but their uncertain
    // post-retirement gaps cannot determine the selected blob graph's fate.
    let selected_arena_generation =
        current_generation.and_then(PhysicalArtifactGeneration::encoded);
    let arena_routes_intact = selected_arena_generation.is_some_and(|generation| {
        arena_observations
            .iter()
            .filter(|observation| observation.generation() == generation)
            .all(|observation| matches!(observation.outcome(), Outcome::Intact))
    });
    if let Some(source) = historical_blobs.finish(&arena_observations) {
        blobs = blobs.with_historical_source(source.with_routes(historical_routes));
    }
    blobs = blobs.with_route_inventory(selected_routes);
    for observation in arena_observations {
        emitted.push_untyped(&mut observations, observation);
    }
    for observation in blobs.finish(root, walk, arena_routes_intact) {
        if observation.identity().as_str().starts_with("blob-record:") {
            selected_records.insert(observation.identity().as_str().into());
        }
        walk.record_outcome(observation.outcome());
        emitted.push_untyped(&mut observations, observation);
    }
    for observation in super::selected_index_walk::observe_selected_indexes(
        root,
        roots,
        current_generation,
        store,
        walk,
    ) {
        if observation.family().declared() == Some(Family::BTreeNode)
            && observation.identity().as_str().starts_with("index-record:")
        {
            selected_records.insert(observation.identity().as_str().into());
        }
        walk.record_outcome(observation.outcome());
        emitted.push_untyped(&mut observations, observation);
    }
    observations
}
