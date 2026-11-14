//! Fresh Home authority consumes the original retry and never repeats its effect.
use super::*;
use temporal::{
    primary_graph, TemporalConditional, TemporalHostSchema, TemporalInstallationProgram,
};
use worth_query_host::facade::WorthQueryApplicationHome;
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation as temporal;

type Home = WorthQueryApplicationHome<TemporalHostSchema, TemporalInstallationProgram>;

fn open(location: &std::path::Path, request: &worth_execution::ExecutionRequest<'_, '_>) -> Home {
    with_bootstrap_advancement_from_request(*request, |phase| {
        temporal::directory::request(location.to_path_buf()).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("temporal Home open: {refused:?}"))
}

fn observe(
    home: &Home,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> primary_graph::WorthQueryConditionalClockObservationOutcome<temporal::CourtroomClock> {
    let application = home.runtime();
    let installed = application.conditional::<TemporalConditional>().unwrap();
    let installed = installed.as_ref().as_ref().unwrap();
    let selected = application
        .with_host_advancement(|phase| {
            application
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    let outcome = selected
        .conditional_clock(request, &installed.clock)
        .unwrap()
        .observe(request);
    if let primary_graph::WorthQueryConditionalClockObservationOutcome::Failed(cause) = &outcome {
        panic!("clock observation {:?}: {}", cause.kind(), cause.detail());
    }
    outcome
}

#[test]
fn conditional_temporal_custody_consumes_original_key_after_directory_reopen() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&location, &execution);
    let original = {
        let application = home.runtime();
        let installed = application.conditional::<TemporalConditional>().unwrap();
        let installed = installed.as_ref().as_ref().unwrap();
        installed.preconditions_panic.as_ref().unwrap().set(true);
        let first = observe(&home, &execution);
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(first) = first
        else {
            panic!("initial clock did not execute");
        };
        assert_eq!(first.failed_operation_count(), 1);
        assert_eq!(installed.contacts.snapshot().3, 0);
        let original = application.conditional_decision_obligations();
        assert_eq!(original.len(), 1);
        assert!(matches!(
            original[0].posture,
            primary_graph::QueryConditionalPosture::Retryable
        ));
        drop(first);
        original
    };
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("retained retry handoff: {refused:?}"))
        .reclaim();

    let home = open(&location, &execution);
    assert_eq!(home.runtime().conditional_decision_obligations(), original);
    // A fresh source/invoker/clock replaces the old panicking provider. The
    // ordinary clock door reads the durable row before admitting this key.
    // Discard this response without reading it. The cold authored revision
    // below is the independent commit oracle.
    let response = observe(&home, &execution);
    let signal = match &response {
        primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt)
        | primary_graph::WorthQueryConditionalClockObservationOutcome::Duplicate(receipt) => {
            receipt
                .execution_provenance()
                .iter()
                .map(|item| item.signal_decision())
                .collect::<Vec<_>>()
        }
        _ => Vec::new(),
    };
    drop(response);
    assert!(
        home.runtime().conditional_decision_obligations().is_empty(),
        "fresh Signal {signal:?}, contacts {:?}, retained {:?}",
        home.runtime()
            .conditional::<TemporalConditional>()
            .unwrap()
            .as_ref()
            .as_ref()
            .unwrap()
            .contacts
            .snapshot(),
        home.runtime().conditional_decision_obligations()
    );
    assert_eq!(
        home.runtime()
            .conditional::<TemporalConditional>()
            .unwrap()
            .as_ref()
            .as_ref()
            .unwrap()
            .contacts
            .snapshot()
            .3,
        1
    );
    let issued_runs = home.runtime().managed_run_obligations();
    let [issued_run] = issued_runs.as_slice() else {
        panic!("one actual native managed run");
    };
    let issued_run = issued_run.clone();
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("committed clock handoff: {refused:?}"))
        .reclaim();

    let home = open(&location, &execution);
    assert!(home.runtime().conditional_decision_obligations().is_empty());
    {
        use worth_query_execution::facade::domain_computation::{
            WorthQueryManagedRunCleanupDisposition as Cleanup,
            WorthQueryManagedRunTerminalKind as Terminal,
        };
        let retained = home.runtime().managed_run_obligations();
        let [retained] = retained.as_slice() else {
            panic!("same original managed work");
        };
        assert_eq!(retained.identity, issued_run.identity);
        assert_eq!(retained.logical_run, issued_run.logical_run);
        assert_eq!(retained.original_attempt, issued_run.original_attempt);
        assert_eq!(retained.attempt, issued_run.attempt);
        assert!(
            retained.original.is_some(),
            "the actual original World attempt remains joined"
        );
        assert_eq!(
            retained.posture,
            primary_graph::QueryManagedRunPosture::DirectCleanup {
                terminal: Terminal::Completed,
                disposition: Cleanup::CleanupComplete,
                resources:
                    worth_query_execution::facade::query_obligations::QueryManagedRunResources {
                        native_released: true,
                        checkpoint: None,
                        provider_execution: None,
                        restored_execution: None
                    },
            }
        );
        home.runtime()
            .with_host_advancement(|phase| {
                home.runtime()
                    .discharge_managed_run(&phase, issued_run.identity)
            })
            .unwrap()
            .unwrap();
        assert!(home.runtime().managed_run_obligations().is_empty());
    }
    let response = observe(&home, &execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(response) = response
    else {
        panic!("reopened clock did not observe committed intent");
    };
    assert_eq!(response.committed_operation_count(), 0);
    let application = home.runtime();
    let scope = temporal::request_scope();
    let authentication = temporal::admit_identity_adapter(application.installed_schema());
    let principal = temporal::block_on(authentication.authenticate((), &scope)).unwrap();
    let result = application.request(&principal, &scope).query(
        worth_query_testkit_host::facade::host::temporal_conditional_operation::schema::current_read::TemporalIntentReadRequest { identity: "intent-1".to_owned() }
    ).execute().unwrap();
    assert_eq!(result.rows()[0].revision, 2);
    assert_eq!(result.rows()[0].lifecycle, "completed");
    assert_eq!(
        application
            .conditional::<TemporalConditional>()
            .unwrap()
            .as_ref()
            .as_ref()
            .unwrap()
            .contacts
            .snapshot()
            .3,
        0
    );
    drop(result);
    drop(response);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("final temporal handoff: {refused:?}"))
        .reclaim();
}

#[test]
fn conditional_temporal_custody_releases_live_cleanup_and_discharges_cold_association() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&location, &execution);
    let guard = home
        .runtime()
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let response = observe(&home, &execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(response) = response
    else {
        panic!("cleanup clock did not execute");
    };
    assert_eq!(response.indeterminate_operation_count(), 1);
    let original = home.runtime().conditional_decision_obligations();
    let [decision] = original.as_slice() else {
        panic!("one retained conditional cleanup");
    };
    let primary_graph::QueryConditionalPosture::CleanupRequired(identity) = decision.posture else {
        panic!("cleanup retains its shared identity");
    };
    let cleanup = home.runtime().managed_cleanup_obligations();
    assert_eq!(cleanup[0].identity, identity);
    assert!(cleanup[0].release.is_none());
    let runs = home.runtime().managed_run_obligations();
    let [run] = runs.as_slice() else {
        panic!("one actual failed native cleanup");
    };
    let original_run = run.clone();
    drop(response);
    drop(guard);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("live cleanup handoff: {refused:?}"))
        .reclaim();

    let home = open(&location, &execution);
    assert_eq!(home.runtime().conditional_decision_obligations(), original);
    let retained = home.runtime().managed_cleanup_obligations();
    assert_eq!(retained[0].identity, identity);
    assert_eq!(retained[0].run, cleanup[0].run);
    assert!(
        retained[0].release.is_some(),
        "the original live owner recorded release before close"
    );
    let runs = home.runtime().managed_run_obligations();
    let [run] = runs.as_slice() else {
        panic!("one retained native terminal");
    };
    assert_eq!(run.identity, original_run.identity);
    assert_eq!(run.original_attempt, original_run.original_attempt);
    assert_eq!(run.attempt, retained[0].run);
    assert_eq!(run.original, retained[0].original);
    let worth_query_execution::facade::domain_computation::WorthQueryMutationCleanupFailureKind::Native(failure) = retained[0].failure
    else {
        panic!("actual native cleanup failure");
    };
    assert_eq!(
        run.recovery,
        Some(primary_graph::QueryManagedRunRecovery::Cleanup(failure))
    );
    assert!(
        matches!(run.posture, primary_graph::QueryManagedRunPosture::DirectCleanup { resources, .. } if resources.released())
    );
    home.runtime()
        .with_host_advancement(|phase| home.runtime().discharge_managed_run(&phase, run.identity))
        .unwrap()
        .unwrap();
    assert!(home.runtime().managed_run_obligations().is_empty());
    home.runtime()
        .with_host_advancement(|phase| home.runtime().discharge_managed_cleanup(&phase, identity))
        .unwrap()
        .unwrap();
    assert!(home.runtime().managed_cleanup_obligations().is_empty());
    assert!(home.runtime().conditional_decision_obligations().is_empty());
    let response = observe(&home, &execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(response) = response
    else {
        panic!("discharged clock did not observe");
    };
    assert_eq!(response.committed_operation_count(), 0);
    assert_eq!(
        home.runtime()
            .conditional::<TemporalConditional>()
            .unwrap()
            .as_ref()
            .as_ref()
            .unwrap()
            .contacts
            .snapshot()
            .3,
        0
    );
    drop(response);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("cold cleanup discharged: {refused:?}"))
        .reclaim();
}

#[test]
fn conditional_temporal_custody_live_logical_discharge_does_not_republish_cleanup() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&location, &execution);
    let guard = home
        .runtime()
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    drop(observe(&home, &execution));
    let pending = home.runtime().managed_cleanup_obligations();
    let [cleanup] = pending.as_slice() else {
        panic!("one actual failed cleanup");
    };
    let identity = cleanup.identity;
    assert!(cleanup.release.is_none());
    drop(guard);
    // The original owner releases during ordinary observation, before close.
    drop(observe(&home, &execution));
    let released = home.runtime().managed_cleanup_obligations();
    assert_eq!(released[0].identity, identity);
    assert!(released[0].release.is_some());
    home.runtime()
        .with_host_advancement(|phase| home.runtime().discharge_managed_cleanup(&phase, identity))
        .unwrap()
        .unwrap();
    assert!(home.runtime().managed_cleanup_obligations().is_empty());
    assert!(home.runtime().conditional_decision_obligations().is_empty());
    // Re-observation must consume the acknowledged logical discharge rather
    // than resurrect its former CleanupRequired association.
    drop(observe(&home, &execution));
    assert!(home.runtime().conditional_decision_obligations().is_empty());
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("live logical discharge permits close: {refused:?}"))
        .reclaim();
}
