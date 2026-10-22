//! The cursor owns ordered application and destroys every unvisited slot.
use super::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use crate::domain_computation::primary_graph::application_contribution::computation::prepared_request::PreparedCompute;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

struct ResultValue {
    rank: Cell<usize>,
    dropped: Arc<Mutex<Vec<usize>>>,
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
            Some(Instant::now() + Duration::from_secs(60)),
            cancellation.token(),
        );
        let execution = QueryRequestExecution::open(
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
            plans.push(OwnerPlan::new(
                vec![PreparedCompute::new(move |member| {
                    assert_eq!(member.interruption(), None);
                    computed.lock().unwrap().push(rank);
                    ResultValue {
                        rank: Cell::new(rank),
                        dropped,
                    }
                })],
                move |mut slots| {
                    assert_eq!(slots.len(), 1);
                    let result = slots.pop().unwrap().into_value::<ResultValue>();
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
        let cursor = CanonicalApplyCursor::compute(keys, plans, &execution);
        assert_eq!(*computed.lock().unwrap(), vec![0, 1, 2]);
        assert!(completed.borrow().is_empty());

        let mut applied = Vec::new();
        assert_eq!(
            cursor.apply(
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

mod phase_failure;

mod host_refusal;

mod queue_outcomes;

mod present_interruption;

mod charge_prefix;
