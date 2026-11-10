//! A Complete checkpoint carries each live root's index selections through a cold reopen.
use crate::index_retirement::aspect_field_locator;
use crate::leased_preparation::ordinary_context::support::{aspect_key, field_key, main_basis};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{RelationalCheckpointArtifactPolicy, RelationalNativeCheckpoint},
    history::BranchId,
    indexes::{
        DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexGeneration, DerivedIndexId,
        DerivedIndexKind,
    },
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    cold_recovery::{immutable_source, recover, update_entity},
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};

struct Carried {
    changed: DerivedIndexId,
    stable: DerivedIndexId,
    current: Arc<DerivedIndexGeneration>,
    carried: Arc<DerivedIndexGeneration>,
}

/// Two built indexes, then one write that changes only the first: the second
/// index's generation is carried, unchanged, to the newer root.
fn producer(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) -> Carried {
    let register = |name: &str, aspect: &str| {
        runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: name.into(),
                branch_scoped: true,
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key(aspect), field_key(aspect)),
                },
            })
            .unwrap()
            .index_id
    };
    let changed = register("complete.name", "name");
    let stable = register("complete.lifecycle", "lifecycle");
    let created = create_entity_outcome(execution, runtime, "complete-carry-value");
    let entity = changed_entities(&created)[0];
    release_test_commit_snapshot(runtime, &created);
    let basis = main_basis(runtime);
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: basis.identity().branch_id().clone(),
            index_ids: vec![changed, stable],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    drop(basis);
    let carried = generation(execution, runtime, stable);
    update_entity(execution, runtime, entity, "complete-carry-updated");
    let current = generation(execution, runtime, changed);
    assert_eq!(generation(execution, runtime, stable), carried);
    assert!(carried.applicability.version_id < current.applicability.version_id);
    Carried {
        changed,
        stable,
        current,
        carried,
    }
}

/// The exact generation the current main root selects for one index.
fn generation(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    index: DerivedIndexId,
) -> Arc<DerivedIndexGeneration> {
    let (_, basis) = runtime
        .observe_branch(&runtime.branch_identity(&BranchId("main".into())).unwrap())
        .unwrap();
    runtime
        .index_access()
        .read_published_generation_for_observation(execution, index, &basis.observation())
        .unwrap()
        .expect("the current root selects a generation for every built index")
}

fn complete(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> RelationalNativeCheckpoint {
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(
            execution,
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    complete.checkpoint().clone()
}

/// Both indexes select their exact original generations on the reopened root.
fn assert_selected(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    at: &Carried,
) {
    for (index, expected) in [(at.changed, &at.current), (at.stable, &at.carried)] {
        let selected = generation(execution, runtime, index);
        assert_eq!(selected.generation_id, expected.generation_id);
        assert_eq!(selected.applicability, expected.applicability);
        assert_eq!(selected.entries, expected.entries);
    }
}

#[test]
fn complete_checkpoint_carries_an_unchanged_index_selection_through_cold_reopen() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer_runtime = durable_runtime(&medium);
    let at = producer(execution, &producer_runtime);
    let checkpoint = complete(execution, &producer_runtime);
    let source = immutable_source(&checkpoint, &medium);
    drop(producer_runtime);
    let reopened = recover(execution, checkpoint, source, durable_runtime(&medium));
    assert_selected(execution, &reopened, &at);
}
