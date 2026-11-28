//! Index occurrence reads stay bounded by the selected value and ancestry.
use super::super::durable_publication::branch_history_publications::{
    fork, update_entity, update_entity_on_branch,
};
use super::{admit_main, collect, name_locator, selected};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::{
        BranchId, CommitId, RelationalAuthoredEntityIndexDefinition,
        RelationalAuthoredWriteSelector,
    },
    identity::KindId,
    indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    runtime::RelationalRuntime,
    storage::AuthoritativeFieldComparisonKey,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_publication::{create_entity_outcome, memory_runtime as runtime_with_test_schema},
    shared_relational_certification_test_execution_policy::serial_request,
};

fn scan(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    source: &AdmittedRelationalBranchBasis,
    target: &AdmittedRelationalBranchBasis,
    key: RelationalAuthoredWriteSelector,
) -> (Vec<CommitId>, usize) {
    collect(
        runtime,
        execution,
        runtime
            .authored_write_history_cursor(execution, source, selected(runtime, target), &[key])
            .unwrap(),
    )
}

#[test]
fn indexed_history_visits_exclude_other_values_and_sibling_ancestry() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "authored-predicate".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: name_locator(),
            },
            branch_scoped: true,
            unique_entity_kind: None,
        })
        .unwrap();
    let key = |value: &str| RelationalAuthoredWriteSelector::EntityFieldIndex {
        definition: RelationalAuthoredEntityIndexDefinition {
            index_id: definition.index_id,
            name: definition.name.clone(),
            branch_scoped: definition.branch_scoped,
            locator: name_locator(),
        },
        kind: KindId(1),
        value: AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::String(
            value.into(),
        )),
    };
    let created = create_entity_outcome(&execution, &runtime, "match");
    let entity = changed_entities(&created)[0];
    let source = admit_main(&runtime).unwrap();
    fork(&execution, &runtime, "index-sibling");
    let one = update_entity(&execution, &runtime, entity, "away");
    let two = update_entity(&execution, &runtime, entity, "match");
    let target = admit_main(&runtime).unwrap();
    let expected = scan(&runtime, execution, &source, &target, key("match"));
    assert_eq!(
        expected,
        (vec![one.commit.commit_id, two.commit.commit_id], 2)
    );
    for index in 0..32 {
        let other = create_entity_outcome(&execution, &runtime, &format!("different-{index}"));
        let sibling = update_entity_on_branch(
            &execution,
            &runtime,
            entity,
            "match",
            BranchId("index-sibling".into()),
        );
        release_test_commit_snapshot(&runtime, &other);
        release_test_commit_snapshot(&runtime, &sibling);
    }
    let target = admit_main(&runtime).unwrap();
    assert_eq!(
        scan(&runtime, execution, &source, &target, key("match")),
        expected
    );
    assert!(
        scan(&runtime, execution, &source, &target, key("never-present"))
            .0
            .is_empty()
    );
    for outcome in [&created, &one, &two] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[path = "indexed/membership.rs"]
mod membership;

#[path = "indexed/recovery.rs"]
mod recovery;

#[path = "indexed/durable_coverage.rs"]
mod durable_coverage;
