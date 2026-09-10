use super::*;

#[test]
fn nested_work_limits_preserve_consumption_and_restore_ceilings_on_error_and_unwind() {
    let mut work = RetainedStoragePreparation::new(20);
    work.reserve_visits(3).unwrap();
    {
        let mut outer = work.limit_additional_visits(8);
        assert!(!outer.outer_limited());
        outer.reserve_visits(2).unwrap();
        {
            let mut inner = outer.limit_additional_visits(usize::MAX);
            assert!(inner.outer_limited());
            inner.reserve_visits(6).unwrap();
            assert!(inner.visit().is_err());
        }
        assert_eq!(outer.visits(), 11);
        assert!(outer.visit().is_err());
    }
    assert_eq!((work.visits(), work.maximum_visits()), (11, 20));
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut limit = work.limit_additional_visits(4);
        limit.reserve_visits(3).unwrap();
        panic!("scope unwind");
    }));
    assert!(panic.is_err());
    assert_eq!((work.visits(), work.maximum_visits()), (14, 20));
    work.reserve_visits(6).unwrap();
    assert!(work.visit().is_err());
}
