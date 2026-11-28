//! Real Relational revision capture for retained-basis historical cursors
//! (Store Milestone 4, Phase M4.4).
//!
//! Reads canonical committed entity and relation records directly from the
//! exact observed branch snapshot (`read_observation`) and walks the pinned
//! revision's real commit ancestry DAG (`ancestor_closure_by_commit_id_order` +
//! `immutable_commit_receipt` + `bounded_canonical_commit_patches`) in a single
//! topological persistent-state pass so sibling branches in a diamond merge
//! never leak patches and merge-heavy histories scale sub-quadratically.

use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroUsize;

use worth_execution::ExecutionRequest;
use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_relational::facade::history::{BoundedCanonicalCommitPatchDenial, BranchId, CommitId};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId, VersionId};
use worth_relational::facade::publication::{
    PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_relational::facade::storage::{RecordLifecycleState, RelationalStorageSubject};
use worth_relational::facade::transactions::RecordRef;
use worth_runtime_world::facade::{
    BranchOriginIdentity, CanonicalKeyDomain, HistoricalBasisRow, HistoricalReadDenial,
    SchemaVersionIdentity,
};

use super::relational_capture_codec::{
    encode_canonical_entity_payload, encode_canonical_history_version,
    encode_canonical_relation_payload,
};
use super::relational_capture_replay::{
    replay_dag_histories, topological_sort_receipts, AncestorRecordCommit, TopologicalCommitPatches,
};

const INITIAL_PATCH_RECORD_LIMIT: usize = 4096;

/// Real committed Relational revision extracted from a [`RelationalRuntime`].
pub struct CapturedRelationalRevision {
    pub commit_id: CommitId,
    pub branch_origin: BranchOriginIdentity,
    pub schema_version: SchemaVersionIdentity,
    pub descriptor: RelationalBranchBasisDescriptor,
    pub rows: Vec<HistoricalBasisRow>,
}

/// Captures the exact committed canonical records and complete real multi-version
/// history (including all inherited parent branch versions up to the pinned
/// commit) visible on `branch_id` across the requested entity and relation kinds.
pub fn capture_relational_branch_revision(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    branch_id: &BranchId,
    schema_id: SchemaId,
    schema_version_id: SchemaVersionId,
    entity_kinds: &[KindId],
    relation_kinds: &[KindId],
) -> Result<CapturedRelationalRevision, HistoricalReadDenial> {
    if execution.consult().is_err() {
        return Err(HistoricalReadDenial::RequestStopped);
    }
    let identity = runtime
        .branch_identity(branch_id)
        .map_err(|_| corrupt(PartitionId(0)))?;
    let (_, observed) = runtime
        .observe_branch(&identity)
        .map_err(|_| corrupt(PartitionId(0)))?;
    let descriptor = observed.descriptor().clone();
    let version_id = observed.observation().version_id();
    let commit_id = runtime
        .history()
        .branch_head_for_observation(&observed.observation())
        .map_err(|_| corrupt(PartitionId(0)))?
        .map(|receipt| receipt.commit_id)
        .or_else(|| {
            runtime
                .history()
                .immutable_commit_receipt_for_version(version_id)
                .map(|r| r.commit_id)
        })
        .unwrap_or(CommitId(version_id.0));
    let branch_origin = BranchOriginIdentity::from_basis_descriptor(&descriptor)
        .ok_or_else(|| corrupt(PartitionId(0)))?;
    let schema_version =
        SchemaVersionIdentity::from_basis_descriptor(schema_id, schema_version_id, &descriptor)
            .ok_or_else(|| corrupt(PartitionId(0)))?;

    let view = runtime
        .read_truth()
        .read_observation(execution, &observed.observation())
        .map_err(|_| HistoricalReadDenial::RequestStopped)?
        .map_err(|_| corrupt(PartitionId(0)))?;

    let mut live_entities = Vec::new();
    let mut head_entity_payloads: BTreeMap<EntityId, Vec<u8>> = BTreeMap::new();
    for record in view.entities() {
        if record.lifecycle != RecordLifecycleState::Live
            || !entity_kinds.contains(&record.kind.kind_id)
        {
            continue;
        }
        let payload = encode_canonical_entity_payload(record.authoritative_aspect_state.as_ref());
        head_entity_payloads.insert(record.entity_id, payload.clone());
        live_entities.push((
            record.entity_id,
            record.kind.kind_id,
            record.created_at_version,
            payload,
        ));
    }

    let mut live_relations = Vec::new();
    let mut head_relation_info: BTreeMap<RelationId, ((EntityId, EntityId), Vec<u8>)> =
        BTreeMap::new();
    for record in view.relations() {
        if record.lifecycle != RecordLifecycleState::Live
            || !relation_kinds.contains(&record.kind.kind_id)
        {
            continue;
        }
        let payload = encode_canonical_relation_payload(
            record.source,
            record.target,
            record.authoritative_aspect_state.as_ref(),
        );
        head_relation_info.insert(
            record.relation_id,
            ((record.source, record.target), payload.clone()),
        );
        live_relations.push((
            record.relation_id,
            record.kind.kind_id,
            record.created_at_version,
            payload,
        ));
    }

    let topo_commits = collect_topological_commit_patches(
        runtime,
        execution,
        commit_id,
        version_id,
        &head_entity_payloads,
        &head_relation_info,
    )?;
    let mut replayed = replay_dag_histories(
        execution,
        &topo_commits,
        commit_id,
        version_id,
        &head_entity_payloads,
        &head_relation_info,
    )?;

    let mut rows = Vec::with_capacity(live_entities.len() + live_relations.len());
    for (entity_id, kind_id, created_at_version, payload) in live_entities {
        let logical_key = encode_record_key(
            kind_id.0,
            entity_id.partition_id.0,
            entity_id.local_slot.0,
            entity_id.generation.0,
        );
        let mut history_versions = replayed.entities.remove(&entity_id).unwrap_or_default();
        if history_versions.is_empty() {
            history_versions.push(encode_canonical_history_version(
                commit_id,
                created_at_version,
                RecordStructuralChange::Created,
                &[],
                &payload,
            ));
        }
        rows.push(HistoricalBasisRow {
            partition_id: entity_id.partition_id,
            domain: CanonicalKeyDomain::Entity,
            kind_id,
            logical_key,
            payload,
            history_versions,
        });
    }

    for (relation_id, kind_id, created_at_version, payload) in live_relations {
        let logical_key = encode_record_key(
            kind_id.0,
            relation_id.partition_id.0,
            relation_id.local_slot.0,
            relation_id.generation.0,
        );
        let mut history_versions = replayed.relations.remove(&relation_id).unwrap_or_default();
        if history_versions.is_empty() {
            history_versions.push(encode_canonical_history_version(
                commit_id,
                created_at_version,
                RecordStructuralChange::Created,
                &[],
                &payload,
            ));
        }
        rows.push(HistoricalBasisRow {
            partition_id: relation_id.partition_id,
            domain: CanonicalKeyDomain::Relation,
            kind_id,
            logical_key,
            payload,
            history_versions,
        });
    }

    Ok(CapturedRelationalRevision {
        commit_id,
        branch_origin,
        schema_version,
        descriptor,
        rows,
    })
}

pub(super) fn collect_topological_commit_patches(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    head_commit: CommitId,
    max_version: VersionId,
    head_entity_payloads: &BTreeMap<EntityId, Vec<u8>>,
    head_relation_info: &BTreeMap<RelationId, ((EntityId, EntityId), Vec<u8>)>,
) -> Result<Vec<TopologicalCommitPatches>, HistoricalReadDenial> {
    let history = runtime.history();
    let mut receipts = Vec::new();
    let mut seen: HashMap<CommitId, ()> = HashMap::new();
    let mut worklist = vec![head_commit];
    while let Some(cid) = worklist.pop() {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        if seen.contains_key(&cid) {
            continue;
        }
        seen.insert(cid, ());
        let Some(receipt) = history.immutable_commit_receipt(cid) else {
            continue;
        };
        if receipt.version_id.0 > max_version.0 {
            continue;
        }
        for &parent in receipt.parents.iter().rev() {
            if !seen.contains_key(&parent) {
                worklist.push(parent);
            }
        }
        receipts.push(receipt);
    }

    let topo_receipts = topological_sort_receipts(receipts);
    let mut topo_commits = Vec::with_capacity(topo_receipts.len());

    for (receipt, primary_parent) in topo_receipts {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        let patches = read_commit_patches(runtime, execution, receipt.commit_id)?;
        let mut entities = Vec::new();
        let mut relations = Vec::new();
        for patch in patches {
            match patch.target {
                RecordRef::Entity(entity_id) => {
                    if !head_entity_payloads.contains_key(&entity_id) {
                        continue;
                    }
                    entities.push((
                        entity_id,
                        AncestorRecordCommit {
                            commit_id: receipt.commit_id,
                            version_id: receipt.version_id,
                            structural_change: patch.structural_change,
                            changed_aspects: patch.authoritative_changed_aspects(),
                            patch: patch.authoritative_patch,
                            relation_endpoints: None,
                        },
                    ));
                }
                RecordRef::Relation(relation_id) => {
                    if !head_relation_info.contains_key(&relation_id) {
                        continue;
                    }
                    relations.push((
                        relation_id,
                        AncestorRecordCommit {
                            commit_id: receipt.commit_id,
                            version_id: receipt.version_id,
                            structural_change: patch.structural_change,
                            changed_aspects: patch.authoritative_changed_aspects(),
                            patch: patch.authoritative_patch,
                            relation_endpoints: patch
                                .relation_birth
                                .map(|birth| (birth.source, birth.target)),
                        },
                    ));
                }
            }
        }
        topo_commits.push(TopologicalCommitPatches {
            primary_parent,
            entities,
            relations,
        });
    }
    Ok(topo_commits)
}

fn read_commit_patches(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    commit_id: CommitId,
) -> Result<Vec<PublishedAuthoritativeRecordPatch>, HistoricalReadDenial> {
    let initial_limit = NonZeroUsize::new(INITIAL_PATCH_RECORD_LIMIT).expect("non-zero");
    match runtime
        .history()
        .bounded_canonical_commit_patches(execution, commit_id, initial_limit)
    {
        Ok(patches) => Ok(patches),
        Err(BoundedCanonicalCommitPatchDenial::WorkExhausted { required_records }) => {
            let exact = NonZeroUsize::new(required_records.max(1)).expect("non-zero");
            runtime
                .history()
                .bounded_canonical_commit_patches(execution, commit_id, exact)
                .map_err(|err| match err {
                    BoundedCanonicalCommitPatchDenial::ReadRequest(_) => {
                        HistoricalReadDenial::RequestStopped
                    }
                    _ => corrupt(PartitionId(0)),
                })
        }
        Err(BoundedCanonicalCommitPatchDenial::ReadRequest(_)) => {
            Err(HistoricalReadDenial::RequestStopped)
        }
        Err(BoundedCanonicalCommitPatchDenial::CommitUnavailable) => Err(corrupt(PartitionId(0))),
    }
}

fn encode_record_key(kind_id: u32, part: u32, slot: u64, gen_num: u32) -> Vec<u8> {
    let mut key = Vec::with_capacity(20);
    key.extend_from_slice(&kind_id.to_be_bytes());
    key.extend_from_slice(&part.to_be_bytes());
    key.extend_from_slice(&slot.to_be_bytes());
    key.extend_from_slice(&gen_num.to_be_bytes());
    key
}

fn corrupt(partition: PartitionId) -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(partition),
    }
}

#[cfg(test)]
#[path = "relational_capture_filter_tests.rs"]
mod filter_tests;
