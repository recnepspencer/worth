//! C3.3: the governed workflow is killed after its approval, once between two
//! steps and once mid-step while the guarded operation's managed computation
//! runs. A fresh process reopens, resumes under the original keys with the
//! approval recorded, and reaches the reference terminal state; every step
//! has exactly one effect.
use super::*;
use crate::process::host::durable_retry::child_reexecution::{
    drive_test, expected, location, wait_for_kill,
};
use worth_query_decl::facade::application_program::{
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
};
use worth_query_execution::facade::query_obligations::{
    QueryColdReleaseTerminal, QueryManagedRunPosture,
};
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;

/// Where the writer dies, after the approval step completed.
#[derive(Clone, Copy)]
enum Cut {
    /// Before the guarded operation step starts.
    BetweenSteps,
    /// Inside the guarded operation step: its managed computation is running
    /// and its candidate is prepared, before any owner effect.
    MidStep,
}

#[test]
fn workflow_killed_after_approval_between_steps_resumes_to_the_reference_terminal() {
    journey(
        Cut::BetweenSteps,
        "workflow_killed_after_approval_between_steps_resumes_to_the_reference_terminal",
    );
}
#[test]
fn workflow_killed_mid_step_while_its_managed_computation_runs_resumes_to_the_reference_terminal() {
    journey(
        Cut::MidStep,
        "workflow_killed_mid_step_while_its_managed_computation_runs_resumes_to_the_reference_terminal",
    );
}

fn journey(cut: Cut, behavior: &str) {
    let parent = module_path!().split_once("::").unwrap().1;
    drive_test(
        &format!("{parent}::{behavior}"),
        || writer(cut),
        || reader(cut),
    );
}

fn writer(cut: Cut) {
    let (home, auth) = open(&Location::Directory(location()));
    let (definition, target) = publish(&home);
    let (mut instance, original) = start(&home, definition, 1002);
    assert!(matches!(
        propose(&home, instance.clone(), 1003, &target),
        WorkflowProposalOutcome::Published(_)
    ));
    for (settlement, acceptance) in [(1004, 1005), (1006, 1007)] {
        instance = observation::instance(&home, &original);
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
    let WorkflowProgressOutcome::AwaitingApproval(required) =
        advance(&home, instance.clone(), 1009)
    else {
        panic!("the workflow requires its approval");
    };
    let proposal = observation::proposal(&home, &original);
    assert!(matches!(
        approve(&home, &auth, instance.clone(), &required, &proposal, 1010),
        WorkflowProgressOutcome::Completed(_)
    ));
    match cut {
        Cut::BetweenSteps => wait_for_kill(),
        Cut::MidStep => {
            let required = awaiting_approved_operation(&home, &original);
            let pause = home
                .runtime()
                .runtime()
                .pause_after_application_candidate_preparation_for_test(
                    std::num::NonZeroUsize::MIN,
                );
            std::thread::scope(|threads| {
                threads.spawn(|| {
                    let instance = observation::instance(&home, &original);
                    drop(resume::perform(&home, &instance, &required));
                });
                assert!(
                    pause.wait_until_reached(std::time::Duration::from_secs(60)),
                    "the guarded operation's managed computation is running"
                );
                let active: Vec<_> = home
                    .runtime()
                    .runtime()
                    .managed_run_obligations()
                    .into_iter()
                    .filter(|run| run.posture == QueryManagedRunPosture::Active)
                    .collect();
                let [parked] = active.as_slice() else {
                    panic!("expected exactly one active managed run for the parked guarded operation: {active:?}");
                };
                std::fs::write(
                    expected(),
                    format!(
                        "{}\n{}",
                        parked.identity.acquire_position().get(),
                        parked.logical_run
                    ),
                )
                .unwrap();
                wait_for_kill();
            });
        }
    }
}

fn reader(cut: Cut) {
    let reference = resume::journey(Location::MemoryOnly, false);
    let (home, auth) = open(&Location::Directory(location()));
    if let Cut::MidStep = cut {
        assert!(
            home.runtime()
                .runtime()
                .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
                .unwrap()
                .rows()
                .is_empty(),
            "no World recovery custody remains"
        );
        let recorded = std::fs::read_to_string(expected()).unwrap();
        let mut lines = recorded.lines();
        let expected_position: u64 = lines.next().unwrap().parse().unwrap();
        let expected_logical_run = lines.next().unwrap();
        let runs = home.runtime().runtime().managed_run_obligations();
        let matched = runs
            .iter()
            .find(|run| run.identity.acquire_position().get() == expected_position)
            .unwrap_or_else(|| {
                panic!(
                    "expected parked managed run at acquire_position {expected_position} in {runs:?}"
                )
            });
        assert_eq!(matched.logical_run, expected_logical_run);
        assert_eq!(matched.posture, QueryManagedRunPosture::Active);
        assert_eq!(
            matched.cold_release,
            Some(QueryColdReleaseTerminal::Abandoned),
            "R7: the active managed run at kill terminates as Abandoned independently of the workflow"
        );
    }
    let original = original_start(&home);
    let required = awaiting_approved_operation(&home, &original);
    let instance = observation::instance(&home, &original);
    use resume::GuardedOperation::{Committed, RecoveredCommitted};
    assert_eq!(resume::perform(&home, &instance, &required), Committed);
    assert_eq!(
        resume::perform(&home, &instance, &required),
        RecoveredCommitted,
        "the guarded operation has exactly one effect"
    );
    let instance = observation::instance(&home, &original);
    assert!(
        matches!(advance(&home, instance, 1013), WorkflowProgressOutcome::Completed(done) if done.node_path() == "approved")
    );
    let actual = match observation::read(&home, &original) {
        WorthQueryWorkflowInstanceRead::Historical { observation, .. } => {
            (observation.state(), observation.recorded_paths().to_vec())
        }
        _ => panic!("the original workflow is terminal"),
    };
    assert_eq!(actual, reference, "the reference terminal state");
    assert_eq!(
        document::readback::read_retention(
            home.runtime().runtime(),
            home.runtime().runtime().current_world()
        ),
        8
    );
    drop(auth);
    close(home);
}

/// The recorded approval admits the original guarded operation.
fn awaiting_approved_operation(
    home: &Home,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema>,
) -> RequiredWorkflowOperation {
    let instance = observation::instance(home, original);
    let WorkflowProgressOutcome::AwaitingOperation(required) = advance(home, instance, 1011) else {
        panic!("the approved workflow awaits its guarded operation");
    };
    assert!(
        required
            .workflow_execution_requirements()
            .approval()
            .is_some(),
        "the approval is recorded"
    );
    required
}

/// The original start intent, re-derived from the published definition and
/// the original start key; preparing it publishes nothing.
fn original_start(
    home: &Home,
) -> WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema> {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let basis = request.retain_current_read().unwrap();
    let target = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("reviewed-document").unwrap(),
    );
    let found = request
        .at(&basis)
        .discover_workflow_definition::<GovernedDocumentWorkflow, document::workflow::governed_read::MetadataRead>(
            home.workflow_vocabulary::<GovernedDocumentWorkflow>().unwrap(),
            &target,
            observation::input(),
        )
        .unwrap();
    let WorthQueryWorkflowDefinitionDiscovery::Current(definition) = found.definition() else {
        panic!("the published definition is current");
    };
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&1002u64)
        .prepare_workflow_instance_start(
            home.workflow_vocabulary::<GovernedDocumentWorkflow>()
                .unwrap(),
            definition.clone(),
        )
        .unwrap()
        .discovery_intent()
}
