use super::*;
use crate::domain_computation::primary_graph::WorthQueryMutationHandlerWork;

#[test]
fn handler_projection_work_survives_candidate_denials_and_domain_refusal() {
    let world = installed_authorization_world(true);
    for plan in [
        OptionalOutputPlan::RequiredAndOptionalTwice,
        OptionalOutputPlan::OptionalOnly,
    ] {
        let (report, _) = execute_report_with_key(&world, plan, "candidate-denial");
        let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
            panic!("the real installed handler must execute before candidate refusal");
        };
        assert!(work.handler_contacted());
        assert_eq!(work.projection_work().equality_lookups(), 1);
        assert_eq!(work.projection_work().index_candidates_examined(), 1);
        assert_eq!(work.projection_work().provider_work_units(), 2);
        assert!(
            report.into_outcome().is_err(),
            "the real candidate contract refuses {plan:?}"
        );
    }

    let (report, key) = execute_report_with_key(
        &world,
        OptionalOutputPlan::RequiredAfterIndexedAbsence,
        "publish-membership",
    );
    let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
        panic!("the complete indexed decision executes");
    };
    assert_eq!(work.projection_work().equality_lookups(), 2);
    assert_eq!(work.projection_work().index_candidates_examined(), 1);
    assert_eq!(work.projection_work().provider_work_units(), 3);
    let Ok(HandlerResult::Completed(completed)) = report.into_outcome() else {
        panic!("the installed handler prepares the membership correction");
    };
    let (program, _) = completed.into_parts();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(program, key),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let (report, _) = execute_report_for_input(
        &world,
        OptionalOutputInput {
            status: "unrelated".to_owned(),
            plan: OptionalOutputPlan::RequiredAfterIndexedAbsence,
        },
        "membership-exists",
    );
    let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
        panic!("the installed decision executes before refusing existing membership");
    };
    assert!(work.handler_contacted());
    assert_eq!(work.projection_work().equality_lookups(), 2);
    assert_eq!(work.projection_work().index_candidates_examined(), 2);
    assert_eq!(work.projection_work().provider_work_units(), 4);
    assert!(matches!(
        report.into_outcome(),
        Ok(HandlerResult::DomainDenied(_))
    ));
}
