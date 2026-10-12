use super::*;

#[test]
fn delete_existing_verified_denies_mismatch_typed_and_leaves_truth_unchanged() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.delete-existing-verified-mismatch")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.delete-existing-verified-mismatch-table",
            |q| {
                q.from("Task")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("status", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    )
                    .schema_basis("tasks-delete-existing-verified-mismatch-table")
            },
        )
        .expect("live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
            .set_aspect(
                test_aspect_touch("status.value"),
                test_authored_string_aspect_value("open"),
            )
        })
        .expect("seed insert should execute");
    let binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding should build");

    let error = workspace
        .delete_existing_verified(
            request_execution,
            binding.clone(),
            |task| {
                task.set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("closed"),
                )
            },
            |delete| delete.touch(test_aspect_touch("status.value")),
        )
        .expect_err("mismatched verified delete should deny");

    match error {
        WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial) => {
            let expected = test_authored_string_terminal_digest("status.value", "closed");
            let found = test_native_string_value_identity("open");
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthAssertionDenialKind::AssertedValueMismatch
            );
            assert_eq!(
                denial.asserted_aspect_touch(),
                Some(&test_aspect_touch("status.value"))
            );
            assert_eq!(
                denial.expected_terminal_value_digest(),
                Some(expected.as_str())
            );
            assert_eq!(denial.found_terminal_value_digest(), Some(found.as_str()));
        }
        other => panic!("expected typed assertion denial, got {other:?}"),
    }

    let probe = workspace
        .probe_existing(
            request_execution,
            binding,
            test_aspect_touches(["status.value"]),
        )
        .expect("probe should still succeed after denied delete");
    assert_eq!(
        probe
            .field_for_touch(&test_aspect_touch("status.value"))
            .expect("status field should remain present")
            .foundational_value(),
        &test_string_aspect_value("open")
    );
}

#[test]
fn delete_existing_verified_denies_unsupported_backend_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime_with_support(
        request_execution,
        WorthQueryRuntimeSupportProfile::bridge_backed(
            "test-subscription-activation",
            "test-preview-basis",
            "test-inspector-evidence",
        ),
    );
    let mut workspace = runtime
        .workspace("tasks.delete-existing-verified-unsupported")
        .expect("workspace should open");
    let binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"), test_entity_identity("Task:1"))
                .expect("existing entity target should build")
                .in_target_collection("Task")
                .expect("existing entity target collection should build"),
        )
        .expect("binding should build");

    let error = workspace
        .delete_existing_verified(
            request_execution,
            binding,
            |task| {
                task.set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("open"),
                )
            },
            |delete| delete.touch(test_aspect_touch("status.value")),
        )
        .expect_err("unsupported backend verified delete should deny");

    match error {
        WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthAssertionDenialKind::BackendVerificationUnsupported
            );
        }
        other => panic!("expected typed assertion denial, got {other:?}"),
    }
}
