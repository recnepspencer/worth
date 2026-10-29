pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_approval::commit_boundary::advancement_identity::*;

#[test]
fn foreign_phase_cannot_advance_another_workflow() {
    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for placement in [
        Placement::Serial,
        Placement::Leased(std::num::NonZeroUsize::MIN),
    ] {
        let previous = place(placement);
        let (a, ap, ai) = fixture(1_400);
        let (b, bp, bi) = fixture(1_500);
        let scope = request_scope();
        reports();
        a.runtime().with_application_advancement(&scope, |phase| {
            let before = reads();
            let stopped = Adapter::compare_and_commit(&phase, b.runtime(), bp, bi);
            assert!(matches!(stopped, WorkflowProgressOutcome::Application(
                WorthQueryApplicationUncommitted::Denied(ref denial)
            ) if matches!(denial.kind(), WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial: Resource::ForeignAdvancementPhase, partition_identity: None, policy_ancestor: None,
            })));
            assert_eq!(reads(), before);
        }).unwrap();
        let refused = reports();
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].as_ref().unwrap().charged_work(), 0);
        a.runtime()
            .with_application_advancement(&scope, |phase| {
                let before = reads();
                assert!(matches!(
                    Adapter::compare_and_commit(&phase, a.runtime(), ap, ai),
                    WorkflowProgressOutcome::AwaitingApproval(_)
                ));
                assert!(
                    reads() > before,
                    "A's same workflow door revalidates its own source"
                );
            })
            .unwrap();
        place(previous);
    }
}
