use super::*;
use crate::facade::indexes::{DerivedIndexBuildRequest, DerivedIndexKind};
use crate::facade::{config::CascadeDeletePolicy, history::BranchId, identity::VersionId};
use crate::tests::support::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

#[test]
fn pinned_read_shares_native_custody_and_outlives_the_runtime() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<RelationalPinnedRead>();
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&request, &runtime, "before");
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(17),
            name: "pinned-name".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: AspectFieldLocator::new(
                    LocatorAuthority::Authoritative,
                    aspect_key("name"),
                    CanonicalFieldPath::single(field_key("name")),
                ),
            },
            branch_scoped: false,
        })
        .index_id;
    let (_, admitted) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let built = runtime.index_authority().build_for_basis(
        &request,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index],
        },
        &admitted,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    let snapshot = runtime.visibility_authority().snapshot();
    let read = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[index], &request)
        .unwrap();
    let basis =
        crate::visibility::snapshot_states::resolve_snapshot_basis(&runtime, &snapshot).unwrap();
    assert!(Arc::ptr_eq(read.basis.root(), basis.root()));
    assert!(Arc::ptr_eq(
        &read.indexes[&index].definition,
        &runtime.indexes.definition(index).unwrap()
    ));
    let generation = runtime
        .indexes
        .exact_generation(index, None, basis.version_id(), read.context.schema_version)
        .unwrap();
    assert!(Arc::ptr_eq(
        read.indexes[&index].generation.as_ref().unwrap(),
        &generation
    ));
    let old_symbol = read.symbols().symbol("name");
    runtime.services.symbols.with_write(|symbols| {
        symbols.intern("after-pin");
    });
    assert_eq!(read.symbols().symbol("after-pin"), None);
    drop(basis);
    drop(runtime);
    assert_eq!(read.query_context().version_id, VersionId(1));
    assert_eq!(read.symbols().symbol("name"), old_symbol);
    assert_eq!(
        read.index_generation(index),
        Some(DerivedIndexGenerationId(1))
    );
    assert!(std::ptr::eq(
        read.schema(),
        read.basis.root().schema_authority().registry()
    ));
}
