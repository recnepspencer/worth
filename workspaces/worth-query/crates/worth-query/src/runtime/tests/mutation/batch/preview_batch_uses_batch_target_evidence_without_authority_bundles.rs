use super::*;

#[test]
fn preview_batch_uses_batch_target_evidence_without_authority_bundles() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-batch-evidence")
        .expect("task runtime should open a named workspace");
    let mut preview = workspace
        .preview_with_options(
            request_execution,
            test_session_label("task-preview-batch-evidence"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview should open");

    let receipt = preview
        .batch(request_execution, |batch| {
            batch
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("preview-task-1"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Preview title one"),
                    )
                })
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("preview-task-2"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Preview title two"),
                    )
                })
        })
        .expect("preview batch should stage");

    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .expect("fixture digest must fit")
            .component_count(),
        2
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .expect("fixture digest must fit")
            .target_evidence_count(),
        2
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .expect("fixture digest must fit")
            .resolved_target_count(),
        0
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .expect("fixture digest must fit")
            .target_collection_count(),
        2
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .expect("fixture digest must fit")
            .target_entity_count(),
        0
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .expect("fixture digest must fit")
            .causality_bundle_count(),
        0
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .expect("fixture digest must fit")
            .provenance_bundle_count(),
        0
    );
    assert!(receipt
        .batch_mutation_evidence()
        .expect("fixture digest must fit")
        .aggregate_causality_digest()
        .is_none());
    assert!(receipt
        .batch_mutation_evidence()
        .expect("fixture digest must fit")
        .aggregate_provenance_digest()
        .is_none());
}
