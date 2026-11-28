use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectContractRevision, AspectFieldLocator, AspectIdentity,
    AspectKey, AspectValue, CanonicalFieldPath, ContractValidationInput, FieldKey,
    LocatorAuthority, PortableAspectContractBasis, PortableAspectPatchOperation,
    PortableRecordAspectPatch, ScalarAspectType,
};
use worth_relational::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, VersionId};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::publication::{PublishedAuthoritativePatch, RecordStructuralChange};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationIntegrityDeclarations, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    ApplyRelationAspectPatchIntent, AspectFieldPatch, CreateIntent, EntityReference, EntitySpec,
    MutationIntent, RecordRef, RelationMutationIntent, RelationSpec, WorkerIntentBatch,
};

use super::{replay_dag_histories, AncestorRecordCommit, TopologicalCommitPatches};

fn rel_contract(name: &str, id: u64) -> AspectContract {
    AspectContract::scalar(
        AspectKey::new(name).unwrap(),
        AspectIdentity(id),
        AspectContractRevision(1),
        ScalarAspectType::String,
    )
}

fn commit_patch(
    rt: &RelationalRuntime,
    exec: &ExecutionRequest<'_, '_>,
    branch: &BranchId,
    batch: WorkerIntentBatch,
) -> (CommitId, VersionId, Vec<AncestorRecordCommit>) {
    let id = rt.branch_identity(branch).unwrap();
    let (_, obs) = rt.observe_branch(&id).unwrap();
    let mut tx = rt
        .begin_branch_transaction(&obs, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let committed = tx.commit(exec, rt).unwrap();
    let (cid, vid) = (committed.commit.commit_id, committed.commit.version_id);
    rt.snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    drop(obs);
    let raw = rt
        .history()
        .bounded_canonical_commit_patches(exec, cid, NonZeroUsize::new(16).unwrap())
        .unwrap();
    let rel_entries = raw
        .into_iter()
        .filter(|p| matches!(p.target, RecordRef::Relation(_)))
        .map(|p| AncestorRecordCommit {
            commit_id: cid,
            version_id: vid,
            structural_change: p.structural_change,
            changed_aspects: p.authoritative_changed_aspects(),
            patch: p.authoritative_patch,
            relation_endpoints: p.relation_birth.map(|b| (b.source, b.target)),
        })
        .collect();
    (cid, vid, rel_entries)
}

#[test]
fn replay_relation_history_isolates_sibling_branches_on_same_relation_id() {
    let ver = SchemaVersionId(1);
    let decls = KindAspectContractDeclarations::new(vec![
        DeclaredAspectContractBinding {
            binding: AspectBinding::RelationField {
                field: FieldKey::new("ralpha").unwrap(),
            },
            contract: rel_contract("ralpha", 101),
        },
        DeclaredAspectContractBinding {
            binding: AspectBinding::RelationField {
                field: FieldKey::new("rbeta").unwrap(),
            },
            contract: rel_contract("rbeta", 102),
        },
    ]);
    let reg = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "e".into(),
            schema_id: SchemaId("replay.rel".into()),
            schema_version_id: ver,
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "r".into(),
            schema_id: SchemaId("replay.rel".into()),
            schema_version_id: ver,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: decls,
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .unwrap();
    let rt = RelationalRuntimeApi::builder()
        .schema_setup(|s| {
            s.schema_registry(reg);
        })
        .build()
        .unwrap();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    let (branch_a, branch_b) = (BranchId("main".into()), BranchId("sibling-b".into()));

    for key in ["e0", "e1"] {
        commit_patch(
            &rt,
            &exec,
            &branch_a,
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    fields: AspectFieldPatch::default(),
                },
            ))),
        );
    }
    let main_id = rt.branch_identity(&branch_a).unwrap();
    let (_, obs0) = rt.observe_branch(&main_id).unwrap();
    let mut eids: Vec<EntityId> = rt
        .read_truth()
        .read_observation(&exec, &obs0.observation())
        .unwrap()
        .unwrap()
        .entities()
        .iter()
        .filter(|e| e.lifecycle == RecordLifecycleState::Live)
        .map(|e| e.entity_id)
        .collect();
    eids.sort();
    drop(obs0);
    let (e0, e1) = (eids[0], eids[1]);

    let mut base_fields = BTreeMap::new();
    for (k, v) in [("ralpha", "RA-BASE"), ("rbeta", "RB-BASE")] {
        base_fields.insert(
            AspectFieldLocator::new(
                LocatorAuthority::Planned,
                AspectKey::new(k).unwrap(),
                CanonicalFieldPath::single(FieldKey::new(k).unwrap()),
            ),
            AspectValue::String(v.into()),
        );
    }
    let (c_base, _, mut base_entries) = commit_patch(
        &rt,
        &exec,
        &branch_a,
        WorkerIntentBatch::new("r0").push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("r0"),
                source: EntityReference::Existing(e0),
                target: EntityReference::Existing(e1),
                fields: AspectFieldPatch::from(base_fields),
            },
        ))),
    );
    let (_, obs_r) = rt.observe_branch(&main_id).unwrap();
    let r0 = rt
        .read_truth()
        .read_observation(&exec, &obs_r.observation())
        .unwrap()
        .unwrap()
        .relations()[0]
        .relation_id;
    drop(obs_r);

    let (_, fork_basis) = rt.observe_fork_source(&branch_a).unwrap();
    rt.fork_branch(&exec, branch_b.clone(), fork_basis).unwrap();

    let rel_update = |aspect: &str, id: u64, val: &str| {
        let c = rel_contract(aspect, id);
        let p = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&c),
            value: ContractValidationInput::Scalar(AspectValue::String(val.into())),
        }]);
        WorkerIntentBatch::new(val).push(MutationIntent::Relation(
            RelationMutationIntent::ApplyAspectPatch(ApplyRelationAspectPatchIntent {
                relation_id: r0,
                aspect_patch: p,
            }),
        ))
    };
    let (c_a, _, mut a_entries) = commit_patch(
        &rt,
        &exec,
        &branch_a,
        rel_update("ralpha", 101, "RA-BRANCH-ONLY"),
    );
    let (c_b, _, mut b_entries) = commit_patch(
        &rt,
        &exec,
        &branch_b,
        rel_update("rbeta", 102, "RB-SIBLING-ONLY"),
    );

    let (base_e, a_e, b_e) = (
        base_entries.pop().unwrap(),
        a_entries.pop().unwrap(),
        b_entries.pop().unwrap(),
    );
    let c_merge = CommitId(c_b.0 + 1);
    let merge_patch = b_e.patch.clone();
    let merge_e = AncestorRecordCommit {
        commit_id: c_merge,
        version_id: VersionId(b_e.version_id.0 + 1),
        structural_change: RecordStructuralChange::Updated,
        changed_aspects: b_e.changed_aspects.clone(),
        patch: merge_patch,
        relation_endpoints: None,
    };
    let c_target_keep_merge = CommitId(c_b.0 + 2);
    let keep_target_e = AncestorRecordCommit {
        commit_id: c_target_keep_merge,
        version_id: VersionId(b_e.version_id.0 + 2),
        structural_change: RecordStructuralChange::Updated,
        changed_aspects: Vec::new(),
        patch: PublishedAuthoritativePatch::empty(),
        relation_endpoints: None,
    };
    let commits = vec![
        TopologicalCommitPatches {
            primary_parent: None,
            entities: vec![],
            relations: vec![(r0, base_e)],
        },
        TopologicalCommitPatches {
            primary_parent: Some(0),
            entities: vec![],
            relations: vec![(r0, a_e)],
        },
        TopologicalCommitPatches {
            primary_parent: Some(0),
            entities: vec![],
            relations: vec![(r0, b_e)],
        },
        TopologicalCommitPatches {
            primary_parent: Some(1),
            entities: vec![],
            relations: vec![(r0, merge_e)],
        },
        TopologicalCommitPatches {
            primary_parent: Some(1),
            entities: vec![],
            relations: vec![(r0, keep_target_e)],
        },
    ];
    let mut head_rels = BTreeMap::new();
    head_rels.insert(r0, ((e0, e1), Vec::new()));
    let replayed = replay_dag_histories(
        &exec,
        &commits,
        CommitId(u64::MAX),
        VersionId(u64::MAX),
        &BTreeMap::new(),
        &head_rels,
    )
    .unwrap();
    let frames = &replayed.relations[&r0];
    let has = |hay: &[u8], needle: &[u8]| hay.windows(needle.len()).any(|w| w == needle);
    assert_eq!(frames.len(), 5);
    assert!(has(&frames[0], b"RA-BASE") && has(&frames[0], b"RB-BASE"));
    assert!(
        has(&frames[1], b"RA-BRANCH-ONLY")
            && has(&frames[1], b"RB-BASE")
            && !has(&frames[1], b"RB-SIBLING-ONLY"),
        "branch_a relation frame must not leak branch_b"
    );
    assert!(
        has(&frames[2], b"RA-BASE")
            && has(&frames[2], b"RB-SIBLING-ONLY")
            && !has(&frames[2], b"RA-BRANCH-ONLY"),
        "branch_b relation frame must not leak branch_a"
    );
    assert!(
        has(&frames[3], b"RA-BRANCH-ONLY") && has(&frames[3], b"RB-SIBLING-ONLY"),
        "diamond merge must apply Relational's merge patch cleanly onto parents[0]"
    );
    assert!(
        has(&frames[4], b"RA-BRANCH-ONLY")
            && has(&frames[4], b"RB-BASE")
            && !has(&frames[4], b"RB-SIBLING-ONLY"),
        "target-keeping merge with empty patch must preserve parents[0] without leaking unmerged parents[1] edits"
    );
}

#[test]
fn replay_emits_frames_in_canonical_version_order_even_when_fifo_visits_sibling_first() {
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    let eid = EntityId::new(PartitionId::main(), 1, 1);
    let mk = |cid: u64, change: RecordStructuralChange| AncestorRecordCommit {
        commit_id: CommitId(cid),
        version_id: VersionId(cid),
        structural_change: change,
        changed_aspects: Vec::new(),
        patch: PublishedAuthoritativePatch::empty(),
        relation_endpoints: None,
    };
    // FIFO topological order: 1 -> 2 (depth 1) -> 4 (depth 1 sibling!) -> 3 (depth 2).
    // Canonical version order MUST emit commit 3 BEFORE commit 4.
    let commits = vec![
        TopologicalCommitPatches {
            primary_parent: None,
            entities: vec![(eid, mk(1, RecordStructuralChange::Created))],
            relations: vec![],
        },
        TopologicalCommitPatches {
            primary_parent: Some(0),
            entities: vec![],
            relations: vec![],
        },
        TopologicalCommitPatches {
            primary_parent: Some(0),
            entities: vec![(eid, mk(4, RecordStructuralChange::Updated))],
            relations: vec![],
        },
        TopologicalCommitPatches {
            primary_parent: Some(1),
            entities: vec![(eid, mk(3, RecordStructuralChange::Updated))],
            relations: vec![],
        },
    ];
    let replayed = replay_dag_histories(
        &exec,
        &commits,
        CommitId(99),
        VersionId(99),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    let frames = &replayed.entities[&eid];
    assert_eq!(frames.len(), 3);
    let commit_ids: Vec<u64> = frames
        .iter()
        .map(|f| u64::from_le_bytes(f[5..13].try_into().unwrap()))
        .collect();
    assert_eq!(
        commit_ids,
        vec![1, 3, 4],
        "replayed frames must preserve canonical (version_id, commit_id) order"
    );
}
