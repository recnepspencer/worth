#[allow(unused_imports)]
pub(crate) use crate::fast::sealed_request::*;

#[test]
fn sealed_leased_request_projects_the_exact_callers_lease() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: policy(1 << 20),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let (same, _) = ExecutionRequest::leased(&lease)
        .run(ExecutionWorkCeiling::new(Some(1_000)), |carried| {
            std::ptr::eq(
                carried.expect("leased carriage cannot erase its proof"),
                &lease,
            )
        })
        .unwrap();
    assert!(same);
}

#[test]
fn leased_policy_work_ceiling_still_bounds_the_same_scan() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: one_unit_policy(),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    assert_eq!(
        ExecutionRequest::leased(&lease)
            .in_scope(charged_scan)
            .unwrap(),
        (false, 0)
    );
}
