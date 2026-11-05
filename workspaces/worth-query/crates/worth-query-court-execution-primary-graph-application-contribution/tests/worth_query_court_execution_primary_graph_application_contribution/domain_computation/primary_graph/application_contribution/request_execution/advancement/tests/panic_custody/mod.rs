//! Pin caller custody at refusal and native result destruction boundaries.
use crate::domain_computation::primary_graph::application_contribution::request_execution::advancement::tests::*;
use std::cell::Cell;

pub struct CaptureDrop<'a>(&'a Cell<usize>);
impl Drop for CaptureDrop<'_> {
    fn drop(&mut self) {
        self.0
            .set(worth_query_execution::facade::domain_computation::advancement_observations_on_this_thread_for_test().len());
    }
}

#[test]
fn refused_capture_drops_before_the_result_observation() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let seen = Cell::new(usize::MAX);
    let capture = CaptureDrop(&seen);
    let mut root = request();
    (*root.declared_work_mut_for_test()) = Some(0);
    let refused = root.run_advancement_for_test(None, None, move |_| {
        drop(capture);
        panic!("zero work cannot invoke the body")
    });
    assert_eq!(
        refused,
        Err(WorthQueryAdvancementDenial::Resource(
            Resource::WorkExhausted
        ))
    );
    assert_eq!(
        seen.get(),
        0,
        "pending callback drops before result recording"
    );
    assert_eq!(
        worth_query_execution::facade::domain_computation::advancement_observations_on_this_thread_for_test().len(),
        1
    );
}

pub struct ResultDrop<'a> {
    pub drops: &'a Cell<usize>,
    pub panic: bool,
}
impl Drop for ResultDrop<'_> {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        assert!(!self.panic, "original native result destructor");
    }
}

#[test]
fn native_late_refusal_contains_result_drop_panic_and_preserves_cancellation() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let policy = worth_foundational::ExecutionRequestPolicy::for_workers(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        NonZeroUsize::MIN,
    );
    let cancellation = worth_execution::CancellationSource::new();
    let root = QueryRequestExecution::open_control_for_test(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy),
        cancellation.token(),
        None,
    );
    assert!(!root.has_accounting_for_test());
    let drops = Cell::new(0);
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        root.run_advancement_for_test(None, None, |_| {
            cancellation.cancel();
            ResultDrop {
                drops: &drops,
                panic: true,
            }
        })
    }));
    let refused = caught
        .expect("Execution contains the result destructor panic")
        .err();
    assert_eq!(
        refused,
        Some(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled,
        ))
    );
    assert_eq!(drops.get(), 1);
    let records = worth_query_execution::facade::domain_computation::advancement_observations_on_this_thread_for_test();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].refusal, refused);
    assert_eq!(
        request().run_advancement_for_test(None, None, |_| 19),
        Ok(19)
    );
    assert_eq!(
        worth_query_execution::facade::domain_computation::advancement_observations_on_this_thread_for_test().len(),
        1
    );
}

#[test]
fn accounted_completion_keeps_its_value_when_the_body_cancels_the_request() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let cancellation = worth_execution::CancellationSource::new();
    let root = QueryRequestExecution::open_control_for_test(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy(NonZeroUsize::MIN, None),
        ),
        cancellation.token(),
        None,
    );
    assert!(root.has_accounting_for_test());
    let drops = Cell::new(0);
    let returned = root
        .run_advancement_for_test(None, None, |_| {
            cancellation.cancel();
            ResultDrop {
                drops: &drops,
                panic: false,
            }
        })
        .expect("accounted advancement preserves the value whenever the body ran");
    assert_eq!(drops.get(), 0);
    drop(returned);
    assert_eq!(drops.get(), 1);
    let records = worth_query_execution::facade::domain_computation::advancement_observations_on_this_thread_for_test();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].refusal, None);
}

pub fn panicking_value(payload: std::sync::Arc<String>) -> u64 {
    std::panic::panic_any(payload)
}

#[test]
fn body_panic_leaves_the_slot_empty_and_resumes_the_identical_payload() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let root = request();
    let slot = std::cell::RefCell::new(None::<u64>);
    let payload = std::sync::Arc::new(String::from("original advancement panic payload"));
    let mut write = |_: WorthQueryAdvancementPhase<'_>| {
        let value = panicking_value(std::sync::Arc::clone(&payload));
        *slot.borrow_mut() = Some(value);
    };
    let mut discard = || {
        *slot.borrow_mut() = None;
    };
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::runner::run_advancement(&root, None, None, &mut write, &mut discard)
    }))
    .expect_err("the original body panic resumes rather than becoming a slot failure");
    assert!(slot.borrow().is_none());
    let resumed = caught
        .downcast::<std::sync::Arc<String>>()
        .expect("the original payload type survives");
    assert!(std::sync::Arc::ptr_eq(&resumed, &payload));
    let records = worth_query_execution::facade::domain_computation::advancement_observations_on_this_thread_for_test();
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].refusal,
        Some(WorthQueryAdvancementDenial::Panicked)
    );
    assert_eq!(root.run_advancement_for_test(None, None, |_| 23), Ok(23));
    assert_eq!(
        worth_query_execution::facade::domain_computation::advancement_observations_on_this_thread_for_test().len(),
        1
    );
}
