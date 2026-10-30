#[allow(unused_imports)]
pub(crate) use {
    crate::work_ceiling::ordinary_context::authority,
    crate::work_ceiling::ordinary_context::exhausted_at,
    crate::work_ceiling::ordinary_context::reduce,
    crate::work_ceiling::ordinary_context::PARTITION_WORK, std::num::NonZeroUsize,
    worth_execution::CancellationToken, worth_execution::LeaseRequest,
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
};

#[test]
fn the_narrower_of_the_lease_and_the_declared_ceiling_binds() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = authority();
    let lease = |work| {
        authority
            .request_lease(LeaseRequest {
                policy: worth_foundational::ExecutionRequestPolicy::new(
                    worth_foundational::ExecutionPosture::Automatic,
                    worth_foundational::DeterminismContract::CanonicalBitwise,
                    worth_foundational::ExecutionBudget::new(
                        std::num::NonZeroUsize::new(2).unwrap(),
                        None,
                        Some(work),
                    ),
                ),
                deadline: None,
                cancellation: worth_execution::CancellationToken::new(),
            })
            .unwrap()
    };
    let (complete, _) = reduce(Some(&lease(10_000)), 10_000, 4);
    let (sum, report) = complete.expect("both bounds admit the work");
    assert_eq!(sum, 10);
    let (declared, _) = reduce(Some(&lease(10_000)), 2 * PARTITION_WORK + 2, 4);
    assert_eq!(exhausted_at(&declared), Some(3));
    let (leased, _) = reduce(Some(&lease(2 * PARTITION_WORK + 2)), 10_000, 4);
    assert_eq!(exhausted_at(&leased), Some(3));
    let (serial, _) = reduce(None, u64::MAX, 4);
    assert_eq!(
        serial.expect("serial completes").1.charged_work(),
        report.charged_work()
    );
}
#[path = "retained_run/mod.rs"]
pub(crate) mod retained_run;
