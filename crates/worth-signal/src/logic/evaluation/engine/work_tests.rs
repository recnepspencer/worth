use super::*;
use crate::data::graph::PendingRevalidationPreparationDenial;
use crate::data::retained_storage::RetainedStoragePreparationDenial;

fn reserve_waiter_visits(
    work: &mut RetainedStoragePreparation,
    visits: usize,
) -> Result<(), SignalError> {
    work.reserve_visits(visits)
        .map_err(|denial| PendingRevalidationPreparationDenial::Storage(denial).into_signal_error())
}

#[test]
fn absent_waiter_limit_keeps_the_attempt_counter_and_explicit_denial_owner() {
    let mut unbounded = RetainedStoragePreparation::new(None);
    unbounded.reserve_visits(7).unwrap();
    EvaluationWork::Conditional(&mut unbounded)
        .with_waiter_limit(None, |work| {
            assert_eq!(work.maximum_visits(), None);
            reserve_waiter_visits(work, 4 * 1_024 * 1_024 + 1)
        })
        .unwrap();
    assert_eq!(unbounded.visits(), 7 + 4 * 1_024 * 1_024 + 1);
    assert_eq!(unbounded.maximum_visits(), None);
    assert_eq!(
        EvaluationWork::Conditional(&mut unbounded)
            .with_waiter_limit(Some(1), |work| { reserve_waiter_visits(work, 2) }),
        Err(SignalError::WaiterResolutionWorkExhausted { maximum_visits: 1 })
    );
    assert_eq!(unbounded.maximum_visits(), None);

    let mut finite = RetainedStoragePreparation::new(Some(3));
    finite.reserve_visits(2).unwrap();
    assert_eq!(
        EvaluationWork::Conditional(&mut finite)
            .with_waiter_limit(None, |work| { reserve_waiter_visits(work, 2) }),
        Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits: 3 })
    );
    assert_eq!((finite.visits(), finite.maximum_visits()), (2, Some(3)));
}

#[test]
fn absent_waiter_limit_preserves_original_request_cancel_and_deadline() {
    use worth_execution::{
        CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest,
    };
    for expired in [false, true] {
        let cancellation = CancellationSource::new();
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(None),
            cancellation.token(),
            expired.then(std::time::Instant::now),
        );
        let request = ExecutionRequest::serial(&serial);
        if !expired {
            cancellation.cancel();
        }
        let expected = request.check_live().unwrap_err();
        let mut owner = RetainedStoragePreparation::new(None);
        owner.reserve_visits(7).unwrap();
        {
            let mut checkpoint = |_| {
                request
                    .check_live()
                    .map_err(|stop| RetainedStoragePreparationDenial::ExecutionStopped(stop.into()))
            };
            let mut observed = owner.reborrow_with_checkpoint(&mut checkpoint);
            assert_eq!(
                EvaluationWork::Conditional(&mut observed)
                    .with_waiter_limit(None, |work| { reserve_waiter_visits(work, 1) }),
                Err(SignalError::ExecutionCheckpointStopped(expected.into()))
            );
        }
        assert_eq!((owner.visits(), owner.maximum_visits()), (7, None));
    }
}
