//! Process-local custody outstanding when a real writer is killed: the fresh
//! Home cold-releases it before service, keeps its original, discharges it
//! through the ordinary door and closes gracefully.
use super::fate_reopening::{open, Home};
use super::*;
use crate::host::application_home::{
    managed_cleanup::require_cleanup, managed_runs::start_running_managed_work,
};
use worth_query_host::facade::primary_graph::{QueryColdReleaseTerminal, QueryManagedRunPosture};

#[derive(Clone, Copy, Debug)]
enum Outstanding {
    /// A failed commit's native cleanup obligation, its release unrecorded.
    Cleanup,
    /// Admitted managed work still running, before World admits an attempt.
    ActiveRun,
}

#[test]
fn managed_cleanup_outstanding_at_kill_is_cold_released_with_its_original_then_discharged() {
    journey(
        Outstanding::Cleanup,
        "cold_release::managed_cleanup_outstanding_at_kill_is_cold_released_with_its_original_then_discharged",
    );
}
#[test]
fn active_managed_run_at_kill_is_cold_released_as_abandoned_then_discharged() {
    journey(
        Outstanding::ActiveRun,
        "cold_release::active_managed_run_at_kill_is_cold_released_as_abandoned_then_discharged",
    );
}

fn journey(outstanding: Outstanding, behavior: &str) {
    child_reexecution::drive(behavior, || writer(outstanding), || reader(outstanding));
}

/// What the killed writer retained, as Debug text the reader compares.
fn record(fields: &[String]) {
    std::fs::write(child_reexecution::expected(), fields.join("\n")).unwrap();
}
fn recorded() -> Vec<String> {
    std::fs::read_to_string(child_reexecution::expected())
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn writer(outstanding: Outstanding) {
    let home = open(true, Arc::new(AtomicUsize::new(0)));
    let (_, scope) = scope();
    match outstanding {
        Outstanding::Cleanup => {
            let required = require_cleanup(&home);
            let [row] = &home.runtime().runtime().managed_cleanup_obligations()[..] else {
                panic!("one retained cleanup obligation");
            };
            assert!(row.release.is_none() && row.original.is_some());
            record(&[
                format!("{:?}", row.identity),
                row.run.clone(),
                format!("{:?}", row.failure),
                format!("{:?}", row.original),
            ]);
            child_reexecution::wait_for_kill();
            drop(required);
        }
        Outstanding::ActiveRun => {
            let running = start_running_managed_work(&home, &scope);
            let [row] = &home.runtime().runtime().managed_run_obligations()[..] else {
                panic!("one retained managed run");
            };
            assert_eq!(row.posture, QueryManagedRunPosture::Active);
            record(&[
                format!("{:?}", row.identity),
                row.logical_run.clone(),
                running.identity().to_owned(),
            ]);
            child_reexecution::wait_for_kill();
            drop(running);
        }
    }
}

fn reader(outstanding: Outstanding) {
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(false, contacts.clone());
    let expected = recorded();
    let runtime = home.runtime().runtime();
    let (_, live) = scope();
    match outstanding {
        Outstanding::Cleanup => {
            let [row] = &runtime.managed_cleanup_obligations()[..] else {
                panic!("the killed writer's cleanup obligation is retained");
            };
            let original = format!("{:?}", row.original);
            let observed = [
                format!("{:?}", row.identity),
                row.run.clone(),
                format!("{:?}", row.failure),
                original,
            ];
            assert_eq!(observed[..], expected[..], "its original, run and failure");
            assert!(row.release.is_none(), "no live release is invented");
            assert_eq!(
                row.cold_release,
                Some(QueryColdReleaseTerminal::ReleaseOutstanding)
            );
            let identity = row.identity;
            runtime
                .with_host_advancement(|phase| runtime.discharge_managed_cleanup(&phase, identity))
                .unwrap()
                .unwrap();
            assert!(runtime.managed_cleanup_obligations().is_empty());
            assert!(
                matches!(
                    set(home.runtime(), &live, 0x4847_0701, 29).unwrap(),
                    Outcome::RecoveredCommitted(_)
                ),
                "World's original fate is untouched by the cold release"
            );
            assert_eq!(balance(home.runtime(), &live), 29);
        }
        Outstanding::ActiveRun => {
            let [row] = &runtime.managed_run_obligations()[..] else {
                panic!("the killed writer's managed run is retained");
            };
            let observed = [
                format!("{:?}", row.identity),
                row.logical_run.clone(),
                row.original_attempt.clone(),
            ];
            assert_eq!(observed[..], expected[..], "the original work identity");
            assert_eq!(row.posture, QueryManagedRunPosture::Active);
            assert!(row.original.is_none(), "no World original is fabricated");
            assert_eq!(row.cold_release, Some(QueryColdReleaseTerminal::Abandoned));
            let identity = row.identity;
            runtime
                .with_host_advancement(|phase| runtime.discharge_managed_run(&phase, identity))
                .unwrap()
                .unwrap();
            assert!(runtime.managed_run_obligations().is_empty());
            assert_eq!(balance(home.runtime(), &live), 7);
        }
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 0, "nothing re-executes");
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("cold-released custody closes: {refused:?}"))
        .reclaim();
    let home: Home = open(false, contacts.clone());
    assert!(home
        .runtime()
        .runtime()
        .managed_cleanup_obligations()
        .is_empty());
    assert!(home
        .runtime()
        .runtime()
        .managed_run_obligations()
        .iter()
        .all(|run| run.released()));
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("reopened Home closes: {refused:?}"))
        .reclaim();
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}
