use std::collections::BTreeMap;

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
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    ApplyEntityAspectPatchIntent, AspectFieldPatch, CreateIntent, DeleteEntityIntent,
    EntityMutationIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};

use super::{capture_relational_branch_revision, collect_topological_commit_patches};

#[test]
fn capture_skips_deleted_and_excluded_kind_patches_before_replay() {
    let contract = |name: &str, id: u64| {
        AspectContract::scalar(
            AspectKey::new(name).unwrap(),
            AspectIdentity(id),
            AspectContractRevision(1),
            ScalarAspectType::String,
        )
    };
    let make_decls = |base_id: u64| {
        KindAspectContractDeclarations::new(vec![
            DeclaredAspectContractBinding {
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("blob").unwrap(),
                },
                contract: contract("blob", base_id),
            },
            DeclaredAspectContractBinding {
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("tick").unwrap(),
                },
                contract: contract("tick", base_id + 1),
            },
        ])
    };
    let ver = SchemaVersionId(1);
    let reg = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "selected".into(),
            schema_id: SchemaId("capture.filter".into()),
            schema_version_id: ver,
            aspect_contract_declarations: make_decls(201),
        })
        .unwrap()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(2),
            kind_name: "excluded".into(),
            schema_id: SchemaId("capture.filter".into()),
            schema_version_id: ver,
            aspect_contract_declarations: make_decls(301),
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
    let branch = BranchId("main".into());
    let commit_batch = |rt: &RelationalRuntime, batch: WorkerIntentBatch| {
        let id = rt.branch_identity(&branch).unwrap();
        let (_, obs) = rt.observe_branch(&id).unwrap();
        let mut tx = rt
            .begin_branch_transaction(&obs, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let committed = tx.commit(&exec, rt).unwrap();
        let out = (committed.commit.commit_id, committed.commit.version_id);
        rt.snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
        out
    };
    let make_fields = |blob: String, tick: &str| {
        let mut map = BTreeMap::new();
        for (k, v) in [("blob", blob), ("tick", tick.into())] {
            map.insert(
                AspectFieldLocator::new(
                    LocatorAuthority::Planned,
                    AspectKey::new(k).unwrap(),
                    CanonicalFieldPath::single(FieldKey::new(k).unwrap()),
                ),
                AspectValue::String(v.into()),
            );
        }
        AspectFieldPatch::from(map)
    };
    commit_batch(
        &rt,
        WorkerIntentBatch::new("seed")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("keep"),
                fields: make_fields("keep".into(), "0"),
            })))
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("doomed"),
                fields: make_fields("D".repeat(8192), "0"),
            })))
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("excluded"),
                fields: make_fields("X".repeat(8192), "0"),
            }))),
    );
    let id = rt.branch_identity(&branch).unwrap();
    let (_, obs) = rt.observe_branch(&id).unwrap();
    let view = rt
        .read_truth()
        .read_observation(&exec, &obs.observation())
        .unwrap()
        .unwrap();
    let live: Vec<_> = view
        .entities()
        .iter()
        .filter(|e| e.lifecycle == RecordLifecycleState::Live)
        .collect();
    let mut selected_ids: Vec<EntityId> = live
        .iter()
        .filter(|e| e.kind.kind_id == KindId(1))
        .map(|e| e.entity_id)
        .collect();
    selected_ids.sort();
    let (keep_id, doomed_id) = (selected_ids[0], selected_ids[1]);
    let excluded_id = live
        .iter()
        .find(|e| e.kind.kind_id == KindId(2))
        .unwrap()
        .entity_id;
    drop((view, obs));

    let update_tick = |eid: EntityId, contract_id: u64, tick: usize| {
        let c = contract("tick", contract_id);
        let p = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&c),
            value: ContractValidationInput::Scalar(AspectValue::String(format!("t{tick}").into())),
        }]);
        WorkerIntentBatch::new(format!("u-{tick}")).push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: eid,
                aspect_patch: p,
            }),
        ))
    };
    for i in 1..=20usize {
        commit_batch(&rt, update_tick(doomed_id, 202, i));
        commit_batch(&rt, update_tick(excluded_id, 302, i));
    }
    let (head_cid, head_vid) = commit_batch(
        &rt,
        WorkerIntentBatch::new("delete-doomed").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: doomed_id,
            }),
        )),
    );
    let mut head_entities = BTreeMap::new();
    head_entities.insert(keep_id, Vec::new());
    let topo = collect_topological_commit_patches(
        &rt,
        &exec,
        head_cid,
        head_vid,
        &head_entities,
        &BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        topo.iter().map(|c| c.entities.len()).sum::<usize>(),
        1,
        "deleted and excluded-kind records must be filtered before replay"
    );
    let captured = capture_relational_branch_revision(
        &rt,
        &exec,
        &branch,
        SchemaId("capture.filter".into()),
        ver,
        &[KindId(1)],
        &[],
    )
    .unwrap();
    assert_eq!(captured.rows.len(), 1);
    assert_eq!(captured.rows[0].history_versions.len(), 1);
}
