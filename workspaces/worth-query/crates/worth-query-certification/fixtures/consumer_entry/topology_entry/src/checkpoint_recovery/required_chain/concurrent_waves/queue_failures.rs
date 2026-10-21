//! A queued failure belongs to its own member, not to the independent wave.
use super::*;
use computation::{arm, Failure, Schedule};
use std::{collections::BTreeMap, num::NonZeroUsize};
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test as observations,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryAdvancementEventKindForTest as Kind,
    WorthQueryAdvancementIdentityForTest as Identity,
    WorthQueryExecutionPlacementForTest as Placement,
};
struct Restore(Placement);
impl Drop for Restore {
    fn drop(&mut self) {
        arm(None);
        place(self.0);
    }
}
#[derive(Debug, PartialEq)]
struct History {
    commits: Vec<String>,
    retry: Vec<String>,
    runs: [usize; 2],
    work: u64,
}
fn run(workers: usize, failure: Failure, caught: bool) -> History {
    let _restore = Restore(place(
        NonZeroUsize::new(workers).map_or(Placement::Serial, Placement::Leased),
    ));
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<application::Program>(
        None,
        profile,
        4096,
        diamond::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    macro_rules! start {
        ($key:expr) => {
            request
                .demand(PlanarOutputDemand::new($key))
                .start_in_program::<application::Program, program::ChainRoot>(&application)
                .unwrap()
        };
    }
    let mut left = start!("diamond-left");
    let mut right = start!("diamond-right");
    let mut caller = start!("diamond-join");
    for demand in [&mut left, &mut right, &mut caller] {
        assert!(matches!(
            demand.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
    }
    let mut names = BTreeMap::new();
    for key in ["diamond-left", "diamond-right"] {
        let selected = request
            .query(PlanarRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        names.insert(
            selected.observed_sources()[0].root_entity_for_test(),
            key.to_owned(),
        );
    }
    let initial = observations();
    let mut keys = initial
        .iter()
        .flat_map(|record| &record.applied)
        .filter_map(|event| match &event.identity {
            Identity::Producer {
                producer,
                query,
                parameters,
                root,
                source: _,
            } if event.kind == Kind::Commit && names.contains_key(root) => Some((
                (producer.clone(), *query, *parameters, *root),
                names[root].clone(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    keys.sort();
    assert_eq!(keys.len(), 2);
    let roots: [String; 2] = keys
        .into_iter()
        .map(|(_, name)| name)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    for (key, y, id) in [
        ("diamond-left", 5, 0x9176_7710_u64),
        ("diamond-right", 54, 0x9176_7711),
    ] {
        let selected = request
            .query(PlanarRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: key.into(),
                replacement_y: length(y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&id)
            .execute_performed::<application::Program, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    let mut schedule = Schedule::new(
        roots.clone(),
        Some((0, failure)),
        workers > 1 && failure != Failure::Prepare,
    );
    std::sync::Arc::get_mut(&mut schedule)
        .unwrap()
        .catches_denial = caught;
    arm(Some(schedule.clone()));
    observations();
    let outcome = caller.advance(&request);
    assert!(
        matches!(
            outcome,
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(_))
        ),
        "a queued sibling's failure cannot replace the caller: {:?}",
        outcome.as_ref().err()
    );
    let commits = |records: Vec<
        worth_query_host::facade::primary_graph::WorthQueryAdvancementObservationForTest,
    >| {
        records
            .into_iter()
            .flat_map(|record| record.applied)
            .filter_map(|event| match event.identity {
                Identity::Producer { root, .. } if event.kind == Kind::Commit => {
                    Some(names[&root].clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let records = observations();
    assert_eq!(records.len(), 1);
    let work = records[0].report.charged_work();
    let committed = commits(records);
    let expected = if caught {
        roots.to_vec()
    } else {
        vec![roots[1].clone()]
    };
    assert_eq!(
        committed, expected,
        "the cursor continues after a queued member fails"
    );
    let state = schedule.state.lock().unwrap();
    assert!(!state.watchdog, "the declared completion schedule happened");
    let expected_runs = if failure == Failure::Prepare {
        [0, 1]
    } else {
        [1, 1]
    };
    assert_eq!(
        state.runs, expected_runs,
        "each dispatched member computes exactly once"
    );
    drop(state);
    arm(None);
    observations();
    let retried = caller.advance(&request);
    assert!(matches!(
        retried,
        Ok(WorthQueryApplicationOutputDemandProgress::Settled(_))
    ));
    let retry = commits(observations());
    assert_eq!(
        retry,
        if caught {
            vec![]
        } else {
            vec![roots[0].clone()]
        },
        "only the held sibling retries"
    );
    History {
        commits: committed,
        retry,
        runs: expected_runs,
        work,
    }
}
#[test]
fn queued_phase_failures_and_caught_denials_agree_across_widths() {
    let _guard = checkpoint_recovery_test_guard();
    if let Ok(workers) = std::env::var("WORTH_QUEUE_FAILURE_WIDTH") {
        let phase = std::env::var("WORTH_QUEUE_FAILURE_PHASE").unwrap();
        let (failure, caught) = match phase.as_str() {
            "prepare" => (Failure::Prepare, false),
            "complete" => (Failure::Complete, false),
            "caught" => (Failure::Compute, true),
            _ => panic!("unknown phase"),
        };
        println!(
            "WORTH_QUEUE_HISTORY:{:?}",
            run(workers.parse().unwrap(), failure, caught)
        );
        return;
    }
    for phase in ["prepare", "complete", "caught"] {
        let mut serial = None;
        for workers in [0, 1, 2, 4] {
            let output = std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact", "checkpoint_recovery::required_chain::concurrent_waves::queue_failures::queued_phase_failures_and_caught_denials_agree_across_widths", "--nocapture"]).env("WORTH_QUEUE_FAILURE_WIDTH", workers.to_string()).env("WORTH_QUEUE_FAILURE_PHASE", phase).output().unwrap();
            assert!(
                output.status.success(),
                "width {workers}, {phase}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8(output.stdout).unwrap();
            let actual = stdout
                .lines()
                .find_map(|line| line.strip_prefix("WORTH_QUEUE_HISTORY:"))
                .unwrap()
                .to_owned();
            assert_eq!(actual, *serial.get_or_insert_with(|| actual.clone()));
        }
    }
}
