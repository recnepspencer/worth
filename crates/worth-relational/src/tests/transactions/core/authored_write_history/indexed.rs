//! Canonical equality-predicate writes, not just changed current candidates.
use super::*;
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;

fn register(runtime: &RelationalRuntime, locator: AspectFieldLocator) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "authored-predicate".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator,
            },
            branch_scoped: true,
            unique_entity_kind: None,
        })
        .expect("actual authored-field index registration succeeds")
}

// Live canonical index verification is the private evidence behind the external membership court.
#[test]
fn authored_index_retirement_preserves_verified_entity_generation_and_old_kind() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        )],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key("summary"),
        CanonicalFieldPath::single(field_key("status")),
    );
    register(&runtime, locator);
    let phantom = super::super::struct_field_patch_authority::create_entity_with_summary_fields(
        &execution,
        &runtime,
        "phantom",
        "other-title",
        "open",
        false,
        false,
    );
    let retired = delete_entity(&execution, &runtime, phantom);
    let entries = runtime
        .history
        .canonical_envelope(retired.commit.commit_id)
        .unwrap();
    assert!((*entries.owner_authored_index_writes())
        .verified_entries()
        .unwrap()
        .iter()
        .any(|entry| entry.entity == phantom
            && entry.entity.generation_value() == phantom.generation_value()
            && entry.old_kind == Some(crate::identity::data::KindId(1))
            && entry.new_kind.is_none()));
    release_test_commit_snapshot(&runtime, &retired);
}

/// One authored equal-value write to a built entity-field index.
fn indexed_equal_write(
    execution: &ExecutionRequest<'_, '_>,
) -> (RelationalRuntime, CommitResult, CommitResult) {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let source = test_owner_main_basis(&runtime).unwrap();
    let definition = register(&runtime, name_locator());
    let built = runtime.index_authority().build_for_basis(
        execution,
        crate::facade::indexes::DerivedIndexBuildRequest {
            source_commit_id: source.observation().commit_id().unwrap(),
            branch_id: BranchId("main".into()),
            index_ids: vec![definition.index_id],
        },
        &source,
    );
    assert!(built.failed_indexes.is_empty());
    create_branch_from_main(&runtime, "index-source");
    let equal = update_entity(execution, &runtime, entity, "before");
    (runtime, created, equal)
}

#[path = "indexed/recovery.rs"]
mod recovery;
