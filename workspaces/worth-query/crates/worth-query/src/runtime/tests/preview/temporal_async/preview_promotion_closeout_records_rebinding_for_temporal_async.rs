//! preview promotion closeout records rebinding for temporal async and mixed handles.
use super::*;

#[test]
fn preview_promotion_closeout_records_rebinding_for_temporal_async_and_mixed_handles() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let temporal_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.promote-temporal",
            task_live_request(),
            task_schema(),
        )
        .expect("temporal live view should declare");
    let async_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.promote-async",
            task_live_request(),
            task_schema(),
        )
        .expect("async live view should declare");
    let mixed_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.promote-mixed",
            task_live_request(),
            task_schema(),
        )
        .expect("mixed live view should declare");
    runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("authoritative target live view should declare");
    install_temporal_async_and_mixed_residue(
        &mut runtime,
        &temporal_view,
        &async_view,
        &mixed_view,
        resource_request,
    );

    let outcome = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("preview temporal-async promote"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should admit");
        preview.use_view(&temporal_view);
        preview.use_view(&async_view);
        preview.use_view(&mixed_view);
        preview
            .write(
                request_execution,
                insert_command(
                    "Task",
                    [
                        (
                            "identity.id",
                            test_string_aspect_value("preview-promote-temporal-async"),
                        ),
                        (
                            "title.value",
                            test_string_aspect_value("Promoted preview residue task"),
                        ),
                    ],
                ),
            )
            .expect("preview write should stage");
        preview
            .promote(request_execution)
            .expect("preview promotion should succeed")
    };

    let closeout = outcome.closeout_evidence();
    assert!(outcome.promoted());
    assert_eq!(closeout.temporal_wake_residue_count(), 1);
    assert_eq!(closeout.async_result_residue_count(), 1);
    assert_eq!(closeout.mixed_cause_residue_count(), 1);
    assert_eq!(closeout.crossed_authoritative_residue_count(), 0);
    assert!(closeout.rebinding_digest().is_some());
    assert!(!closeout
        .preview_basis_snapshot_identity()
        .evidence_identity()
        .as_str()
        .is_empty());
    assert!(!closeout
        .target_basis_snapshot_identity()
        .evidence_identity()
        .as_str()
        .is_empty());

    let inspection = runtime
        .inspect_preview_outcome(request_execution, &outcome)
        .expect("preview outcome inspection should succeed");
    assert!(inspection.rebinding_digest().is_some());
    assert_eq!(
        inspection.preview_basis_snapshot_identity(),
        closeout.preview_basis_snapshot_identity()
    );
    assert_eq!(
        inspection.target_basis_snapshot_identity(),
        closeout.target_basis_snapshot_identity()
    );
    assert_eq!(inspection.temporal_wake_residue_count(), 1);
    assert_eq!(inspection.async_result_residue_count(), 1);
    assert_eq!(inspection.mixed_cause_residue_count(), 1);
}
