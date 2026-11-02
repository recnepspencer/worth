//! Actual original-root observation excludes detached diagnostic meters.
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseDenial,
    RequestConsultObservationForTest as Observation, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};
fn serial() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}
#[test]
fn scoped_consult_observation_keeps_original_children_and_excludes_detached_diagnostics() {
    let request = serial();
    let execution = ExecutionRequest::serial(&request);
    let mut saved = None;
    let ((), report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&request), |root| {
            let observed =
                Observation::install_in_scope_on_this_thread_for_test(root, |_| {}).unwrap();
            root.checkpoint(3).unwrap();
            execution
                .run_native_with_checkpoint(|child| {
                    for context in [&*child, &*root] {
                        assert_eq!(
                            Observation::install_in_scope_on_this_thread_for_test(context, |_| {})
                                .err(),
                            Some(WorkCeilingDenial::Admission(
                                LeaseDenial::NoActiveExecutionScope
                            ))
                        );
                    }
                    assert!(observed.is_current_scope_observed_for_test());
                    child.account_completed_work(2).unwrap();
                    child.checkpoint(3).unwrap();
                })
                .unwrap();
            let count = observed.count_for_test();
            let prefixes = observed.charged_prefixes_for_test();
            execution
                .run_native_detached_for_test(|_| {
                    assert!(!observed.is_current_scope_observed_for_test());
                    execution
                        .run_native_with_checkpoint(|diagnostic| {
                            diagnostic.account_completed_work(40).unwrap();
                            diagnostic.checkpoint(61).unwrap()
                        })
                        .unwrap();
                })
                .unwrap();
            assert_eq!(
                observed.count_for_test(),
                count,
                "foreign consults do not enter original evidence"
            );
            assert_eq!(observed.charged_prefixes_for_test(), prefixes);
            assert!(observed.is_current_scope_observed_for_test());
            root.checkpoint(7).unwrap();
            saved = Some(observed);
        })
        .unwrap();
    assert_eq!(report.charged_work(), 15);
    let observed = saved.unwrap();
    assert!(!observed.is_current_scope_observed_for_test());
    assert_eq!(
        observed
            .charged_prefixes_for_test()
            .into_iter()
            .map(|(_, work)| work)
            .collect::<Vec<_>>(),
        vec![3, 5, 8, 15]
    );
    let count = observed.count_for_test();
    let (_, next) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&request), |fresh_root| {
            assert!(!observed.is_current_scope_observed_for_test());
            fresh_root.checkpoint(37).unwrap();
        })
        .unwrap();
    assert_eq!(next.charged_work(), 37);
    assert_eq!(
        observed.count_for_test(),
        count,
        "the same request cannot retarget a closed root observation"
    );
    assert_eq!(observed.charged_prefixes_for_test().last().unwrap().1, 15);
}
#[test]
fn scoped_consult_observation_restores_nested_and_unwound_thread_guards() {
    let request = serial();
    let outer = Observation::install_on_this_thread_for_test(|_| {});
    let result =
        ExecutionWorkCeiling::new(None).run_with_checkpoint(None, Some(&request), |root| {
            let observed =
                Observation::install_in_scope_on_this_thread_for_test(root, |_| {}).unwrap();
            root.checkpoint(3).unwrap();
            let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _nested = Observation::install_on_this_thread_for_test(|_| {});
                root.checkpoint(7).unwrap();
                panic!("unwind nested observation");
            }));
            assert!(caught.is_err());
            root.checkpoint(5).unwrap();
            assert_eq!(
                observed
                    .charged_prefixes_for_test()
                    .into_iter()
                    .map(|(_, work)| work)
                    .collect::<Vec<_>>(),
                vec![3, 15]
            );
            panic!("unwind scoped observation");
        });
    assert_eq!(result, Err(WorkCeilingDenial::Panicked));
    let (_, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&request), |root| root.checkpoint(2).unwrap())
        .unwrap();
    assert_eq!(report.charged_work(), 2);
    assert_eq!(
        outer
            .charged_prefixes_for_test()
            .into_iter()
            .map(|(_, work)| work)
            .collect::<Vec<_>>(),
        vec![2]
    );
}
