pub(crate) use worth_query_worlds::facade::query::suite::advancement_custody::direct_roots::foreign_phase_tests::*;

#[test]
fn borrowed_execution_refuses_a_foreign_phase_before_contact() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&host_request);
    let mut owner = workspace(&request);
    let mut recipient = workspace(&request);
    let operation = admitted(&recipient);
    let before = reads();
    owner
        .with_advancement_mut(|phase, _| {
            let TransitionOutcome::Denied(denial) =
                operation.execute_in_advancement(&phase, &mut recipient)
            else {
                panic!("another workspace's phase cannot execute this operation")
            };
            assert_eq!(
                denial.kind(),
                &domain::WorthQueryBoundExecutionDenialKind::ExecutionRequest(Denial::Resource(
                    Resource::ForeignAdvancementPhase
                ))
            );
            assert_eq!(denial.counters(), Default::default());
            assert_eq!(reads(), before);
        })
        .unwrap();
}

#[test]
fn borrowed_promotion_refuses_a_foreign_phase_before_contact() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&host_request);
    let mut owner = workspace(&request);
    let mut recipient = workspace(&request);
    let current = current(&mut recipient);
    let before = reads();
    owner
        .with_advancement_mut(|phase, _| {
            let domain::WorthQueryProjectionPromotionOutcome::Denied(denial) =
                current.promote_in_advancement(&phase, &mut recipient)
            else {
                panic!("another workspace's phase cannot promote this projection")
            };
            assert_eq!(
                denial.kind(),
                domain::WorthQueryProjectionPromotionDenialKind::ExecutionRequest(
                    Denial::Resource(Resource::ForeignAdvancementPhase)
                )
            );
            assert_eq!(denial.counters(), Default::default());
            assert_eq!(reads(), before);
        })
        .unwrap();
}
