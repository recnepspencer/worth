use super::*;

#[test]
fn batch_existing_targets_preserve_component_and_aggregate_binding_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.batch-existing")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.batch-existing-table", |q| {
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
                .schema_basis("tasks-batch-existing-table")
        })
        .expect("live view should declare");

    let seed_one = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("First"),
            )
        })
        .expect("first seed should execute");
    let seed_two = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-2"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Second"),
            )
        })
        .expect("second seed should execute");

    let binding_one_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity");
    let binding_two_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-2")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity");
    let binding_one = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                binding_one_authority.clone(),
                seed_one.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding one should build");
    let binding_two = workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                binding_two_authority.clone(),
                seed_two.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding two should build");

    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
                .update_existing(binding_one, |task| {
                    task.set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("First renamed"),
                    )
                })
                .delete_existing_with(binding_two, |delete| {
                    delete.touch(test_aspect_touch("title.value"))
                })
        })
        .expect("existing-target batch should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("batch receipt should inspect");

    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .existing_truth_binding_count(),
        2
    );
    assert!(receipt
        .batch_mutation_evidence()
        .aggregate_existing_truth_binding_digest()
        .is_some());

    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .existing_truth_binding_count(),
                2
            );
            assert_eq!(
                inspection.component_operations()[0]
                    .existing_truth_binding_evidence()
                    .expect("first component should retain existing binding")
                    .authoritative_identity()
                    .as_str(),
                "authority:task-1"
            );
            assert_eq!(
                inspection.component_operations()[1]
                    .existing_truth_binding_evidence()
                    .expect("second component should retain existing binding")
                    .authoritative_identity()
                    .as_str(),
                "authority:task-2"
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .aggregate_existing_truth_binding_digest(),
                receipt
                    .batch_mutation_evidence()
                    .aggregate_existing_truth_binding_digest()
            );
        }
        other => panic!("expected batch write receipt inspection, got {other:?}"),
    }
}

#[test]
fn mixed_existing_and_symbolic_batch_preserves_aggregate_session_digests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.batch-existing-symbolic")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.batch-existing-symbolic-table",
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
                    .schema_basis("tasks-batch-existing-symbolic-table")
            },
        )
        .expect("live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-existing"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Existing"),
            )
        })
        .expect("seed should execute");
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
        .batch(request_execution, |batch| {
            batch
                .insert_symbolic("draft-task", "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-draft"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Draft"),
                    )
                })
                .update_existing(binding, |task| {
                    task.set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Existing renamed"),
                    )
                })
                .update_symbolic(
                    WorthQuerySymbolicTargetReference::new("draft-task")
                        .expect("symbolic reference should build")
                        .in_target_collection("Task")
                        .expect("symbolic collection should build"),
                    |task| {
                        task.set_aspect(
                            test_aspect_touch("title.value"),
                            test_authored_string_aspect_value("Draft renamed"),
                        )
                    },
                )
        })
        .expect("mixed existing/symbolic batch should execute");

    let batch_evidence = receipt.batch_mutation_evidence();
    assert_eq!(batch_evidence.existing_truth_binding_count(), 1);
    assert_eq!(batch_evidence.symbolic_target_reference_count(), 1);
    assert!(batch_evidence
        .aggregate_existing_truth_binding_digest()
        .is_some());
    assert!(batch_evidence
        .aggregate_symbolic_target_reference_digest()
        .is_some());
}
