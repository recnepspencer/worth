//! The cursor owns ordered application and destroys every unvisited slot.
pub use std::sync::Arc;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::canonical_apply::CanonicalApplyCursor;
pub use worth_proof::CanonicalUniqueVec;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::computation::prepared_request::PreparedCompute;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::QueryRequestExecution;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::failed;
pub use std::cell::Cell;
pub use std::cell::RefCell;
pub use std::rc::Rc;
pub use std::sync::Mutex;
pub use std::time::Duration;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::test_placement::test_policy;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;

pub struct ResultValue {
    pub rank: Cell<usize>,
    pub dropped: Arc<Mutex<Vec<usize>>>,
}
impl Drop for ResultValue {
    fn drop(&mut self) {
        self.dropped.lock().unwrap().push(self.rank.get());
    }
}

#[test]
fn every_compute_precedes_owner_completion_and_failure_destroys_the_suffix() {
    for failure in 0..3 {
        let cancellation = WorthQueryCancellationSource::new();
        let request = WorthQueryRequestScope::new(
            Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
            cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let execution = QueryRequestExecution::open_for_test(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
                std::num::NonZeroUsize::MIN,
                None,
            )),
            &request,
        );
        let computed = Arc::new(Mutex::new(Vec::new()));
        let dropped = Arc::new(Mutex::new(Vec::new()));
        // Rc is deliberately owner-only. Cell results are Send, not Sync.
        let completed = Rc::new(RefCell::new(Vec::new()));
        let mut plans = Vec::new();
        for rank in 0..3 {
            let computed = Arc::clone(&computed);
            let dropped = Arc::clone(&dropped);
            let completion = Rc::clone(&completed);
            plans.push(OwnerPlan::new_for_test(
                vec![PreparedCompute::new_for_test(move |member| {
                    assert_eq!(member.interruption(), None);
                    computed.lock().unwrap().push(rank);
                    ResultValue {
                        rank: Cell::new(rank),
                        dropped,
                    }
                })],
                move |mut slots| {
                    assert_eq!(slots.len(), 1);
                    let result = slots.pop().unwrap().into_value_for_test::<ResultValue>();
                    completion.borrow_mut().push(result.rank.get());
                    if result.rank.get() == failure {
                        Err(failure)
                    } else {
                        Ok(result.rank.get())
                    }
                },
            ));
        }
        let keys = CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2]).unwrap();
        let cursor = CanonicalApplyCursor::compute_for_test(keys, plans, &execution);
        assert_eq!(*computed.lock().unwrap(), vec![0, 1, 2]);
        assert!(completed.borrow().is_empty());

        let mut applied = Vec::new();
        assert_eq!(
            cursor.apply_for_test(
                |key, value| {
                    let value = value?;
                    assert_eq!(key, value, "canonical slots retain their semantic keys");
                    applied.push(value);
                    Ok(())
                },
                |_, denied| panic!("unexpected compute refusal: {denied:?}")
            ),
            Err(failure)
        );
        assert_eq!(applied, (0..failure).collect::<Vec<_>>());
        assert_eq!(*completed.borrow(), (0..=failure).collect::<Vec<_>>());
        let mut drops = dropped.lock().unwrap().clone();
        drops.sort_unstable();
        assert_eq!(
            drops,
            vec![0, 1, 2],
            "every result dies, including the inaccessible suffix"
        );
    }
}

#[path = "charge_prefix/mod.rs"]
pub mod charge_prefix;
#[path = "host_refusal/mod.rs"]
pub mod host_refusal;
#[path = "phase_failure/mod.rs"]
pub mod phase_failure;
#[path = "present_interruption/mod.rs"]
pub mod present_interruption;
#[path = "queue_outcomes/mod.rs"]
pub mod queue_outcomes;
