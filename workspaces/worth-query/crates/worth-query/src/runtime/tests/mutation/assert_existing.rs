use super::super::support::*;

#[test]
fn assert_existing_preserves_binding_evidence_without_mutation_deltas() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.assert-existing")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.assert-existing-table", |q| {
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
                .schema_basis("tasks-assert-existing-table")
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
        .assert_existing(request_execution, binding, |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect("existing-truth assertion should execute");

    assert_eq!(
        receipt.mutation_family(),
        WorthQueryMutationFamily::Assertion
    );
    assert!(receipt.deltas().is_empty());
    assert_eq!(
        receipt.target_entity_identity(),
        Some(&seed.deltas()[0].entity_identity)
    );
    assert_eq!(
        receipt.terminal_target_collection_projection(),
        Some("Task")
    );
    assert!(receipt.declared_aspect_value_digest().is_some());

    let binding = receipt
        .existing_truth_binding_evidence()
        .expect("assertion should retain existing-binding evidence");
    assert_eq!(
        binding.family(),
        WorthQueryExistingTruthBindingFamily::DirectEntityIdentity
    );
    assert_eq!(
        binding.authoritative_identity().as_str(),
        "authority:task-1"
    );
    let assertion = receipt
        .existing_truth_assertion_evidence()
        .expect("assertion should retain assertion evidence");
    assert_eq!(
        assertion.mode(),
        WorthQueryExistingTruthAssertionMode::RetainedAuthoritativeAssertion
    );

    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("assertion receipt should inspect");
    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            assert!(inspection.live_patch_artifacts().is_empty());
            assert_eq!(
                inspection
                    .existing_truth_assertion_evidence()
                    .expect("inspection should retain assertion evidence")
                    .mode(),
                WorthQueryExistingTruthAssertionMode::RetainedAuthoritativeAssertion
            );
            assert_eq!(
                inspection.declared_aspect_value_digest(),
                receipt.declared_aspect_value_digest()
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn assert_existing_inspection_digest_changes_with_asserted_value() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.assert-existing-digest")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.assert-existing-digest-table",
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
                    .schema_basis("tasks-assert-existing-digest-table")
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

    let left_binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("left binding should build");
    let right_binding = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("right binding should build");

    let left = workspace
        .assert_existing(request_execution, left_binding, |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect("left assertion should execute");
    let right = workspace
        .assert_existing(request_execution, right_binding, |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Different title"),
            )
        })
        .expect("right assertion should execute");

    assert_ne!(
        left.declared_aspect_value_digest(),
        right.declared_aspect_value_digest()
    );

    let left_inspection = workspace
        .inspect(request_execution, &left)
        .expect("left assertion should inspect");
    let right_inspection = workspace
        .inspect(request_execution, &right)
        .expect("right assertion should inspect");

    match (left_inspection, right_inspection) {
        (WorthQueryInspection::WriteReceipt(left), WorthQueryInspection::WriteReceipt(right)) => {
            assert_ne!(left.inspection_digest(), right.inspection_digest())
        }
        other => panic!("expected paired write receipt inspections, got {other:?}"),
    }
}

#[path = "assert_existing/batch_assert_existing_mixes_with_existing_delete_and_retains_bin.rs"]
mod batch_assert_existing_mixes_with_existing_delete_and_retains_bin;

#[test]
fn preview_assert_existing_requires_authoritative_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-assert-existing")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.preview-assert-existing-table",
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
                    .schema_basis("tasks-preview-assert-existing-table")
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
    let mut preview = workspace
        .preview(
            request_execution,
            test_session_label("assert-existing-preview"),
        )
        .expect("preview should open");
    let binding = preview
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

    let error = preview
        .assert_existing(request_execution, binding, |task| {
            task.set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect_err("preview assertion should require authoritative lane");

    match error.stop_class() {
        WorthQueryStopClass::ExistingTruthAssertionRequiresAuthorityLane { required_lane } => {
            assert_eq!(required_lane, WorthQueryAuthorityLane::AuthoritativeTruth);
        }
        other => panic!("expected typed authoritative lane stop class, got {other:?}"),
    }
}
