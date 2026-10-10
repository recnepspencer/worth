//! mixed batch symbolic and existing targets preserve distinct evidence.
use super::*;

#[test]
fn mixed_batch_symbolic_and_existing_targets_preserve_distinct_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.mixed-target-batch")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.mixed-target-table", |q| {
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
                .schema_basis("tasks-mixed-target-table")
        })
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
        .expect("seed insert should execute");
    let existing_binding = WorthQueryExistingTruthTargetBinding::direct_entity(
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new(
                "authority:task-existing",
            )
            .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity"),
        seed.deltas()[0].entity_identity.clone(),
    )
    .expect("existing binding should build")
    .in_target_collection("Task")
    .expect("existing binding collection should build");
    let symbolic = WorthQuerySymbolicTargetReference::new("draft-task")
        .expect("symbolic reference should build")
        .in_target_collection("Task")
        .expect("symbolic reference collection should build");

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
                .update_symbolic(symbolic.clone(), |task| {
                    task.set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Draft renamed"),
                    )
                })
                .update_existing(existing_binding, |task| {
                    task.set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Existing renamed"),
                    )
                })
        })
        .expect("mixed target batch should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("batch should inspect");

    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .existing_truth_binding_count(),
        1
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_target_reference_count(),
        1
    );

    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .symbolic_target_reference_count(),
                1
            );
            assert_eq!(
                inspection.component_operations()[1]
                    .symbolic_target_reference_evidence()
                    .expect("symbolic component should retain symbolic evidence")
                    .symbol()
                    .as_str(),
                "draft-task"
            );
            assert_eq!(
                inspection.component_operations()[2]
                    .existing_truth_binding_evidence()
                    .expect("existing component should retain existing-target evidence")
                    .authoritative_identity()
                    .as_str(),
                "authority:task-existing"
            );
        }
        other => panic!("expected batch inspection, got {other:?}"),
    }
}
