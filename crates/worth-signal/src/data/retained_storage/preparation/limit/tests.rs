use super::*;

#[test]
fn nested_work_limits_preserve_consumption_and_restore_ceilings_on_error_and_unwind() {
    let mut work = RetainedStoragePreparation::new(Some(20));
    work.reserve_visits(3).unwrap();
    {
        let mut outer = work.limit_additional_visits(8).unwrap();
        assert!(!outer.outer_limited());
        outer.reserve_visits(2).unwrap();
        {
            let mut inner = outer.limit_additional_visits(usize::MAX).unwrap();
            assert!(inner.outer_limited());
            inner.reserve_visits(6).unwrap();
            assert!(inner.visit().is_err());
        }
        assert_eq!(outer.visits(), 11);
        assert!(outer.visit().is_err());
    }
    assert_eq!((work.visits(), work.maximum_visits()), (11, Some(20)));
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut limit = work.limit_additional_visits(4).unwrap();
        limit.reserve_visits(3).unwrap();
        panic!("scope unwind");
    }));
    assert!(panic.is_err());
    assert_eq!((work.visits(), work.maximum_visits()), (14, Some(20)));
    work.reserve_visits(6).unwrap();
    assert!(work.visit().is_err());
}

#[test]
fn request_checkpoints_precede_visits_and_preserve_consumption_on_error_and_unwind() {
    let mut work = RetainedStoragePreparation::new(Some(20));
    work.reserve_visits(3).unwrap();
    let mut charged = 0;
    {
        let mut checkpoint = |units| {
            charged += units;
            if charged > 8 {
                Err(
                    super::super::RetainedStoragePreparationDenial::WorkExhausted {
                        maximum_visits: 8,
                    },
                )
            } else {
                Ok(())
            }
        };
        let mut observed = work.reborrow_with_checkpoint(&mut checkpoint);
        observed.reserve_visits(2).unwrap();
        observed.checkpoint_request_only(5).unwrap();
        assert!(observed.reserve_visits(2).is_err());
        assert_eq!(observed.visits(), 5);
    }
    assert_eq!(charged, 9);
    assert_eq!((work.visits(), work.maximum_visits()), (5, Some(20)));
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut checkpoint = |units| {
            charged += units;
            Ok(())
        };
        let mut observed = work.reborrow_with_checkpoint(&mut checkpoint);
        let mut limited = observed.limit_additional_visits(4).unwrap();
        limited.reserve_visits(3).unwrap();
        panic!("request scope unwind");
    }));
    assert!(panic.is_err());
    assert_eq!(charged, 12);
    assert_eq!((work.visits(), work.maximum_visits()), (8, Some(20)));
}

#[test]
fn absent_attempt_ceiling_preserves_narrow_limits_consumption_and_original_stops() {
    use super::super::RetainedStoragePreparationDenial as Denial;
    use crate::data::error::SignalCheckpointDenial;
    let maximum = crate::runtime_policy::SignalConditionalEvaluationBudget::development()
        .maximum_attempt_visits;
    let mut work = RetainedStoragePreparation::new(maximum);
    work.reserve_visits(8_000_001).unwrap();
    {
        let mut limited = work.limit_additional_visits(2).unwrap();
        assert!(!limited.outer_limited());
        limited.visit().unwrap();
        assert!(limited.reserve_visits(2).is_err());
    }
    assert_eq!(work.maximum_visits(), None);
    assert_eq!(work.visits(), 8_000_002);
    for cause in [
        SignalCheckpointDenial::Cancelled,
        SignalCheckpointDenial::DeadlineElapsed,
    ] {
        let before = work.visits();
        let mut checkpoint = |_| Err(Denial::ExecutionStopped(cause));
        let mut observed = work.reborrow_with_checkpoint(&mut checkpoint);
        assert_eq!(observed.visit(), Err(Denial::ExecutionStopped(cause)));
        assert_eq!(observed.visits(), before);
    }
    assert_eq!(work.reserve_visits(usize::MAX), Err(Denial::ChargeOverflow));
    assert_eq!(work.visits(), 8_000_002);
}
