//! Projection is bound to the actual current receiver and worker ancestry.
use super::*;

fn unrelated(result: Result<ExecutionInterruptionView, ExecutionInterruptionViewDenial>) -> bool {
    matches!(
        result,
        Err(ExecutionInterruptionViewDenial::Request(
            WorkCeilingDenial::Admission(worth_execution::LeaseDenial::UnrelatedNestedLease)
        ))
    )
}

#[test]
fn root_projection_uses_the_active_child_controls_and_memory() {
    let root = ExecutionResourceLease::serial(lease_request()).unwrap();
    let cancellation = CancellationSource::new();
    let child = root.controlled_child(cancellation.token(), None);
    let mut view = None;
    ExecutionRequest::leased(&child)
        .run_native_with_checkpoint(|context| {
            view = Some(context.retain_interruption_view(Some(&root)).unwrap());
        })
        .unwrap();
    assert!(child.charged_memory_bytes() > 0);
    assert_eq!(child.charged_memory_bytes(), root.charged_memory_bytes());
    cancellation.cancel();
    assert_eq!(
        view.as_ref().unwrap().check_live(),
        Err(MapKernelStop::Cancelled)
    );
    drop(view);
    assert_eq!(child.charged_memory_bytes(), 0);
    assert_eq!(root.charged_memory_bytes(), 0);
}

#[test]
fn foreign_sibling_and_unentered_child_refuse_before_cancelled_context_observation() {
    enum RefusedLineage {
        Foreign,
        Sibling,
        UnenteredChild,
    }
    for kind in [
        RefusedLineage::Foreign,
        RefusedLineage::Sibling,
        RefusedLineage::UnenteredChild,
    ] {
        let root = ExecutionResourceLease::serial(lease_request()).unwrap();
        let cancellation = CancellationSource::new();
        let active = root.controlled_child(cancellation.token(), None);
        let refused = match kind {
            RefusedLineage::Foreign => ExecutionResourceLease::serial(lease_request()).unwrap(),
            RefusedLineage::Sibling => root.child(lease_request()).unwrap(),
            RefusedLineage::UnenteredChild => active.child(lease_request()).unwrap(),
        };
        let mut observed = false;
        let mut no_retention = false;
        let outer = ExecutionRequest::leased(&active).run_native_with_checkpoint(|context| {
            let before = active.charged_memory_bytes();
            cancellation.cancel();
            observed = unrelated(context.retain_interruption_view(Some(&refused)));
            no_retention = active.charged_memory_bytes() == before;
        });
        assert!(observed && no_retention);
        assert!(matches!(
            outer,
            Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
        ));
    }
}

#[test]
fn leased_receiver_requires_its_original_backing() {
    let lease = ExecutionResourceLease::serial(lease_request()).unwrap();
    let mut observed = false;
    let outer = ExecutionRequest::leased(&lease).run_native_with_checkpoint(|context| {
        observed = unrelated(context.retain_interruption_view(None));
    });
    assert!(observed);
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
    ));
}

#[test]
fn serial_receiver_cannot_take_a_foreign_lease_for_projection() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let lease = ExecutionResourceLease::serial(lease_request()).unwrap();
    let mut observed = false;
    let outer = ExecutionRequest::serial(&request).run_native_with_checkpoint(|context| {
        observed = unrelated(context.retain_interruption_view(Some(&lease)));
    });
    assert!(observed);
    assert_eq!(lease.charged_memory_bytes(), 0);
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
    ));
}

#[test]
fn saved_parent_receiver_refuses_while_another_meter_is_current() {
    let lease = ExecutionResourceLease::serial(lease_request()).unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let mut observed = false;
    let mut inner_stopped = false;
    let outer = execution.run_native_with_checkpoint(|parent| {
        let inner = execution.run_native_with_checkpoint(|_| {
            observed = unrelated(parent.retain_interruption_view(Some(&lease)));
        });
        inner_stopped = matches!(
            inner,
            Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
        );
    });
    assert!(observed && inner_stopped);
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
    ));
}

#[test]
fn supplied_child_can_project_only_after_its_real_scope_is_entered() {
    let root = ExecutionResourceLease::serial(lease_request()).unwrap();
    let child = root.child(lease_request()).unwrap();
    let mut view = None;
    ExecutionRequest::leased(&root)
        .run_native_with_checkpoint(|_| {
            ExecutionRequest::leased(&child)
                .run_native_with_checkpoint(|context| {
                    view = Some(context.retain_interruption_view(Some(&child)).unwrap());
                })
                .unwrap();
        })
        .unwrap();
    assert_eq!(view.unwrap().check_live(), Ok(()));
}
