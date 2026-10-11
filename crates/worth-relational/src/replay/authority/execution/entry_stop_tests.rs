#[test]
fn pre_cancelled_replay_has_no_reader_contact_or_diagnostic() {
    use crate::durability::data::RelationalRecoveryStop;
    use crate::execution::RelationalExecutionDenialCause as Cause;
    use worth_execution::{
        CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest,
    };
    let runtime = crate::tests::support::runtime_with_test_schema();
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(1024), cancellation.token(), None);
    let before = format!("{runtime:#?}");
    let contacts = crate::replay::planning::replay_contacts::count();
    let result = runtime.replay_authority().replay_commit(
        crate::replay::data::RelationalReplayRequest {
            commit_id: crate::history::data::CommitId(1),
            branch_id: crate::history::data::BranchId("main".into()),
            execution_mode: crate::replay::data::ReplayExecutionMode::SerialDeterministic,
            verification_mode:
                crate::replay::data::ReplayVerificationMode::NormalRecoveryVerification,
        },
        &ExecutionRequest::serial(&host),
    );
    assert_eq!(
        crate::replay::planning::replay_contacts::count() - contacts,
        0
    );
    assert_eq!(
        result,
        Err(RelationalRecoveryStop::Request(Cause::Cancelled))
    );
    assert_eq!(format!("{runtime:#?}"), before);
}
