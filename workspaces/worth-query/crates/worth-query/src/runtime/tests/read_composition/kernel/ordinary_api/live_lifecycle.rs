use std::cell::Cell;
use std::rc::Rc;

use super::live::{closed, open_task_resource, task_workspace};
use crate::ordinary::live::WorthQueryManagedLiveLifecyclePosture;
use crate::runtime::tests::support::{
    complete_backend_from_parts_builder, insert_command, test_string_aspect_value,
    CountingSourceAdapter, CountingWriteAuthority,
};

#[test]
fn managed_live_observation_tracks_delivery_without_exposing_lifecycle_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_workspace(request_execution, "managed-live-observation");
    let handle = open_task_resource(request_execution, &mut workspace, "tasks.observed");
    let opened = handle
        .observe(&mut workspace)
        .expect("opened managed resource should be observable");
    assert_eq!(
        opened.posture(),
        WorthQueryManagedLiveLifecyclePosture::Active
    );
    assert_eq!(opened.pending_delivery_batch_count(), 0);
    assert_eq!(opened.last_delivery_sequence(), None);

    workspace
        .write(request_execution, task_insert("Observed"))
        .expect("relevant write should route to observed resource");
    let delivered = handle
        .observe(&mut workspace)
        .expect("delivery posture should remain observable");
    assert_eq!(delivered.pending_delivery_batch_count(), 1);
    assert_eq!(delivered.last_delivery_sequence(), Some(1));

    handle
        .drain(&mut workspace)
        .expect("observed delivery should drain");
    let drained = handle
        .observe(&mut workspace)
        .expect("drained resource should remain active");
    assert_eq!(drained.pending_delivery_batch_count(), 0);
    assert_eq!(drained.last_delivery_sequence(), Some(1));
    assert!(closed(handle.close(&mut workspace)).lane_terminal());
}

#[test]
fn abandoned_handle_is_retried_before_write_and_cannot_receive_delivery() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declared = Rc::new(Cell::new(0));
    let closed_count = Rc::new(Cell::new(0));
    let close_denied = Rc::new(Cell::new(true));
    let attempted_writes = Rc::new(Cell::new(0));
    let mut workspace = lifecycle_workspace(
        request_execution,
        &declared,
        &closed_count,
        &close_denied,
        &attempted_writes,
    );
    let handle = open_task_resource(request_execution, &mut workspace, "tasks.abandoned");
    drop(handle);

    assert!(workspace
        .resolve_live_artifact_target("tasks.abandoned")
        .is_err());
    assert!(workspace
        .write(request_execution, task_insert("Denied cleanup"))
        .is_err());
    assert_eq!(closed_count.get(), 0);
    assert_eq!(attempted_writes.get(), 0);

    close_denied.set(false);
    let receipt = workspace
        .write(request_execution, task_insert("After cleanup"))
        .expect("write should retry and complete abandoned-resource cleanup first");
    assert_eq!(closed_count.get(), 1);
    assert_eq!(attempted_writes.get(), 1);
    assert!(receipt
        .terminal_affected_live_view_ids_projection()
        .is_empty());

    let replacement = open_task_resource(request_execution, &mut workspace, "tasks.abandoned");
    assert_eq!(declared.get(), 2);
    assert!(closed(replacement.close(&mut workspace)).lane_terminal());
    assert_eq!(closed_count.get(), 2);
}

#[test]
fn runtime_shutdown_closes_live_resources_even_when_handles_still_exist() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declared = Rc::new(Cell::new(0));
    let closed_count = Rc::new(Cell::new(0));
    let close_denied = Rc::new(Cell::new(false));
    let attempted_writes = Rc::new(Cell::new(0));
    let mut workspace = lifecycle_workspace(
        request_execution,
        &declared,
        &closed_count,
        &close_denied,
        &attempted_writes,
    );
    let handle = open_task_resource(request_execution, &mut workspace, "tasks.runtime-shutdown");

    drop(workspace);
    assert_eq!(closed_count.get(), 1);
    drop(handle);
    assert_eq!(closed_count.get(), 1);
}

fn lifecycle_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    declared: &Rc<Cell<usize>>,
    closed: &Rc<Cell<usize>>,
    close_denied: &Rc<Cell<bool>>,
    attempted_writes: &Rc<Cell<usize>>,
) -> crate::runtime::WorthQueryWorkspace {
    complete_backend_from_parts_builder()
        .source_adapter(CountingSourceAdapter::lifecycle_counting(
            Rc::clone(declared),
            Rc::clone(closed),
            Rc::clone(close_denied),
        ))
        .write_authority(CountingWriteAuthority {
            attempted_writes: Rc::clone(attempted_writes),
        })
        .build_backend_from_parts()
        .build(request_execution)
        .expect("lifecycle-counting runtime should build")
        .workspace("managed-live-lifecycle")
        .expect("lifecycle-counting workspace should open")
}

fn task_insert(title: &str) -> crate::runtime::WorthQueryWriteCommand {
    insert_command(
        "Task",
        [
            ("identity.id", test_string_aspect_value("")),
            ("title.value", test_string_aspect_value(title)),
        ],
    )
}
