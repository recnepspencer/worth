use super::validation_engine_fixtures::*;
use crate::capabilities::AspectPlanSource;
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::tests::support::{aspect_key, field_key};
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, CommitConflict, ConflictClass, CreateIntent,
    EntityAspectCreateIntent, EntityMutationIntent, MutationIntent, TransactionCommitError,
    WorkerIntentBatch,
};
use crate::validation::data::InvariantViolationFields;
use worth_foundational::facade::{
    AspectValue, ContractValidatedAspectValueView, ContractValidationInput, InternedString,
    PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
    StructAspectValue,
};

#[test]
fn unique_entity_aspect_field_invariant_rejects_duplicate_struct_field_projection() {
    let runtime = runtime_with_summary_title_uniqueness();
    commit_entity_with_summary(&runtime, "alpha", "shared-title", "open")
        .expect("first summary entity");

    let before = runtime_marker(&runtime);
    let duplicate = commit_entity_with_summary(&runtime, "beta", "shared-title", "closed");

    assert_unique_entity_field_conflict(duplicate.unwrap_err(), "shared-title");
    assert_runtime_marker_unchanged(&runtime, before);
}

#[test]
fn unique_entity_aspect_field_invariant_ignores_sibling_struct_field_values() {
    let runtime = runtime_with_summary_title_uniqueness();
    commit_entity_with_summary(&runtime, "alpha", "alpha-title", "shared-status")
        .expect("first summary entity");
    let distinct_title =
        commit_entity_with_summary(&runtime, "beta", "beta-title", "shared-status");

    assert!(distinct_title.is_ok());
}

#[test]
fn unique_entity_aspect_field_rejects_entity_aspect_create() {
    let mut runtime = runtime_with_summary_title_commit_boundary_uniqueness();
    let contract = runtime
        .entity_aspect_plan(KindId(1))
        .expect("entity aspect plan")
        .contract_for(&aspect_key("summary"))
        .expect("summary contract")
        .clone();
    let patch = whole_summary_patch(&contract, "shared-title", "open");

    let create = |runtime: &crate::runtime::RelationalRuntime, key: &str| {
        let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
        transaction
            .push_batch(WorkerIntentBatch::new(key).push(MutationIntent::Create(
                CreateIntent::EntityAspects(EntityAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    aspect_patch: patch.clone(),
                }),
            )))
            .expect("test staging stays within configured resource budgets");
        transaction.commit(runtime)
    };

    create(&mut runtime, "first").expect("first entity-aspect create");
    assert_eq!(
        runtime.storage_access().entity_slot_count(),
        1,
        "first entity must remain in the selected runtime state"
    );
    let before = runtime_marker(&runtime);
    let duplicate = create(&mut runtime, "duplicate");
    assert_unique_entity_field_conflict(duplicate.unwrap_err(), "shared-title");
    assert_runtime_marker_unchanged(&runtime, before);
    assert_eq!(runtime.storage_access().entity_slot_count(), 1);
}

#[test]
fn unique_entity_aspect_field_rejects_entity_aspect_patch() {
    let runtime = runtime_with_summary_title_commit_boundary_uniqueness();
    commit_entity_with_summary(&runtime, "first", "shared-title", "open")
        .expect("first summary entity");
    let second = commit_entity_with_summary(&runtime, "second", "second-title", "open")
        .expect("second summary entity");
    let second_id = second
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Entity(entity_id) => Some(*entity_id),
            crate::facade::transactions::RecordRef::Relation(_) => None,
        })
        .expect("second entity id");
    let patch = whole_summary_patch(
        &runtime
            .entity_aspect_plan(KindId(1))
            .expect("entity aspect plan")
            .contract_for(&aspect_key("summary"))
            .expect("summary contract"),
        "shared-title",
        "closed",
    );
    let before = runtime_marker(&runtime);
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("duplicate-patch").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: second_id,
                    aspect_patch: patch,
                }),
            )),
        )
        .expect("test staging stays within configured resource budgets");

    let error = transaction.commit(&runtime).unwrap_err();
    assert_unique_entity_field_conflict(error, "shared-title");
    assert_runtime_marker_unchanged(&runtime, before);
    assert_entity_summary(&runtime, second_id, "second-title", "open");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeMarker {
    version_id: crate::identity::data::VersionId,
    catalog_count: usize,
    entity_slots: usize,
    snapshot_version_id: crate::identity::data::VersionId,
}

fn runtime_marker(runtime: &crate::runtime::RelationalRuntime) -> RuntimeMarker {
    let identity = runtime.main_branch_identity();
    let snapshot = crate::tests::support::snapshot_for_owner_identity(runtime, &identity);
    RuntimeMarker {
        version_id: runtime.current_version_id(),
        catalog_count: runtime.history().immutable_commit_count(),
        entity_slots: runtime.storage_access().entity_slot_count(),
        snapshot_version_id: snapshot.version_id,
    }
}

fn assert_runtime_marker_unchanged(
    runtime: &crate::runtime::RelationalRuntime,
    before: RuntimeMarker,
) {
    assert_eq!(runtime_marker(runtime), before);
}

fn assert_unique_entity_field_conflict(error: TransactionCommitError, value: &str) {
    let TransactionCommitError::Conflict { error, commit_log } = error else {
        panic!("expected an invariant conflict, got {error:?}");
    };
    let CommitConflict { class, .. } = error;
    let ConflictClass::InvariantViolation {
        code,
        detail,
        fields:
            InvariantViolationFields::UniqueEntityField {
                field_locator,
                value: observed,
            },
    } = class
    else {
        panic!("expected typed unique-entity-field conflict, got {class:?}");
    };
    assert_eq!(
        code,
        crate::diagnostics::data::DiagnosticCode::InvariantViolation
    );
    assert_eq!(
        detail,
        "entity aspect field 'summary:title' must be unique, duplicate String value"
    );
    assert_eq!(
        field_locator,
        crate::transactions::data::planned_single_field_locator(
            aspect_key("summary"),
            field_key("title"),
        )
    );
    assert_eq!(
        observed,
        AspectValue::String(InternedString::Raw(value.to_owned()))
    );
    assert_eq!(commit_log.summary().invariant_violation_count, 0);
    assert!(!commit_log.has_commit_published());
}

pub(super) fn assert_entity_summary(
    runtime: &crate::runtime::RelationalRuntime,
    entity_id: crate::identity::data::EntityId,
    title: &str,
    status: &str,
) {
    let identity = runtime.main_branch_identity();
    let snapshot = crate::tests::support::snapshot_for_owner_identity(runtime, &identity);
    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot)
        .expect("main snapshot remains readable");
    let record = read.get_entity(entity_id).expect("entity remains visible");
    let summary = record
        .authoritative_aspect_state
        .as_ref()
        .and_then(|state| state.get(&aspect_key("summary")))
        .expect("summary aspect remains present");
    let ContractValidatedAspectValueView::Struct(summary) = summary.view() else {
        panic!("summary aspect changed shape after rejected patch");
    };
    assert_eq!(
        summary.get(&field_key("title")),
        Some(&AspectValue::String(InternedString::Raw(title.to_owned())))
    );
    assert_eq!(
        summary.get(&field_key("status")),
        Some(&AspectValue::String(InternedString::Raw(status.to_owned())))
    );
}

pub(super) fn whole_summary_patch(
    contract: &worth_foundational::facade::AspectContract,
    title: &str,
    status: &str,
) -> PortableRecordAspectPatch {
    let value = StructAspectValue::new([
        (field_key("title"), AspectValue::String(title.into())),
        (field_key("status"), AspectValue::String(status.into())),
    ])
    .expect("valid summary struct");
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Struct(value),
    }])
}
