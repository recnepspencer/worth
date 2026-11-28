//! Schema, mutation, and observation helpers for Store M4.4 `S5` diamond-merge
//! Relational ancestry capture courts.

use std::collections::BTreeMap;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, CanonicalFieldPath, ContractValidationInput, LocatorAuthority,
    PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
};
use worth_relational::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    AspectBinding, DeclaredAspectContractBinding, EntityKindRegistration,
    KindAspectContractDeclarations, RelationIntegrityDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    ApplyRelationAspectPatchIntent, AspectFieldPatch, CreateIntent, EntityMutationIntent,
    EntityReference, EntitySpec, MutationIntent, RelationMutationIntent, RelationSpec,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::commit_and_release;
use crate::batches::{aspect_key, field_key, string_contract};

fn aspect_decls(names: &[&str], entity: bool) -> KindAspectContractDeclarations {
    let bindings = names
        .iter()
        .map(|name| DeclaredAspectContractBinding {
            binding: if entity {
                AspectBinding::EntityField {
                    field: field_key(name),
                }
            } else {
                AspectBinding::RelationField {
                    field: field_key(name),
                }
            },
            contract: string_contract(name),
        })
        .collect();
    KindAspectContractDeclarations::new(bindings)
}

fn two_aspect_registry() -> RelationalSchemaRegistry {
    let ver = SchemaVersionId(1);
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "diamond.entity".into(),
            schema_id: SchemaId("diamond.merge".into()),
            schema_version_id: ver,
            aspect_contract_declarations: aspect_decls(&["alpha", "beta"], true),
        })
        .and_then(|reg| {
            reg.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "diamond.relation".into(),
                schema_id: SchemaId("diamond.merge".into()),
                schema_version_id: ver,
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: aspect_decls(&["ralpha", "rbeta"], false),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("two-aspect diamond schema registry")
}

pub(super) fn fields_patch(pairs: &[(&str, &str)]) -> AspectFieldPatch {
    let mut map = BTreeMap::new();
    for &(name, val) in pairs {
        map.insert(
            AspectFieldLocator::new(
                LocatorAuthority::Planned,
                aspect_key(name),
                CanonicalFieldPath::single(field_key(name)),
            ),
            AspectValue::String(val.to_owned().into()),
        );
    }
    AspectFieldPatch::from(map)
}

pub(super) fn update_entity_aspect(
    entity_id: EntityId,
    aspect: &str,
    val: &str,
) -> WorkerIntentBatch {
    WorkerIntentBatch::new(format!("ent-{aspect}-{val}")).push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id,
            fields: fields_patch(&[(aspect, val)]),
        }),
    ))
}

pub(super) fn update_relation_aspect(
    relation_id: RelationId,
    aspect: &str,
    val: &str,
) -> WorkerIntentBatch {
    let contract = string_contract(aspect);
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Scalar(AspectValue::String(val.to_owned().into())),
    }]);
    WorkerIntentBatch::new(format!("rel-{aspect}-{val}")).push(MutationIntent::Relation(
        RelationMutationIntent::ApplyAspectPatch(ApplyRelationAspectPatchIntent {
            relation_id,
            aspect_patch: patch,
        }),
    ))
}

pub(super) fn open_diamond_home(
    dir: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_application::ReadmittedApplicationHome {
    let builder = RelationalRuntimeApi::builder()
        .runtime_name("diamond-merge-court")
        .schema_registry(two_aspect_registry());
    worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(dir, builder),
        execution,
    )
    .expect("open diamond durable home")
}

pub(super) fn seed_entity(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    branch: &BranchId,
    key: &str,
    a: &str,
    b: &str,
) {
    commit_and_release(
        runtime,
        execution,
        branch,
        WorkerIntentBatch::new(format!("seed-{key}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(key),
                fields: fields_patch(&[("alpha", a), ("beta", b)]),
            }),
        )),
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn seed_relation(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    branch: &BranchId,
    key: &str,
    src: EntityId,
    dst: EntityId,
    a: &str,
    b: &str,
) {
    commit_and_release(
        runtime,
        execution,
        branch,
        WorkerIntentBatch::new(format!("seed-{key}")).push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(key),
                source: EntityReference::Existing(src),
                target: EntityReference::Existing(dst),
                fields: fields_patch(&[("ralpha", a), ("rbeta", b)]),
            }),
        )),
    );
}

pub(super) fn live_ids(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    branch: &BranchId,
) -> (Vec<EntityId>, Vec<RelationId>) {
    let id = runtime.branch_identity(branch).unwrap();
    let (_, obs) = runtime.observe_branch(&id).unwrap();
    let view = runtime
        .read_truth()
        .read_observation(execution, &obs.observation())
        .unwrap()
        .unwrap();
    let mut eids: Vec<EntityId> = view
        .entities()
        .iter()
        .filter(|e| e.lifecycle == RecordLifecycleState::Live && e.kind.kind_id == KindId(1))
        .map(|e| e.entity_id)
        .collect();
    eids.sort();
    let mut rids: Vec<RelationId> = view
        .relations()
        .iter()
        .filter(|r| r.lifecycle == RecordLifecycleState::Live && r.kind.kind_id == KindId(2))
        .map(|r| r.relation_id)
        .collect();
    rids.sort();
    (eids, rids)
}
