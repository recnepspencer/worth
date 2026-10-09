use super::{RetirementEvidence, RetirementFact};
use crate::integrity_observation::copy_evidence::{
    CopyFinalFact, CopyIntentFact, CopyResolutionFact,
};
use crate::integrity_observation::{
    child_expectation::{ChildExpectation, ChildScope},
    families::{
        durable_frame::{read_u16, read_u64},
        extent_arena::ArenaAccounting,
        root_manifest::{read_root_manifest, OfflineRootManifestFacts},
    },
    record_walk::{damage, inspect_expected, project, root_children, TraversalOrigin},
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIntegrityOutcome as Outcome,
    OfflinePhysicalBlastRadius as Blast, OfflinePhysicalDamageCause as Cause,
};
use std::{
    collections::{BTreeSet, VecDeque},
    path::Path,
};
use worth_foundational::{
    PhysicalArtifactFamily as Family, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
    PhysicalByteRange,
};

#[derive(Default)]
struct Graph {
    routes: Vec<ChildExpectation>,
    free: Vec<(u64, u64, u64, u64)>,
    capacity: u64,
}

impl RetirementEvidence {
    pub(crate) fn admit(
        mut self,
        root: &Path,
        roots: &[OfflineRootManifestFacts],
        current: Option<u64>,
        arenas: &mut ArenaAccounting,
        queue: &mut VecDeque<(u64, ChildExpectation, TraversalOrigin)>,
        walk: &mut BoundedMediaWalk,
    ) -> Vec<OfflineArtifactObservation> {
        let mut observations = Vec::new();
        let Some(current) =
            current.and_then(|generation| roots.iter().find(|root| root.generation == generation))
        else {
            return observations;
        };
        observations.extend(self.admit_copies(root, roots, current, arenas, queue, walk));
        for rewrite in std::mem::take(&mut self.rewrites) {
            if current.generation < rewrite.result_root {
                continue;
            }
            let released = self.frames.iter().map(|(_, record)| record).find(|record| {
                !record.completion
                    && !record.arena_only
                    && record.source_root == rewrite.source_root
                    && record.id == rewrite.extent
                    && record.generation == rewrite.source_generation
                    && record.range == Some(rewrite.source)
                    && record.candidate <= current.generation
            });
            let result = if let Some(released) = released {
                admit_intent(root, current, released, walk, &mut observations)
            } else {
                admit_rewrite(root, current, &rewrite, walk, &mut observations).map(Some)
            };
            match result {
                Ok(Some(held)) => {
                    arenas.held_coverage(
                        arena_path(rewrite.source.0),
                        rewrite.source.1,
                        rewrite.source.2,
                        rewrite.extent,
                        rewrite.source_generation,
                        rewrite.source_root,
                        current.generation.saturating_add(1),
                    );
                    queue.push_back((current.generation, held, TraversalOrigin::RetirementProof));
                }
                Ok(None) => {
                    if let Some(released) = released {
                        arenas.held_coverage(
                            arena_path(rewrite.source.0),
                            rewrite.source.1,
                            rewrite.source.2,
                            rewrite.extent,
                            rewrite.source_generation,
                            rewrite.source_root,
                            released.candidate,
                        );
                    }
                }
                Err(outcome) => {
                    walk.record_outcome(&outcome);
                    observations.push(evidence_observation(
                        format!(
                            "families/records/roots/root-{:016x}.manifest",
                            rewrite.source_root
                        ),
                        rewrite.source_root,
                        outcome,
                    ));
                }
            }
        }
        for intent in self.unresolved() {
            let result = admit_intent(root, current, &intent, walk, &mut observations);
            if result.is_ok() {
                if let Some((arena, offset, length)) = intent.range {
                    arenas.held_coverage(
                        arena_path(arena),
                        offset,
                        length,
                        intent.id,
                        intent.generation,
                        intent.source_root,
                        intent.candidate.min(current.generation.saturating_add(1)),
                    );
                }
            }
            match result {
                Ok(Some(held)) => {
                    queue.push_back((current.generation, held, TraversalOrigin::RetirementProof))
                }
                Ok(None) if intent.arena_only => {
                    let path = format!("families/records/arenas/arena-{:016x}.data", intent.id);
                    // The candidate proves namespace ownership was forgotten. The
                    // still-present file is pending exact deletion, not live payload.
                    if current.generation >= intent.candidate
                        && matches!(root.join(&path).try_exists(), Ok(true))
                    {
                        let outcome = match walk.acquire_range(&root.join(&path), 4, 0, 0) {
                            Ok(acquired)
                                if !acquired.is_alias()
                                    && acquired.byte_length as u64 <= intent.bytes =>
                            {
                                arenas.forgotten(path.clone());
                                Outcome::Intact
                            }
                            Ok(_) => mismatch(),
                            Err(outcome) => outcome,
                        };
                        walk.record_outcome(&outcome);
                        observations.push(evidence_observation(path, intent.generation, outcome));
                    }
                }
                Ok(None) => {}
                Err(outcome) => {
                    walk.record_outcome(&outcome);
                    observations.push(evidence_observation(
                        format!(
                            "families/records/roots/root-{:016x}.manifest",
                            intent.source_root
                        ),
                        intent.source_root,
                        outcome,
                    ));
                }
            }
        }
        observations
    }
}

impl RetirementEvidence {
    fn admit_copies(
        &mut self,
        root: &Path,
        roots: &[OfflineRootManifestFacts],
        current: &OfflineRootManifestFacts,
        arenas: &mut ArenaAccounting,
        queue: &mut VecDeque<(u64, ChildExpectation, TraversalOrigin)>,
        walk: &mut BoundedMediaWalk,
    ) -> Vec<OfflineArtifactObservation> {
        let mut observations = Vec::new();
        for intent in std::mem::take(&mut self.copy_intents) {
            let marker = self.copy_resolutions.iter().copied().find(|resolution| {
                resolution.identity() == (intent.operation, intent.lsn, intent.digest)
            });
            let final_copy = self.copy_finals.iter().copied().find(|final_copy| {
                final_copy.intent.operation == intent.operation
                    && final_copy.intent.lsn == intent.lsn
            });
            let valid_final = final_copy.is_none_or(|final_copy| final_copy.intent == intent);
            let valid_marker = match marker {
                Some(CopyResolutionFact::Cancelled { .. }) => final_copy.is_none(),
                Some(CopyResolutionFact::Published {
                    root,
                    publication_lsn,
                    ..
                }) => final_copy.is_some_and(|final_copy| {
                    final_copy.result_root == root && final_copy.publication_lsn == publication_lsn
                }),
                None => true,
            };
            if !valid_final || !valid_marker {
                observations.push(copy_observation(
                    intent,
                    Outcome::Damaged(
                        crate::integrity_observation::OfflinePhysicalDamageLocalization::new(
                            Cause::ScopeMismatch,
                            None,
                            None,
                            Blast::Artifact,
                        ),
                    ),
                ));
                continue;
            }
            if matches!(marker, Some(CopyResolutionFact::Cancelled { .. })) {
                continue;
            }
            if let Some(final_copy) = final_copy {
                if current.generation >= final_copy.result_root
                    && roots
                        .iter()
                        .any(|root| root.generation == final_copy.result_root)
                    && prove_copy_destination(root, current, &final_copy, walk, &mut observations)
                        .is_err()
                {
                    observations.push(copy_observation(intent, mismatch()));
                    continue;
                }
            }
            if marker.is_some() {
                // A published copy may remain the physical owner of a hole in
                // a retained intermediate root until its source-release root.
                // The durable release candidate, not the current file length,
                // bounds that historical protection.
                let release = self.frames.iter().map(|(_, record)| record).find(|record| {
                    !record.completion
                        && !record.arena_only
                        && record.id == intent.extent
                        && final_copy.is_some_and(|copy| record.source_root == copy.source_root)
                        && record.generation == intent.source_generation
                        && record.range == Some(intent.source)
                        && record.candidate <= current.generation
                });
                let validated_release = release.and_then(|record| {
                    admit_intent(root, current, record, walk, &mut observations)
                        .ok()
                        .and_then(|held| held.is_none().then_some(record.candidate))
                });
                let end = validated_release.unwrap_or(current.generation.saturating_add(1));
                arenas.held_coverage(
                    arena_path(intent.source.0),
                    intent.source.1,
                    intent.source.2,
                    intent.extent,
                    intent.source_generation,
                    intent.source_root,
                    end,
                );
                if validated_release.is_none() {
                    match copy_source(root, current, &intent, walk, &mut observations) {
                        Ok(held) => queue.push_back((
                            current.generation,
                            held,
                            TraversalOrigin::RetirementProof,
                        )),
                        Err(outcome) => {
                            walk.record_outcome(&outcome);
                            observations.push(copy_observation(intent, outcome));
                        }
                    }
                }
                continue;
            }
            match copy_source(root, current, &intent, walk, &mut observations) {
                Ok(held) => {
                    arenas.held_coverage(
                        arena_path(intent.source.0),
                        intent.source.1,
                        intent.source.2,
                        intent.extent,
                        intent.source_generation,
                        intent.source_root,
                        current.generation.saturating_add(1),
                    );
                    queue.push_back((current.generation, held, TraversalOrigin::RetirementProof));
                    observe_copy_destination_staging(
                        root,
                        &intent,
                        final_copy,
                        walk,
                        &mut observations,
                    );
                }
                Err(outcome) => {
                    walk.record_outcome(&outcome);
                    observations.push(copy_observation(intent, outcome));
                }
            }
        }
        observations
    }
}

mod graph;
use graph::*;
