use super::super::support::*;

#[test]
fn verify_existing_preserves_backend_verified_assertion_evidence_without_mutation_deltas() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.verify-existing")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.verify-existing-table", |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("tasks-verify-existing-table")
        })
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

    let receipt = workspace
        .verify_existing(request_execution, binding, |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect("backend-verified assertion should execute");

    assert_eq!(
        receipt.mutation_family(),
        WorthQueryMutationFamily::Assertion
    );
    assert!(receipt.deltas().is_empty());
    let evidence = receipt
        .existing_truth_assertion_evidence()
        .expect("verified assertion should retain assertion evidence");
    assert_eq!(
        evidence.mode(),
        WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion
    );
    assert_eq!(evidence.asserted_aspect_count(), 1);
    assert!(!evidence.verification_digest().is_empty());

    match workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect")
    {
        WorthQueryInspection::WriteReceipt(inspection) => {
            let evidence = inspection
                .existing_truth_assertion_evidence()
                .expect("inspection should retain assertion evidence");
            assert_eq!(
                evidence.mode(),
                WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion
            );
            assert_eq!(
                evidence.verification_digest(),
                receipt
                    .existing_truth_assertion_evidence()
                    .expect("receipt should retain assertion evidence")
                    .verification_digest()
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn verify_existing_denies_missing_asserted_aspect_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.verify-existing-missing-aspect")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.verify-existing-missing-aspect-table",
            |q| {
                q.from("Task")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    )
                    .schema_basis("tasks-verify-existing-missing-aspect-table")
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
        .verify_existing(request_execution, binding, |task| {
            task.set_aspect(
                test_aspect_touch("status.value"),
                test_authored_string_aspect_value("open"),
            )
        })
        .expect_err("missing asserted aspect should deny");

    match error {
        WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial) => {
            let expected = test_authored_string_terminal_digest("status.value", "open");
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect
            );
            assert_eq!(
                denial.asserted_aspect_touch(),
                Some(&test_aspect_touch("status.value"))
            );
            assert_eq!(
                denial.expected_terminal_value_digest(),
                Some(expected.as_str())
            );
        }
        other => panic!("expected typed assertion denial, got {other:?}"),
    }
}

#[test]
fn verify_existing_denies_mismatched_value_typed_and_early() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.verify-existing-mismatch")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.verify-existing-mismatch-table",
            |q| {
                q.from("Task")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    )
                    .schema_basis("tasks-verify-existing-mismatch-table")
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
        .verify_existing(request_execution, binding, |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Different title"),
            )
        })
        .expect_err("mismatched asserted value should deny");

    match error {
        WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial) => {
            let expected = test_authored_string_terminal_digest("title.value", "Different title");
            let found = test_native_string_value_identity("Seed title");
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthAssertionDenialKind::AssertedValueMismatch
            );
            assert_eq!(
                denial.asserted_aspect_touch(),
                Some(&test_aspect_touch("title.value"))
            );
            assert_eq!(
                denial.expected_terminal_value_digest(),
                Some(expected.as_str())
            );
            assert_eq!(denial.found_terminal_value_digest(), Some(found.as_str()));
        }
        other => panic!("expected typed assertion denial, got {other:?}"),
    }
}

#[test]
fn verify_existing_reports_the_actual_failing_aspect_in_multi_aspect_requests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.verify-existing-multi-mismatch")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.verify-existing-multi-mismatch-table",
            |q| {
                q.from("Task")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    )
                    .schema_basis("tasks-verify-existing-multi-mismatch-table")
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
        .verify_existing(request_execution, binding, |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("status.value"),
                test_authored_string_aspect_value("open"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect_err("missing asserted aspect should deny");

    match error {
        WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial) => {
            let expected = test_authored_string_terminal_digest("status.value", "open");
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect
            );
            assert_eq!(
                denial.asserted_aspect_touch(),
                Some(&test_aspect_touch("status.value"))
            );
            assert_eq!(
                denial.expected_terminal_value_digest(),
                Some(expected.as_str())
            );
        }
        other => panic!("expected typed assertion denial, got {other:?}"),
    }
}
