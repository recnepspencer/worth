//! The installed managed path owns completion and original-request refusal.
use super::*;
use crate::domain_computation::primary_graph::{HandlerFailure, HandlerResult};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct Prepared(Vec<u8>);
struct Owner {
    cancel: Option<WorthQueryCancellationSource>,
    stop_in_completion: bool,
    completed: Arc<AtomicUsize>,
}
impl WorthQueryManagedComputationOwner<Schema, Feature, Computation> for Owner {
    type Prepared = Prepared;
    type Computed = Vec<u8>;
    type Output = usize;
    type Stopped = HandlerFailure<&'static str>;

    fn prepare(&self, input: &u64) -> Result<Prepared, Self::Stopped> {
        if *input == 0 {
            return Err(HandlerFailure::Domain("empty input"));
        }
        Ok(Prepared(vec![7; *input as usize]))
    }
    fn compute(
        &self,
        prepared: &Prepared,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Vec<u8>, WorthQueryManagedComputationDenial<Self::Stopped>> {
        checkpoint.poll()?;
        if !self.stop_in_completion
            && let Some(cancel) = &self.cancel
        {
            cancel.cancel();
            assert_eq!(
                checkpoint.poll(),
                Err(WorthQueryManagedComputationCheckpointDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                ))
            );
            // A child that ignores the original stop must not publish success.
        }
        Ok(prepared.0.clone())
    }
    fn complete(&self, prepared: Prepared, computed: Vec<u8>) -> Result<usize, Self::Stopped> {
        self.completed.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            prepared.0, computed,
            "the real completion validates its own output"
        );
        if self.stop_in_completion {
            self.cancel.as_ref().unwrap().cancel();
        }
        Ok(computed.len())
    }
}

fn run(
    input: u64,
    stop_in_compute: bool,
    stop_in_completion: bool,
    expired: bool,
) -> (HandlerResult<usize, &'static str>, usize) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(if expired {
            Instant::now() - Duration::from_secs(1)
        } else {
            Instant::now() + Duration::from_secs(60)
        }),
        cancellation.token(),
    );
    let completed = Arc::new(AtomicUsize::new(0));
    let owner = Owner {
        cancel: (stop_in_compute || stop_in_completion).then_some(cancellation),
        stop_in_completion,
        completed: completed.clone(),
    };
    let installed =
        WorthQueryInstalledManagedComputation::<Schema, Feature, Computation, _>::new(owner);
    let execution = QueryRequestExecution::open(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(
            crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(
                std::num::NonZeroUsize::MIN, None)), &scope);
    let result = installed
        .decision_plan(&input, HandlerResult::Completed)
        .run(&execution);
    (result, completed.load(Ordering::SeqCst))
}

#[test]
fn installed_decision_completes_native_payload_without_forecast_or_charged_bytes() {
    let (result, completed) = run(65, false, false, false);
    assert!(matches!(result, HandlerResult::Completed(65)));
    assert_eq!(completed, 1);
    // A fresh original request also admits a larger owner payload.
    let (result, completed) = run(513, false, false, false);
    assert!(matches!(result, HandlerResult::Completed(513)));
    assert_eq!(completed, 1);
}

#[test]
fn ignored_kernel_stop_keeps_original_cause_and_never_completes() {
    let (result, completed) = run(65, true, false, false);
    assert!(matches!(result, HandlerResult::Cancelled));
    assert_eq!(completed, 0);
}

#[test]
fn cancellation_at_owner_handoff_cannot_become_success() {
    let (result, completed) = run(65, false, true, false);
    assert!(matches!(result, HandlerResult::Cancelled));
    assert_eq!(completed, 1);
}

#[test]
fn prepare_domain_refusal_and_original_deadline_keep_distinct_topology() {
    let (result, completed) = run(0, false, false, false);
    assert!(matches!(result, HandlerResult::DomainDenied("empty input")));
    assert_eq!(completed, 0);
    let (result, completed) = run(65, false, false, true);
    assert!(matches!(result, HandlerResult::DeadlineExceeded));
    assert_eq!(completed, 0);
}
