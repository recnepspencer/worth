use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::application::scan_shared_read_mint_forbidden_patterns;
use crate::projection_consumption::ProjectionAuthorityOutcome;

use super::shared_read_support::*;
use super::support::*;

#[test]
fn shared_read_context_consumes_published_projection_facts_through_typed_artifact_handles() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = shared_read_workspace(request_execution, "shared-read.current");
    let invocations = Arc::new(AtomicUsize::new(0));
    let derived = declare_shared_read_derived(
        request_execution,
        &mut workspace,
        "shared-read",
        SharedReadPublishingMaintainer {
            invocations: Arc::clone(&invocations),
            mode: SharedReadPublicationMode::RefreshTitle("Task One"),
        },
    );
    insert_task(request_execution, &mut workspace, "task-1", "Task One");

    let artifact = workspace
        .shared_read_context(request_execution)
        .expect("shared read context should mint")
        .published_derived_artifact(&derived)
        .expect("published artifact handle should mint");
    let consumption = consume_display_title_attempt(&artifact);

    match consumption {
        WorthQueryPublishedProjectionAuthorityOutcome::Current(
            ProjectionAuthorityOutcome::Admitted(completed),
        ) => {
            assert_eq!(completed.receipt().extracted_fact_count(), 1);
            assert_eq!(
                artifact
                    .inspect_projection_consumption()
                    .async_result_state()
                    .expect("republishing posture should stay visible")
                    .kind(),
                WorthQueryRuntimeAsyncResultStateKind::Revalidating
            );
            assert_eq!(consume_display_title(&artifact), "Task One");
        }
        other => panic!("expected published fact consumption, got {other:?}"),
    }

    assert_eq!(
        invocations.load(Ordering::SeqCst),
        1,
        "shared-read consumption must not trigger derived reevaluation"
    );
}

#[test]
fn shared_read_unpublished_artifact_fails_closed_with_typed_pending_state() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = shared_read_workspace(request_execution, "shared-read.pending");
    let derived = declare_shared_read_derived(
        request_execution,
        &mut workspace,
        "shared-read.pending",
        SharedReadPublishingMaintainer {
            invocations: Arc::new(AtomicUsize::new(0)),
            mode: SharedReadPublicationMode::RefreshTitle("Task One"),
        },
    );
    let unpublished = workspace
        .shared_read_context(request_execution)
        .expect("shared read context should mint")
        .published_derived_artifact(&derived)
        .expect("declared unpublished derived view should still mint a typed handle");

    match consume_display_title_attempt(&unpublished) {
        WorthQueryPublishedProjectionAuthorityOutcome::ResultState(state) => {
            assert_eq!(state.kind(), WorthQueryRuntimeAsyncResultStateKind::Pending);
            assert_eq!(
                state.basis_for_reporting(),
                unpublished
                    .snapshot_identity()
                    .evidence_identity()
                    .terminal_projection_for_reporting()
            );
        }
        other => panic!("expected pending async posture, got {other:?}"),
    }
}

#[test]
fn shared_read_foreign_derived_handle_denies_instead_of_masking_as_pending() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = shared_read_workspace(request_execution, "shared-read.foreign");
    let read_ctx = workspace
        .shared_read_context(request_execution)
        .expect("shared read context should mint");
    let mut foreign_workspace =
        shared_read_workspace(request_execution, "shared-read.foreign-source");
    let foreign = declare_shared_read_derived(
        request_execution,
        &mut foreign_workspace,
        "shared-read.foreign",
        SharedReadPublishingMaintainer {
            invocations: Arc::new(AtomicUsize::new(0)),
            mode: SharedReadPublicationMode::RefreshTitle("Task One"),
        },
    );

    let error = read_ctx
        .published_derived_artifact(&foreign)
        .expect_err("foreign derived handles must not masquerade as pending publication");
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingDerivedView(_)
    ));
}

#[test]
fn shared_read_empty_published_artifact_stays_published_instead_of_pending() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = shared_read_workspace(request_execution, "shared-read.empty-published");
    let derived = declare_shared_read_derived(
        request_execution,
        &mut workspace,
        "shared-read.empty",
        SharedReadPublishingMaintainer {
            invocations: Arc::new(AtomicUsize::new(0)),
            mode: SharedReadPublicationMode::EmptyRefresh,
        },
    );
    insert_task(request_execution, &mut workspace, "task-1", "Task One");

    let artifact = workspace
        .shared_read_context(request_execution)
        .expect("shared read context should mint")
        .published_derived_artifact(&derived)
        .expect("published empty artifact handle should mint");
    let inspection = artifact.inspect_projection_consumption();

    assert!(inspection.published());
    assert!(artifact.published_binding().is_some());
    assert_ne!(
        inspection
            .async_result_state()
            .expect("published empty artifact should still retain republication posture")
            .kind(),
        WorthQueryRuntimeAsyncResultStateKind::Pending
    );
}

#[test]
fn shared_read_incremental_republication_preserves_stale_async_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = shared_read_workspace(request_execution, "shared-read.stale");
    let invocations = Arc::new(AtomicUsize::new(0));
    let derived = declare_shared_read_derived(
        request_execution,
        &mut workspace,
        "shared-read.stale",
        SharedReadPublishingMaintainer {
            invocations: Arc::clone(&invocations),
            mode: SharedReadPublicationMode::IncrementalTitle("Task One"),
        },
    );
    insert_task(request_execution, &mut workspace, "task-1", "Task One");

    let artifact = workspace
        .shared_read_context(request_execution)
        .expect("shared read context should mint")
        .published_derived_artifact(&derived)
        .expect("stale artifact handle should mint");

    assert_eq!(
        artifact
            .inspect_projection_consumption()
            .async_result_state()
            .expect("incremental republication should surface async posture")
            .kind(),
        WorthQueryRuntimeAsyncResultStateKind::Stale
    );
    assert_eq!(consume_display_title(&artifact), "Task One");
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
}

#[test]
fn shared_read_republication_keeps_old_context_on_old_artifact_and_new_context_on_new_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = shared_read_workspace(request_execution, "shared-read.republication");
    let invocations = Arc::new(AtomicUsize::new(0));
    let derived = declare_shared_read_derived(
        request_execution,
        &mut workspace,
        "shared-read.republication",
        SharedReadPublishingMaintainer {
            invocations: Arc::clone(&invocations),
            mode: SharedReadPublicationMode::SequencedRefresh(&["Task One", "Task Two"]),
        },
    );
    insert_task(request_execution, &mut workspace, "task-1", "Task One");
    let old_artifact = workspace
        .shared_read_context(request_execution)
        .expect("old shared read context should mint")
        .published_derived_artifact(&derived)
        .expect("old artifact handle should mint");

    insert_task(request_execution, &mut workspace, "task-2", "Task Two");
    let new_artifact = workspace
        .shared_read_context(request_execution)
        .expect("new shared read context should mint")
        .published_derived_artifact(&derived)
        .expect("new artifact handle should mint");

    assert_eq!(consume_display_title(&old_artifact), "Task One");
    assert_eq!(consume_display_title(&new_artifact), "Task Two");
    assert_ne!(
        old_artifact
            .published_binding()
            .expect("old artifact should stay published")
            .binding_for_reporting(),
        new_artifact
            .published_binding()
            .expect("new artifact should stay published")
            .binding_for_reporting()
    );
    assert_eq!(invocations.load(Ordering::SeqCst), 2);
}

#[test]
fn shared_read_context_fails_closed_when_runtime_retires_its_snapshot_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = shared_read_workspace(request_execution, "shared-read.stale-basis");
    let derived = declare_shared_read_derived(
        request_execution,
        &mut workspace,
        "shared-read.stale-basis",
        SharedReadPublishingMaintainer {
            invocations: Arc::new(AtomicUsize::new(0)),
            mode: SharedReadPublicationMode::RefreshTitle("Task One"),
        },
    );
    insert_task(request_execution, &mut workspace, "task-1", "Task One");

    let read_ctx = workspace
        .shared_read_context(request_execution)
        .expect("shared read context should mint");
    let expected_snapshot_identity = read_ctx.snapshot_identity().clone();
    workspace
        .runtime
        .invalidate_shared_read_snapshot_for_certification(&expected_snapshot_identity);

    let error = read_ctx
        .published_derived_artifact(&derived)
        .expect_err("retired shared-read basis must fail closed");
    match error.stop_class() {
        WorthQueryStopClass::SharedReadStaleBasis { snapshot_identity } => {
            assert_eq!(snapshot_identity, &expected_snapshot_identity);
        }
        other => panic!("expected shared-read stale-basis stop class, got {other:?}"),
    }
}

#[test]
fn shared_read_pin_counters_return_to_exact_zero_after_context_and_artifact_drop() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = shared_read_workspace(request_execution, "shared-read.pin-counts");
    let derived = declare_shared_read_derived(
        request_execution,
        &mut workspace,
        "shared-read.pin-counts",
        SharedReadPublishingMaintainer {
            invocations: Arc::new(AtomicUsize::new(0)),
            mode: SharedReadPublicationMode::RefreshTitle("Task One"),
        },
    );
    insert_task(request_execution, &mut workspace, "task-1", "Task One");

    {
        let read_ctx = workspace
            .shared_read_context(request_execution)
            .expect("shared read context should mint");
        let artifact = read_ctx
            .published_derived_artifact(&derived)
            .expect("published artifact should mint");
        assert!(artifact.published_binding().is_some());
        assert_eq!(
            workspace
                .runtime
                .shared_read_counters()
                .unretired_pin_count(),
            2
        );
        assert_eq!(
            workspace
                .runtime
                .shared_read_counters()
                .shared_read_mint_row_clone_count(),
            0
        );
        assert_eq!(
            workspace
                .runtime
                .shared_read_counters()
                .reader_derived_evaluation_count(),
            0
        );
        assert_eq!(
            workspace
                .runtime
                .shared_read_counters()
                .published_artifact_registry_lease_count(),
            1
        );
    }

    let counters = workspace.runtime.shared_read_counters();
    assert_eq!(counters.unretired_pin_count(), 0);
    assert_eq!(counters.orphaned_generation_count(), 0);
}

#[test]
fn shared_read_pinning_inventory_rejects_mint_time_row_clone_patterns() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve");

    let failures = scan_shared_read_mint_forbidden_patterns(workspace_root);

    assert!(
        failures.is_empty(),
        "shared-read mint forbidden patterns must stay absent: {failures:?}"
    );
}
