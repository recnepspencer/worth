//! A close between workflow steps preserves keys, evidence and mandatory approval.
use super::*;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
#[test]
fn directory_workflow_resumes_original_keys_and_required_approval_to_reference_terminal() {
    let reference = journey(Location::MemoryOnly, false);
    let actual = journey(Location::Directory(super::super::fixture::location()), true);
    assert_eq!(actual, reference);
}
fn journey(location: Location, reopen: bool) -> (WorthQueryWorkflowObservedState, Vec<String>) {
    let (home, mut auth) = open_boxed(&location);
    let mut home = Some(home);
    let original = start_and_run_assessments(&mut home, &mut auth, &location, reopen);
    advance_and_approve(&mut home, &mut auth, &location, &original, reopen);
    complete_and_observe(home, auth, &location, &original, reopen)
}

#[inline(never)]
fn open_boxed(location: &Location) -> (Box<Home>, CertificationAuthenticationOwner) {
    let (home, auth) = open(location);
    (Box::new(home), auth)
}

#[inline(never)]
fn reopen_after_close(
    home: &mut Option<Box<Home>>,
    auth: &mut CertificationAuthenticationOwner,
    location: &Location,
    reopen: bool,
) {
    if reopen {
        close(*home.take().expect("open home before reopen"));
        let (next_home, next_auth) = open_boxed(location);
        *home = Some(next_home);
        *auth = next_auth;
    }
}

#[inline(never)]
fn start_and_run_assessments(
    home: &mut Option<Box<Home>>,
    auth: &mut CertificationAuthenticationOwner,
    location: &Location,
    reopen: bool,
) -> WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema> {
    let current = home.as_ref().expect("home present");
    let (definition, target) = publish(current);
    let (mut instance, original) = start(current, definition, 1002);
    let original_entity = instance.entity_id();
    assert!(matches!(
        propose(current, instance.clone(), 1003, &target),
        WorkflowProposalOutcome::Published(_)
    ));
    for (settlement, acceptance) in [(1004, 1005), (1006, 1007)] {
        reopen_after_close(home, auth, location, reopen);
        let current = home.as_ref().expect("home present");
        instance = observation::instance(current, &original);
        assert_eq!(instance.entity_id(), original_entity);
        if reopen && settlement == 1004 {
            assert!(
                matches!(
                    propose(current, instance.clone(), 1003, &target),
                    WorkflowProposalOutcome::RecoveredCommitted(_)
                ),
                "the original proposal key recovers rather than publishing twice"
            );
        }
        let evidence = assessment::settle(current, instance.clone(), settlement);
        let outcome = assessment::accept(current, instance.clone(), &evidence, acceptance);
        assert!(
            matches!(outcome, WorkflowProgressOutcome::Completed(_)),
            "assessment {settlement} must complete: {outcome:?}"
        );
    }
    let current = home.as_ref().expect("home present");
    assert!(matches!(
        advance(current, instance, 1008),
        WorkflowProgressOutcome::Completed(_)
    ));
    original
}

#[inline(never)]
fn advance_and_approve(
    home: &mut Option<Box<Home>>,
    auth: &mut CertificationAuthenticationOwner,
    location: &Location,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema>,
    reopen: bool,
) {
    reopen_after_close(home, auth, location, reopen);
    let current = home.as_ref().expect("home present");
    let instance = observation::instance(current, original);
    if reopen {
        assert!(
            matches!(
                advance(current, instance.clone(), 1008),
                WorkflowProgressOutcome::RecoveredCommitted(_)
            ),
            "the original evidence-join key survives reopen"
        );
    }
    let required = match advance(current, instance.clone(), 1009) {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("reconstructed workflow still requires its approval: {other:?}"),
    };
    let proposal = observation::proposal(current, original);
    assert!(matches!(
        approve(current, auth, instance, &required, &proposal, 1010),
        WorkflowProgressOutcome::Completed(_)
    ));
}

#[inline(never)]
fn complete_and_observe(
    mut home: Option<Box<Home>>,
    mut auth: CertificationAuthenticationOwner,
    location: &Location,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema>,
    reopen: bool,
) -> (WorthQueryWorkflowObservedState, Vec<String>) {
    reopen_after_close(&mut home, &mut auth, location, reopen);
    {
        let current = home.as_ref().expect("home present");
        let instance = observation::instance(current, original);
        let required = match advance(current, instance.clone(), 1011) {
            WorkflowProgressOutcome::AwaitingOperation(required) => required,
            other => panic!("approval survived close: {other:?}"),
        };
        assert!(required
            .workflow_execution_requirements()
            .approval()
            .is_some());
        perform(current, &instance, &required);
    }
    reopen_after_close(&mut home, &mut auth, location, reopen);
    {
        let current = home.as_ref().expect("home present");
        let instance = observation::instance(current, original);
        assert!(
            matches!(advance(current, instance, 1013), WorkflowProgressOutcome::Completed(done) if done.node_path() == "approved")
        );
    }
    reopen_after_close(&mut home, &mut auth, location, reopen);
    let current = home.as_ref().expect("home present");
    let result = match observation::read(current, original) {
        WorthQueryWorkflowInstanceRead::Historical { observation, .. } => {
            (observation.state(), observation.recorded_paths().to_vec())
        }
        _ => panic!("guarded operation completes the original workflow"),
    };
    assert_eq!(result.0, WorthQueryWorkflowObservedState::Completed);
    assert!(result.1.iter().any(|path| path == "apply"));
    drop(auth);
    if reopen {
        close(*home.take().expect("open home before final close"));
    }
    result
}
fn perform(
    home: &Home,
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowOperation,
) {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let result = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(document::retention_entry::ReviewedSetRetentionIntent {
            input: document::schema::SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&1012u64)
        .for_workflow_operation(home.vocabulary(), required)
        .unwrap()
        .advanced()
        .execute_in_program(
            home.runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}
