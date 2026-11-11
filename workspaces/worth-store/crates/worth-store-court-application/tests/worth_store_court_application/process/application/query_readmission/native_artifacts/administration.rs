//! Ordinary Native transactions on the actual Home-owned runtime and schema.
use std::collections::BTreeMap;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_foundational::facade::{
    AspectKey, AspectShape, AspectValue, FieldKey, FieldRequirement, ScalarAspectType,
};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    runtime::{EntityReadRecord, RelationalRuntime, VisibilityProjectionView},
    symbols::ClientKey,
    transactions::*,
};

pub(super) fn view(native: &RelationalRuntime) -> VisibilityProjectionView {
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .unwrap();
    native
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap()
}
pub(super) fn kind(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    name: &str,
) -> KindId {
    let view = view(native);
    let pinned = native
        .read_truth()
        .pin_view_read_capability(&view, &[], execution)
        .unwrap();
    pinned
        .schema()
        .authority_snapshot()
        .entity_kinds
        .iter()
        .find(|kind| kind.kind_name == name)
        .unwrap()
        .kind_id
}
pub(super) fn rows(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    name: &str,
) -> Vec<EntityReadRecord> {
    view(native)
        .bounded_entities_of_kind(execution, kind(native, execution, name), 1024)
        .unwrap()
        .unwrap()
        .into_records()
}
pub(super) fn field<'a>(row: &'a EntityReadRecord, aspect: &str, field: &str) -> &'a AspectValue {
    let state = row.authoritative_aspect_state.as_ref().unwrap();
    let aspect = state.get(&AspectKey::new(aspect).unwrap()).unwrap();
    let worth_foundational::facade::ContractValidatedAspectValueView::Struct(fields) =
        aspect.view()
    else {
        panic!("actual struct row")
    };
    fields.get(&FieldKey::new(field).unwrap()).unwrap()
}
pub(super) fn commit(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
) {
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .unwrap();
    let mut transaction = native
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let committed = transaction.commit(execution, native).unwrap();
    native
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}
pub(super) fn update(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    id: EntityId,
    aspect: &str,
    field: &str,
    value: AspectValue,
) {
    let locator = planned_single_field_locator(
        AspectKey::new(aspect).unwrap(),
        FieldKey::new(field).unwrap(),
    );
    commit(
        native,
        execution,
        WorkerIntentBatch::new("actual Home administrative update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: id,
                fields: AspectFieldPatch::from_locator(locator, value),
            }),
        )),
    );
}
pub(super) fn delete(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    id: EntityId,
) {
    commit(
        native,
        execution,
        WorkerIntentBatch::new("actual Home administrative retirement").push(
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: id,
            })),
        ),
    );
}
pub(super) fn create_fields(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    name: &str,
    key: &str,
    fields: &[(&str, &str, AspectValue)],
) {
    let fields = fields
        .iter()
        .map(|(aspect, field, value)| {
            (
                planned_single_field_locator(
                    AspectKey::new(*aspect).unwrap(),
                    FieldKey::new(*field).unwrap(),
                ),
                value.clone(),
            )
        })
        .collect();
    create(
        native,
        execution,
        kind(native, execution, name),
        key,
        AspectFieldPatch::new(fields),
    );
}
fn create(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    kind_id: KindId,
    key: &str,
    fields: AspectFieldPatch,
) {
    commit(
        native,
        execution,
        WorkerIntentBatch::new("actual Home administrative row").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id,
                client_key: ClientKey::raw(key),
                fields,
            }),
        )),
    );
}

/// These are Native administrative rows, not fabricated workflow executions.
pub(super) fn required_row_batch(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    name: &str,
) -> (KindId, WorkerIntentBatch) {
    let view = view(native);
    let pinned = native
        .read_truth()
        .pin_view_read_capability(&view, &[], execution)
        .unwrap();
    let kind = kind(native, execution, name);
    let registration = pinned.schema().entity_registration(kind).unwrap();
    let mut fields = BTreeMap::new();
    for declared in &registration.aspect_contract_declarations.aspects {
        let AspectShape::Struct(shape) = declared.contract.shape() else {
            panic!("actual administrative family has a struct contract")
        };
        for field in shape.fields() {
            if field.requirement() != FieldRequirement::Required {
                continue;
            }
            let value = match field.value_type() {
                ScalarAspectType::String => {
                    AspectValue::String("administrative-row".to_owned().into())
                }
                ScalarAspectType::UInt64 => AspectValue::UInt64(1),
                other => panic!("unanticipated required administrative field: {other:?}"),
            };
            fields.insert(
                planned_single_field_locator(declared.contract.key().clone(), field.key().clone()),
                value,
            );
        }
    }
    drop(pinned);
    (
        kind,
        WorkerIntentBatch::new("actual administrative owner row").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: kind,
                client_key: ClientKey::raw("persisted-administrative-owner-row"),
                fields: AspectFieldPatch::new(fields),
            }),
        )),
    )
}

pub(super) fn create_required_row(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    name: &str,
) -> (KindId, EntityId) {
    let (kind, batch) = required_row_batch(native, execution, name);
    commit(native, execution, batch);
    let rows = rows(native, execution, name);
    let [row] = rows.as_slice() else {
        panic!("one actual persisted owner row");
    };
    (kind, row.entity_id)
}
