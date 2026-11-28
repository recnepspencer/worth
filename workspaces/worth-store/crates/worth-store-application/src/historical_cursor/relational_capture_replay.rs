//! Single-pass topological persistent-state DAG replay for Relational ancestry
//! history capture (Store Milestone 4, Phase M4.4).
//!
//! Walks the commit DAG ONCE in topological order using an `Arc`-shared
//! persistent big-endian Patricia trie ([`PersistentU128Map`]) over record keys:
//! * Every Relational commit (both ordinary single-parent commits and 2-parent
//!   merge commits produced by `execute_prepared_merge`) applies its
//!   `PublishedAuthoritativePatch` strictly onto its target branch head
//!   (`parents[0]`).
//! * Inheriting `parents[0]`'s persistent map is `O(1)` (`Arc::clone`), and
//!   applying `k` patched records is `O(k log N)` with `O(C + P log N)` total
//!   time and memory across linear, branching, and merge-heavy DAGs.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};
use worth_relational::facade::identity::{EntityId, RelationId, VersionId};
use worth_relational::facade::publication::{PublishedAuthoritativePatch, RecordStructuralChange};
use worth_runtime_world::facade::HistoricalReadDenial;

use super::persistent_map::PersistentU128Map;
use super::relational_capture_codec::{
    apply_published_patch, encode_canonical_history_version, encode_replayed_entity_payload,
    encode_replayed_relation_payload, ReplayedAspectState,
};

pub(super) struct AncestorRecordCommit {
    pub(super) commit_id: CommitId,
    pub(super) version_id: VersionId,
    pub(super) structural_change: RecordStructuralChange,
    pub(super) changed_aspects: Vec<AspectKey>,
    pub(super) patch: PublishedAuthoritativePatch,
    pub(super) relation_endpoints: Option<(EntityId, EntityId)>,
}

pub(super) struct TopologicalCommitPatches {
    pub(super) primary_parent: Option<usize>,
    pub(super) entities: Vec<(EntityId, AncestorRecordCommit)>,
    pub(super) relations: Vec<(RelationId, AncestorRecordCommit)>,
}

pub(super) struct ReplayedDagHistories {
    pub(super) entities: BTreeMap<EntityId, Vec<Vec<u8>>>,
    pub(super) relations: BTreeMap<RelationId, Vec<Vec<u8>>>,
    #[cfg(test)]
    pub(super) steps: usize,
}

#[derive(Clone)]
struct ReplayedRelationState {
    endpoints: (EntityId, EntityId),
    aspects: ReplayedAspectState,
}

#[inline]
const fn entity_u128(id: EntityId) -> u128 {
    ((id.partition_id.0 as u128) << 96)
        | ((id.local_slot.0 as u128) << 32)
        | (id.generation.0 as u128)
}

#[inline]
const fn relation_u128(id: RelationId) -> u128 {
    ((id.partition_id.0 as u128) << 96)
        | ((id.local_slot.0 as u128) << 32)
        | (id.generation.0 as u128)
}

/// Replays all entity and relation histories across a topologically sorted
/// commit DAG in a single `O(C + P log N)` pass with `Arc`-shared persistent
/// maps, applying every commit's `PublishedAuthoritativePatch` strictly onto
/// its target parent (`parents[0]`).
pub(super) fn replay_dag_histories(
    execution: &ExecutionRequest<'_, '_>,
    commits: &[TopologicalCommitPatches],
    head_commit: CommitId,
    head_version: VersionId,
    head_entity_payloads: &BTreeMap<EntityId, Vec<u8>>,
    head_relation_info: &BTreeMap<RelationId, ((EntityId, EntityId), Vec<u8>)>,
) -> Result<ReplayedDagHistories, HistoricalReadDenial> {
    let n = commits.len();
    let mut remaining_children = vec![0usize; n];
    for (idx, commit) in commits.iter().enumerate() {
        if let Some(p_idx) = commit.primary_parent
            && p_idx < idx
        {
            remaining_children[p_idx] += 1;
        }
    }

    let mut entity_states: Vec<Option<PersistentU128Map<ReplayedAspectState>>> = vec![None; n];
    let mut relation_states: Vec<Option<PersistentU128Map<ReplayedRelationState>>> = vec![None; n];
    let mut entity_frames: BTreeMap<EntityId, Vec<(u64, u64, Vec<u8>)>> = BTreeMap::new();
    let mut relation_frames: BTreeMap<RelationId, Vec<(u64, u64, Vec<u8>)>> = BTreeMap::new();
    let mut steps = 0usize;

    for (idx, commit) in commits.iter().enumerate() {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        let p0 = commit.primary_parent.filter(|&p_idx| p_idx < idx);
        let mut cur_entities = p0
            .and_then(|parent_idx| entity_states[parent_idx].clone())
            .unwrap_or_default();
        let mut cur_relations = p0
            .and_then(|parent_idx| relation_states[parent_idx].clone())
            .unwrap_or_default();
        steps += 1;

        for (entity_id, entry) in &commit.entities {
            let key = entity_u128(*entity_id);
            let mut next = match entry.structural_change {
                RecordStructuralChange::Created => ReplayedAspectState::new(),
                _ => cur_entities
                    .get(key, &mut steps)
                    .map(|existing| (**existing).clone())
                    .unwrap_or_default(),
            };
            apply_published_patch(&mut next, &entry.patch);
            let payload = if entry.commit_id == head_commit && entry.version_id == head_version {
                head_entity_payloads
                    .get(entity_id)
                    .cloned()
                    .unwrap_or_else(|| encode_replayed_entity_payload(&next))
            } else {
                encode_replayed_entity_payload(&next)
            };
            entity_frames.entry(*entity_id).or_default().push((
                entry.version_id.0,
                entry.commit_id.0,
                encode_canonical_history_version(
                    entry.commit_id,
                    entry.version_id,
                    entry.structural_change,
                    &entry.changed_aspects,
                    &payload,
                ),
            ));
            cur_entities = cur_entities.insert(key, Arc::new(next), &mut steps);
        }

        for (relation_id, entry) in &commit.relations {
            let key = relation_u128(*relation_id);
            let default_endpoints = head_relation_info
                .get(relation_id)
                .map(|(ep, _)| *ep)
                .or(entry.relation_endpoints)
                .unwrap_or((
                    EntityId::new(relation_id.partition_id, 0, 0),
                    EntityId::new(relation_id.partition_id, 0, 0),
                ));
            let mut next = match entry.structural_change {
                RecordStructuralChange::Created => ReplayedRelationState {
                    endpoints: entry.relation_endpoints.unwrap_or(default_endpoints),
                    aspects: ReplayedAspectState::new(),
                },
                _ => cur_relations
                    .get(key, &mut steps)
                    .map(|existing| (**existing).clone())
                    .unwrap_or(ReplayedRelationState {
                        endpoints: default_endpoints,
                        aspects: ReplayedAspectState::new(),
                    }),
            };
            apply_published_patch(&mut next.aspects, &entry.patch);
            let payload = if entry.commit_id == head_commit && entry.version_id == head_version {
                head_relation_info
                    .get(relation_id)
                    .map(|(_, p)| p.clone())
                    .unwrap_or_else(|| {
                        encode_replayed_relation_payload(
                            next.endpoints.0,
                            next.endpoints.1,
                            &next.aspects,
                        )
                    })
            } else {
                encode_replayed_relation_payload(next.endpoints.0, next.endpoints.1, &next.aspects)
            };
            relation_frames.entry(*relation_id).or_default().push((
                entry.version_id.0,
                entry.commit_id.0,
                encode_canonical_history_version(
                    entry.commit_id,
                    entry.version_id,
                    entry.structural_change,
                    &entry.changed_aspects,
                    &payload,
                ),
            ));
            cur_relations = cur_relations.insert(key, Arc::new(next), &mut steps);
        }

        if let Some(parent_idx) = p0 {
            remaining_children[parent_idx] = remaining_children[parent_idx].saturating_sub(1);
            if remaining_children[parent_idx] == 0 {
                entity_states[parent_idx] = None;
                relation_states[parent_idx] = None;
            }
        }
        if remaining_children[idx] > 0 {
            entity_states[idx] = Some(cur_entities);
            relation_states[idx] = Some(cur_relations);
        }
    }

    Ok(ReplayedDagHistories {
        entities: finalize_canonical_frame_order(entity_frames),
        relations: finalize_canonical_frame_order(relation_frames),
        #[cfg(test)]
        steps,
    })
}

fn finalize_canonical_frame_order<K: Ord>(
    raw: BTreeMap<K, Vec<(u64, u64, Vec<u8>)>>,
) -> BTreeMap<K, Vec<Vec<u8>>> {
    raw.into_iter()
        .map(|(k, mut frames)| {
            if frames
                .windows(2)
                .any(|w| (w[0].0, w[0].1) > (w[1].0, w[1].1))
            {
                frames.sort_unstable_by_key(|(v, c, _)| (*v, *c));
            }
            (k, frames.into_iter().map(|(_, _, bytes)| bytes).collect())
        })
        .collect()
}

/// Linear `O(C + E)` topological sort over receipts already ordered by
/// ascending `CommitId`, returning each receipt paired with its resolved dense
/// `primary_parent` (`parents[0]`) index in the returned topological slice.
pub(super) fn topological_sort_receipts(
    receipts: Vec<RelationalCommitReceipt>,
) -> Vec<(RelationalCommitReceipt, Option<usize>)> {
    let n = receipts.len();
    let mut id_to_raw = HashMap::with_capacity(n);
    for (raw_idx, receipt) in receipts.iter().enumerate() {
        id_to_raw.insert(receipt.commit_id, raw_idx);
    }
    let mut indegree = vec![0usize; n];
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut primary_parent_raw: Vec<Option<usize>> = vec![None; n];
    let mut seen_parent_stamp = vec![usize::MAX; n];
    for (raw_idx, receipt) in receipts.iter().enumerate() {
        primary_parent_raw[raw_idx] = receipt
            .parents
            .first()
            .and_then(|parent| id_to_raw.get(parent).copied());
        for &parent in &receipt.parents {
            if let Some(&parent_raw) = id_to_raw.get(&parent)
                && seen_parent_stamp[parent_raw] != raw_idx
            {
                seen_parent_stamp[parent_raw] = raw_idx;
                indegree[raw_idx] += 1;
                children[parent_raw].push(raw_idx);
            }
        }
    }
    let mut ready = VecDeque::new();
    for (raw_idx, &deg) in indegree.iter().enumerate() {
        if deg == 0 {
            ready.push_back(raw_idx);
        }
    }
    let mut slots: Vec<Option<RelationalCommitReceipt>> = receipts.into_iter().map(Some).collect();
    let mut raw_to_topo: Vec<Option<usize>> = vec![None; n];
    let mut ordered = Vec::with_capacity(n);
    while let Some(raw_idx) = ready.pop_front() {
        let topo_idx = ordered.len();
        raw_to_topo[raw_idx] = Some(topo_idx);
        let primary_parent_topo = primary_parent_raw[raw_idx]
            .and_then(|p_raw| raw_to_topo[p_raw])
            .filter(|&p_topo| p_topo < topo_idx);
        if let Some(receipt) = slots[raw_idx].take() {
            ordered.push((receipt, primary_parent_topo));
        }
        for &kid_raw in &children[raw_idx] {
            indegree[kid_raw] = indegree[kid_raw].saturating_sub(1);
            if indegree[kid_raw] == 0 {
                ready.push_back(kid_raw);
            }
        }
    }
    ordered
}

#[cfg(test)]
#[path = "relational_capture_replay_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "relational_capture_scaling_tests.rs"]
mod scaling_tests;
