//! lifecycle replay resolvers have no replay for an ordinary admission.
use super::*;

#[test]
fn lifecycle_replay_resolvers_have_no_replay_for_an_ordinary_admission() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "lifecycle")
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            idempotency(12, 12),
            ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    // The same key already committed, but an ordinary admission holds no
    // lifecycle binding, so no lifecycle resolver may replay or panic on it.
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "lifecycle", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiTouchOperation::reference())
        .unwrap();
    let mut admission = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let application = &world.application;
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                application.resolve_admitted_elevation_close_replay(
                    &phase,
                    &mut admission,
                    idempotency(12, 12)
                ),
                Ok(None)
            ))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                application.resolve_admitted_elevation_approval_replay(
                    &phase,
                    &mut admission,
                    idempotency(12, 12)
                ),
                Ok(None)
            ))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                application.resolve_admitted_mandatory_review_replay(
                    &phase,
                    &mut admission,
                    idempotency(12, 12)
                ),
                Ok(None)
            ))
        })
        .expect("the fixture read owns its installed host call");

    // Nor can an ordinary admission materialize a lifecycle program: each
    // lane refuses it with the typed transition denial before any effect.
    let ordinary_reads = || {
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                let operation = application
                    .installed_schema()
                    .installed_operation(super::super::TouchAccountOperation::reference())
                    .unwrap();
                let admission = world
                    .selected_product(&(phase).test_host_request())
                    .authorize_operation(
                        &phase,
                        &principal,
                        &account,
                        &operation,
                        Default::default(),
                        &request,
                    )
                    .unwrap();
                let (_, projection, _) = world
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, projected| {
                            reader
                                .require_decision_field(
                                    projected,
                                    super::super::AccountStatus::reference(),
                                )
                                .unwrap();
                        },
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                application
                    .begin_projected_application_read_attempt(
                        &phase,
                        admission,
                        projection,
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .complete_projected_dependencies(
                        &phase,
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
            })
            .expect("the fixture invocation owns its installed host call")
    };
    let denials = [
        ordinary_reads()
            .materialize_elevation_request_program()
            .err(),
        ordinary_reads()
            .materialize_elevation_approval_program()
            .err(),
        ordinary_reads().materialize_elevation_close_program().err(),
        ordinary_reads()
            .materialize_mandatory_review_program()
            .err(),
    ];
    for denial in denials {
        assert_eq!(
            denial.map(|denial| denial.kind()),
            Some(WorthQueryApplicationAttemptDenialKind::ElevationTransitionRequired)
        );
    }
}
