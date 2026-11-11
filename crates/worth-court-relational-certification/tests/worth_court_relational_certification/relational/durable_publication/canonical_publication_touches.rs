//! Sealed scalar and index touches describe the actual published Native delta.
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{batch_create, durable_runtime},
};
#[test]
fn fresh_scalar_commit_seals_exact_native_lifecycle_and_field_touches() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_relational::facade::history::RelationalDescriptiveTouch;
    use worth_relational::facade::identity::KindId;
    let execution_authority = worth_execution::ExecutionAuthority::try_construct(
        worth_execution::ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(8).unwrap(),
            charged_memory_bytes: Some(512 * 1024 * 1024),
        },
    )
    .unwrap();
    let lease = execution_authority
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(4).unwrap(),
                    None,
                    Some(1_000_000),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();

    for checked in [false, true] {
        let medium = TestDurableMedium::new();
        let runtime = durable_runtime(&medium);
        let committed = if checked {
            create_entity_outcome(
                &worth_execution::ExecutionRequest::leased(&lease),
                &runtime,
                "touch-value",
            )
        } else {
            create_entity_outcome(execution, &runtime, "touch-value")
        };
        let envelope = runtime
            .replay()
            .canonical_commit_envelope(execution, committed.commit.commit_id)
            .unwrap()
            .expect("performed commit has sealed envelope");
        let touches = envelope
            .descriptive_touches()
            .exact_touches()
            .expect("fresh scalar commit has exact native delta");
        assert!(touches.iter().any(|touch| matches!(
            touch,
            RelationalDescriptiveTouch::EntityLifecycle {
                kind: KindId(1),
                ..
            }
        )));
        assert!(touches.iter().any(|touch| matches!(touch,
            RelationalDescriptiveTouch::AspectRevision { aspect, .. } if aspect.as_str() == "name"
        )));
        assert!(touches.iter().any(|touch| matches!(touch,
            RelationalDescriptiveTouch::FieldRevision { kind: KindId(1), aspect, path, presence, .. }
                if aspect.as_str() == "name"
                    && path.fields()[0].as_str() == "name"
                    && *presence == worth_relational::facade::runtime::RelationalFieldPresence::Present
        )));
    }
}

#[test]
fn installed_field_index_commit_touch_carries_native_old_and_new_keys() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_foundational::facade::AspectValue;
    use worth_relational::facade::history::RelationalDescriptiveTouch;
    use worth_relational::facade::indexes::{
        DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
    };
    use worth_testkit_relational::aspect_storage::definition::{
        aspect_field_locator, aspect_key, field_key,
    };

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "touch-index".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity");
    let committed = create_entity_outcome(execution, &runtime, "indexed-value");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, committed.commit.commit_id)
        .unwrap()
        .expect("performed indexed commit has envelope");
    let touches = envelope
        .descriptive_touches()
        .exact_touches()
        .expect("exact native graph");
    let expected =
        worth_relational::facade::storage::AuthoritativeFieldComparisonKey::from_aspect_value(
            &AspectValue::String("indexed-value".into()),
        );
    assert!(touches.iter().any(|touch| matches!(touch,
        RelationalDescriptiveTouch::IndexMembership { index: id, old_key: None, new_key: Some(key), .. }
            if *id == index.index_id && key == &expected
    )));
}

fn create_entity_outcome(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    key: &str,
) -> worth_relational::facade::transactions::CommitResult {
    let mut transaction = super::branch_history_publications::transaction(
        runtime,
        &worth_relational::facade::history::BranchId("main".into()),
    );
    transaction
        .push_batch(
            batch_create(key),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}
