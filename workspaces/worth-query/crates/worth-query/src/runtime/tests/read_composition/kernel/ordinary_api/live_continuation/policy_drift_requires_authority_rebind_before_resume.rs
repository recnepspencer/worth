use super::*;

#[test]
fn policy_drift_requires_authority_rebind_before_resume() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = policy_drift_workspace(request_execution);
    let continuation = checkpointed(
        open_task_resource(
            request_execution,
            &mut workspace,
            "tasks.resume-policy-drift",
        )
        .checkpoint(&mut workspace)
        .expect("fixture digest must fit"),
    );

    let stop = stopped(
        continuation
            .resume(&mut workspace)
            .expect("fixture digest must fit"),
    );
    assert_eq!(
        stop.kind(),
        WorthQueryManagedLiveResumeStopKind::AuthorityRebindRequired
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryManagedLiveResumeNextAction::RebindAuthority
    );
    assert!(matches!(
        stop.close(&mut workspace),
        crate::ordinary::live::WorthQueryManagedLiveCloseOutcome::Closed(_)
    ));
}
