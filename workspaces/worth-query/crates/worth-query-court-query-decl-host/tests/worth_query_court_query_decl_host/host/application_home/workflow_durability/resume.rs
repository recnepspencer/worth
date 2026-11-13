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
    let (mut home, mut auth) = open(&location);
    let (definition, target) = publish(&home);
    let (mut instance, original) = start(&home, definition);
    let original_entity = instance.entity_id();
    assert!(matches!(
        propose(&home, instance.clone(), 1003, &target),
        WorkflowProposalOutcome::Published(_)
    ));
    for (settlement, acceptance) in [(1004, 1005), (1006, 1007)] {
        if reopen {
            close(home);
            (home, auth) = open(&location);
        }
        instance = observation::instance(&home, &original);
        assert_eq!(instance.entity_id(), original_entity);
        if reopen && settlement == 1004 {
            assert!(
                matches!(
                    propose(&home, instance.clone(), 1003, &target),
                    WorkflowProposalOutcome::RecoveredCommitted(_)
                ),
                "the original proposal key recovers rather than publishing twice"
            );
        }
        let evidence = assessment::settle(&home, instance.clone(), settlement);
        assert!(matches!(
            assessment::accept(&home, instance.clone(), &evidence, acceptance),
            WorkflowProgressOutcome::Completed(_)
        ));
    }
    assert!(matches!(
        advance(&home, instance.clone(), 1008),
        WorkflowProgressOutcome::Completed(_)
    ));
    if reopen {
        close(home);
        (home, auth) = open(&location);
    }
    instance = observation::instance(&home, &original);
    if reopen {
        assert!(
            matches!(
                advance(&home, instance.clone(), 1008),
                WorkflowProgressOutcome::RecoveredCommitted(_)
            ),
            "the original evidence-join key survives reopen"
        );
    }
    let required = match advance(&home, instance.clone(), 1009) {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("reconstructed workflow still requires its approval: {other:?}"),
    };
    let proposal = observation::proposal(&home, &original);
    assert!(matches!(
        approve(&home, &auth, instance.clone(), &required, &proposal, 1010),
        WorkflowProgressOutcome::Completed(_)
    ));
    if reopen {
        close(home);
        (home, auth) = open(&location);
    }
    instance = observation::instance(&home, &original);
    let required = match advance(&home, instance.clone(), 1011) {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("approval survived close: {other:?}"),
    };
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_some());
    perform(&home, &instance, &required);
    if reopen {
        close(home);
        (home, auth) = open(&location);
    }
    instance = observation::instance(&home, &original);
    assert!(
        matches!(advance(&home, instance, 1013), WorkflowProgressOutcome::Completed(done) if done.node_path() == "approved")
    );
    if reopen {
        close(home);
        (home, auth) = open(&location);
    }
    let result = match observation::read(&home, &original) {
        WorthQueryWorkflowInstanceRead::Historical { observation, .. } => {
            (observation.state(), observation.recorded_paths().to_vec())
        }
        _ => panic!("guarded operation completes the original workflow"),
    };
    assert_eq!(result.0, WorthQueryWorkflowObservedState::Completed);
    assert!(result.1.iter().any(|path| path == "apply"));
    drop(auth);
    if reopen {
        close(home);
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
