use super::super::support::*;

#[test]
fn preview_insert_uses_aspect_native_authoring_and_stays_preview_local() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-aspect-insert")
        .expect("task runtime should open a named workspace");
    let mut preview = workspace
        .preview_with_options(
            request_execution,
            test_session_label("task-preview"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview should open");

    let receipt = preview
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("preview-task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Preview title"),
            )
        })
        .expect("preview insert should stage");
    let outcome = preview.discard();

    assert_eq!(
        receipt.authority_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert_eq!(receipt.basis_lane(), WorthQueryAuthorityLane::PreviewTruth);
    assert_eq!(receipt.mutation_family(), WorthQueryMutationFamily::Insert);
    assert_eq!(
        receipt.terminal_declared_collection_projection(),
        Some("Task")
    );
    assert_eq!(
        receipt.terminal_target_collection_projection(),
        Some("Task")
    );
    assert_eq!(
        receipt.target_evidence().declared().target_class(),
        WorthQueryMutationTargetClass::Collection
    );
    assert_eq!(
        receipt.target_evidence().resolved().target_class(),
        WorthQueryMutationTargetClass::Collection
    );
    assert!(receipt.causality_evidence().is_none());
    assert!(receipt.provenance_evidence().is_none());
    assert_eq!(
        receipt.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["identity.id", "title.value"]).as_slice()
    );
    assert_eq!(outcome.authoritative_residue_count(), 0);
}
