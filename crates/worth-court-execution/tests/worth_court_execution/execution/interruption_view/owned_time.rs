use super::*;
use std::time::Duration;
use worth_execution::MonotonicTimeController;

#[test]
fn leased_serial_kernel_and_retained_controls_use_their_owned_deadline_clocks() {
    for expire_parent in [false, true] {
        let parent_time = MonotonicTimeController::new().unwrap();
        let child_time = MonotonicTimeController::new().unwrap();
        let parent_now = parent_time.hold();
        let child_now = child_time.hold();
        let mut declared = lease_request();
        declared.deadline = Some(
            parent_time
                .clock()
                .deadline_at(parent_now + Duration::from_secs(20)),
        );
        let parent = ExecutionResourceLease::serial(declared).unwrap();
        let child = parent.controlled_child(
            CancellationSource::new().token(),
            Some(
                child_time
                    .clock()
                    .deadline_at(child_now + Duration::from_secs(10)),
            ),
        );
        let serial = SerialRequest::from_policy(
            child.policy(),
            CancellationSource::new().token(),
            ExecutionRequest::leased(&child).deadline(),
        );
        let mut retained = None;
        ExecutionRequest::leased(&child)
            .run_native_with_checkpoint(|work| {
                retained = Some(work.retain_interruption_view(Some(&child)).unwrap());
            })
            .unwrap();
        let retained = retained.unwrap();
        assert_eq!(ExecutionRequest::leased(&child).consult(), Ok(()));
        assert_eq!(ExecutionRequest::serial(&serial).consult(), Ok(()));
        assert_eq!(retained.check_live(), Ok(()));
        if expire_parent {
            parent_time.advance(Duration::from_secs(21)).unwrap();
        } else {
            child_time.advance(Duration::from_secs(11)).unwrap();
        }
        for execution in [
            ExecutionRequest::leased(&child),
            ExecutionRequest::serial(&serial),
        ] {
            assert_eq!(
                execution.consult(),
                Err(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed))
            );
        }
        assert_eq!(retained.check_live(), Err(MapKernelStop::DeadlineElapsed));
        let stopped = ExecutionRequest::leased(&child).run_native_with_checkpoint(|_| {
            panic!("an elapsed owned deadline must refuse before invoking a kernel")
        });
        assert_eq!(
            stopped.unwrap_err(),
            WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed)
        );
    }
}
