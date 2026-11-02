use super::*;
use crate::{ExecutionRequest, MonotonicTimeController, SerialRequest, WorkCeilingDenial};

#[test]
fn leased_serial_kernel_and_retained_controls_use_their_owned_deadline_clocks() {
    let _serial = TEST_LOCK.lock().unwrap();
    for expire_parent in [false, true] {
        let parent_time = MonotonicTimeController::new().unwrap();
        let child_time = MonotonicTimeController::new().unwrap();
        let parent_now = parent_time.hold();
        let child_now = child_time.hold();
        let mut declared = request(1, 32_000, 100);
        declared.deadline = Some(parent_time.clock().deadline_at(parent_now + Duration::from_secs(20)));
        let parent = authority().request_lease(declared).unwrap();
        let child = parent.controlled_child(CancellationToken::new(),
            Some(child_time.clock().deadline_at(child_now + Duration::from_secs(10))));
        let serial = SerialRequest::from_policy(child.policy(), CancellationToken::new(), child.deadline());
        let retained = crate::backend::run_scope_within::<_, (), _>(
            Some(&child), 0, 0, true, Some(100), None, |work| {
                Ok(work.retain_interruption_view(Some(&child)).unwrap())
            }).result.unwrap();
        assert_eq!(ExecutionRequest::leased(&child).consult(), Ok(()));
        assert_eq!(ExecutionRequest::serial(&serial).consult(), Ok(()));
        assert_eq!(retained.check_live(), Ok(()));
        if expire_parent {
            parent_time.advance(Duration::from_secs(21)).unwrap();
        } else {
            child_time.advance(Duration::from_secs(11)).unwrap();
        }
        for execution in [ExecutionRequest::leased(&child), ExecutionRequest::serial(&serial)] {
            assert_eq!(execution.consult(), Err(WorkCeilingDenial::Stopped(KernelStop::DeadlineElapsed)));
        }
        assert_eq!(retained.check_live(), Err(KernelStop::DeadlineElapsed));
        let stopped = crate::backend::run_scope_within::<(), (), _>(
            Some(&child), 0, 0, true, Some(100), None,
            |_| panic!("an elapsed owned deadline must refuse before invoking a kernel"));
        assert!(matches!(stopped.result,
            Err(crate::backend::ScopeStop::Failure(crate::backend::KernelFailure::Stop(KernelStop::DeadlineElapsed)))));
    }
}
