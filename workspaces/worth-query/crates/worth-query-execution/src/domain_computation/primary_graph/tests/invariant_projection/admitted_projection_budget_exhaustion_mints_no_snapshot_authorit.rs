//! admitted projection budget exhaustion mints no snapshot authority.
use super::*;

#[test]
fn admitted_projection_budget_exhaustion_mints_no_snapshot_authority() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let scope = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    assert_eq!(operation.contracts().projection_work_budget(), Some(32));
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &scope,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let baseline = world.invariant.active_snapshot_count(request_execution);

    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| {
                        let account = reader
                            .resolve_entity(AccountStatus::reference(), "open".to_string())
                            .unwrap();
                        for _ in 0..31 {
                            let _ = reader.field(&account, AccountStatus::reference());
                        }
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("provider work beyond the installed limit must deny")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded
    );
    let work = denial
        .projection_work()
        .expect("the exhausted reader retains actual work");
    assert_eq!(work.equality_lookups(), 1);
    assert_eq!(work.index_candidates_examined(), 1);
    assert_eq!(work.field_reads(), 30);
    assert_eq!(work.provider_work_units(), 32);
    assert_eq!(
        world.invariant.active_snapshot_count(request_execution),
        baseline
    );
}
