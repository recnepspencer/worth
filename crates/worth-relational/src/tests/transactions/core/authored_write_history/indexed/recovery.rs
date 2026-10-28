//! Live installation coverage and descriptive-only cold recovery.
use super::*;

#[test]
fn index_rebuild_requires_live_verified_canonical_evidence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let pre_install_source = test_owner_main_basis(&runtime).unwrap();
    let uncovered = update_entity(&execution, &runtime, entity, "before");
    let source = test_owner_main_basis(&runtime).unwrap();
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
    create_branch_from_main(&runtime, "index-source");
    let equal = update_entity(&execution, &runtime, entity, "before");
    let target = test_owner_main_basis(&runtime).unwrap();
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
    let envelope = runtime
        .history
        .canonical_envelope(equal.commit.commit_id)
        .unwrap();
    let encoded = rmp_serde::to_vec_named(envelope.as_ref()).unwrap();
    let decoded: crate::history::data::CanonicalCommitEnvelope =
        rmp_serde::from_slice(&encoded).unwrap();
    assert_eq!(decoded, *envelope);
    assert!(decoded.authored_index_writes.verified_entries().is_none());
    // This owned decoded description has no live verification. Form the
    // optional-field-absent legacy wire without JSON's bounded Value numbers;
    // it is a codec fixture, not newly admitted historical evidence.
    let mut old_shape = decoded;
    old_shape.authored_index_writes = Default::default();
    let legacy_bytes = rmp_serde::to_vec_named(&old_shape).unwrap();
    let legacy: crate::history::data::CanonicalCommitEnvelope =
        rmp_serde::from_slice(&legacy_bytes).unwrap();
    assert!(legacy.authored_index_writes.is_unavailable());
    let (_, recovered) =
        checkpoint_and_recover_with(&execution, &runtime, durable_runtime_with_test_schema);
    let recovered_source = recovered
        .admit_branch_basis(
            &recovered
                .branch_identity(&BranchId("index-source".into()))
                .unwrap(),
        )
        .unwrap();
    let recovered_target = test_owner_main_basis(&recovered).unwrap();
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
    let fresh_source = test_owner_main_basis(&recovered).unwrap();
    let fresh = update_entity(&execution, &recovered, entity, "after-recovery");
    let fresh_target = test_owner_main_basis(&recovered).unwrap();
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
