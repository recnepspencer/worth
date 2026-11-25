//! Cold checkpoint history can replay field occurrences without extending the live index interval.
use super::super::{admit_main, name_locator, selected};
use super::membership::{key, register};
use super::{
    changed_entities, create_entity_outcome, fork, release_test_commit_snapshot,
    runtime_with_test_schema, scan, serial_request, update_entity, BranchId, ExecutionRequest,
    RelationalAuthoredWriteSelector,
};
use worth_foundational::facade::CanonicalFieldPath;
use worth_relational::facade::{
    history::{RelationalAuthoredWriteDenial, RelationalAuthoredWriteScope},
    indexes::DerivedIndexBuildRequest,
    transactions::RecordRef,
};
use worth_testkit_relational::aspect_storage::{aspect_key, field_key};

#[test]
fn index_rebuild_requires_live_verified_canonical_evidence() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let pre_install_source = admit_main(&runtime).unwrap();
    let uncovered = update_entity(&execution, &runtime, entity, "before");
    let source = admit_main(&runtime).unwrap();
    let definition = register(&runtime, name_locator());
    assert!(matches!(
        runtime.authored_write_history_cursor(
            execution,
            &pre_install_source,
            selected(&runtime, &source),
            &[key(&definition, "before")],
        ),
        Err(RelationalAuthoredWriteDenial::IncompleteIndexHistory)
    ));
    let built = runtime.index_authority().build_for_basis(
        &execution,
        DerivedIndexBuildRequest {
            source_commit_id: source.observation().commit_id().unwrap(),
            branch_id: BranchId("main".into()),
            index_ids: vec![definition.index_id],
        },
        &source,
    );
    assert!(built.failed_indexes.is_empty());
    fork(&execution, &runtime, "index-source");
    let equal = update_entity(&execution, &runtime, entity, "before");
    let target = admit_main(&runtime).unwrap();
    let mut fabricated = key(&definition, "before");
    let RelationalAuthoredWriteSelector::EntityFieldIndex {
        definition: fake, ..
    } = &mut fabricated
    else {
        unreachable!();
    };
    fake.name.push_str("-fabricated");
    assert!(matches!(
        runtime.authored_write_history_cursor(
            execution,
            &source,
            selected(&runtime, &target),
            &[fabricated],
        ),
        Err(RelationalAuthoredWriteDenial::IncompleteIndexHistory)
    ));
    let expected = scan(
        &runtime,
        execution,
        &source,
        &target,
        key(&definition, "before"),
    );
    let empty = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        worth_execution::CancellationToken::new(),
        None,
    );
    assert!(runtime
        .rebuild_authored_write_generation(
            ExecutionRequest::serial(&empty),
            selected(&runtime, &target)
        )
        .is_err());
    assert_eq!(
        scan(
            &runtime,
            execution,
            &source,
            &target,
            key(&definition, "before")
        ),
        expected
    );
    runtime
        .discard_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    runtime
        .rebuild_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    assert_eq!(
        scan(
            &runtime,
            execution,
            &source,
            &target,
            key(&definition, "before")
        ),
        expected
    );
    runtime
        .durability_authority()
        .checkpoint(&execution)
        .unwrap();
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
&execution,
execution.allocation_policy(),
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint, worth_testkit_relational::cold_recovery::members_source(&checkpoint))
        .unwrap();
    let recovered_source = recovered
        .admit_branch_basis(
            &recovered
                .branch_identity(&BranchId("index-source".into()))
                .unwrap(),
        )
        .unwrap();
    let recovered_target = admit_main(&recovered).unwrap();
    recovered
        .rebuild_authored_write_generation(execution, selected(&recovered, &recovered_target))
        .unwrap();
    assert_eq!(
        scan(
            &recovered,
            execution,
            &recovered_source,
            &recovered_target,
            RelationalAuthoredWriteSelector::Record {
                record: RecordRef::Entity(entity),
                scope: RelationalAuthoredWriteScope::Field(
                    aspect_key("name"),
                    CanonicalFieldPath::single(field_key("name")),
                ),
            },
        )
        .0,
        [equal.commit.commit_id],
        "cold field history retains the exact authored no-op despite an unchanged value"
    );
    assert!(matches!(
        recovered.authored_write_history_cursor(
            execution,
            &recovered_source,
            selected(&recovered, &recovered_target),
            &[key(&definition, "before")]
        ),
        Err(RelationalAuthoredWriteDenial::IncompleteIndexHistory)
    ));
    let fresh_source = admit_main(&recovered).unwrap();
    let fresh = update_entity(&execution, &recovered, entity, "after-recovery");
    let fresh_target = admit_main(&recovered).unwrap();
    assert_eq!(
        scan(
            &recovered,
            execution,
            &fresh_source,
            &fresh_target,
            key(&definition, "before")
        )
        .0,
        [fresh.commit.commit_id],
        "an earlier decoded gap outside this open interval does not poison live history"
    );
    release_test_commit_snapshot(&recovered, &fresh);
    for outcome in [&created, &uncovered, &equal] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
