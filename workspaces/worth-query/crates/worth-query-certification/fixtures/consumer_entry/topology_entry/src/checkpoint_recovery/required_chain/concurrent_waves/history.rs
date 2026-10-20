//! Binding metadata is inventoried before edits; expectations come from the edit rules.
use super::*;
use computation::{arm, Failure, Schedule};
use std::{collections::BTreeMap, num::NonZeroUsize};
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome;
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
macro_rules! settle {
    ($demand:expr, $request:expr) => {
        (0..128)
            .find_map(|_| match $demand.advance(&$request).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
            })
            .expect("the seeded demand settles within its declared advance count")
    };
}
#[derive(Debug, PartialEq)]
pub(super) struct Observed {
    committed: Vec<String>,
    pub(super) work: u64,
    pub(super) span: u64,
    lengths: [u64; 2],
    settled: bool,
}
fn run(placement: Placement, failure: Option<(usize, Failure)>) -> Observed {
    run_costs(placement, failure, computation::COSTS)
}
pub(super) fn run_costs(
    placement: Placement,
    failure: Option<(usize, Failure)>,
    costs: [usize; 2],
) -> Observed {
    let _restore = Restore(place(placement));
    arm(None);
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<application::Program>(
        None,
        profile,
        4096,
        128 * 1024 * 1024,
        profile
            .limits()
            .source_currentness_work()
            .map(|work| work as u64),
        diamond::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut left = request
        .demand(PlanarOutputDemand::new("diamond-left"))
        .start_in_program::<application::Program, program::ChainRoot>(&application)
        .unwrap();
    let mut right = request
        .demand(PlanarOutputDemand::new("diamond-right"))
        .start_in_program::<application::Program, program::ChainRoot>(&application)
        .unwrap();
    let mut join = request
        .demand(ChainDemand("diamond-join".to_owned()))
        .start_dependent_in_program::<application::Program, program::ChainConnection>(&application)
        .unwrap();
    // These bindings are the installation's actual inputs, obtained before the
    // history under test. Their application order does not define the oracle.
    let mut names = BTreeMap::new();
    for name in ["diamond-left", "diamond-right", "diamond-join"] {
        let rows = request
            .query(PlanarRead {
                body_key: name.to_owned(),
            })
            .execute()
            .unwrap();
        names.insert(
            rows.observed_sources()[0].root_entity_for_test(),
            name.to_owned(),
        );
    }
    observations();
    settle!(left, request);
    settle!(right, request);
    settle!(join, request);
    let initial = observations();
    let bindings: BTreeMap<_, _> = initial
        .iter()
        .flat_map(|record| &record.applied)
        .filter(|event| event.kind == Kind::Commit)
        .filter_map(|event| match &event.identity {
            Identity::Producer { root, .. } => Some((names[root].clone(), event.identity.clone())),
            Identity::Mutation { .. } => None,
        })
        .collect();
    let mut roots = initial
        .iter()
        .flat_map(|r| &r.applied)
        .filter_map(|event| {
            if event.kind != Kind::Commit {
                return None;
            }
            match &event.identity {
                Identity::Producer {
                    producer,
                    query,
                    parameters,
                    root,
                    source: _,
                } if names[root] != "diamond-join" => Some((
                    (producer.clone(), *query, *parameters, *root),
                    names[root].clone(),
                )),
                Identity::Producer { .. } | Identity::Mutation { .. } => None,
            }
        })
        .collect::<Vec<_>>();
    roots.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(roots.len(), 2, "one binding for each independent root");
    let roots: [String; 2] = roots
        .into_iter()
        .map(|(_, name)| name)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    for (key, y, idempotency) in [
        ("diamond-left", 5, 0x9176_7700_u64),
        ("diamond-right", 54, 0x9176_7701),
    ] {
        let rows = request
            .query(PlanarRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        let outcome = request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: key.to_owned(),
                replacement_y: length(y),
            })
            .expect_source(rows.observed_sources()[0].clone())
            .idempotency(&idempotency)
            .execute_performed::<application::Program, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ));
    }
    let overlap = matches!(placement, Placement::Leased(width) if width.get() > 1)
        && !matches!(failure, Some((_, Failure::Prepare)));
    let schedule = Schedule::declared(roots.to_vec(), costs.to_vec(), failure, overlap);
    arm(Some(schedule.clone()));
    observations();
    let outcome = join.advance(&request);
    let records = observations();
    arm(None);
    let state = schedule.state.lock().unwrap();
    assert!(
        !state.watchdog,
        "later root could not finish first: {:?}",
        state.entered
    );
    assert!(
        state.runs.iter().all(|&runs| runs <= 1),
        "each declared schedule root computes at most once; unrelated kernels return zero outside this schedule: {:?}",
        state.runs
    );
    if overlap {
        assert_eq!(
            state.entered,
            [true, true],
            "the schedule reached both kernels"
        );
        assert!(state.finished[1]);
    }
    let [observed] = records.as_slice() else {
        panic!("one advancement covers both waves: {records:?}")
    };
    let mut committed = Vec::new();
    for pair in observed.applied.chunks(2) {
        assert_eq!(pair.len(), 2);
        assert_eq!(pair[0].kind, Kind::Commit);
        assert_eq!(pair[1].kind, Kind::Publication);
        assert_eq!(pair[0].identity, pair[1].identity);
        let Identity::Producer { root, .. } = &pair[0].identity else {
            panic!("producer apply has producer identity")
        };
        let name = &names[root];
        let Identity::Producer {
            producer,
            query,
            parameters,
            root: old_root,
            source: old_source,
        } = &bindings[name]
        else {
            unreachable!("the inventory retains producer bindings")
        };
        let Identity::Producer {
            producer: actual_producer,
            query: actual_query,
            parameters: actual_parameters,
            source,
            ..
        } = &pair[0].identity
        else {
            unreachable!("the applied member is a producer")
        };
        assert_eq!(
            (actual_producer, actual_query, actual_parameters, root),
            (producer, query, parameters, old_root),
            "the event carries the issued semantic binding"
        );
        if name != "diamond-join" {
            assert_ne!(
                source, old_source,
                "the changed source has a new fact commitment"
            );
        }
        committed.push(name.clone());
    }
    let mut expected = roots[..failure.map_or(2, |(rank, _)| rank)].to_vec();
    if failure.is_none() {
        expected.push("diamond-join".to_owned());
    }
    assert_eq!(
        committed, expected,
        "canonical ready roots precede their consumer; no suffix effect survives"
    );
    if let Some((rank, Failure::Complete)) = failure {
        assert_eq!(state.completed, (0..=rank).collect::<Vec<_>>());
    }
    let settled = matches!(
        outcome,
        Ok(WorthQueryApplicationOutputDemandProgress::Settled(_))
    );
    assert_eq!(
        settled,
        failure.is_none(),
        "the seeded phase failure prevents settlement: {:?}",
        outcome.as_ref().err()
    );
    drop(state);
    let mut lengths = [0; 2];
    for (index, (key, old, y)) in [("diamond-left", 2, 5), ("diamond-right", 51, 54)]
        .into_iter()
        .enumerate()
    {
        let rows = request
            .query(PlanarOutputRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        lengths[index] = PositiveLength::get(&rows.rows()[0].value);
        let expected = if committed.iter().any(|name| name == key) {
            y + 1
        } else {
            old
        };
        assert_eq!(
            lengths[index], expected,
            "published facts are derived from the declared edits"
        );
    }
    Observed {
        committed,
        work: observed.report.charged_work(),
        span: observed.report.charged_span(),
        lengths,
        settled,
    }
}
// Each cell has identical runtime-binding inputs. Fresh processes keep unrelated
// installation counters from changing the bytes of owner-stage fact keys.
fn cell(workers: usize, failure: Option<(usize, Failure)>) -> String {
    let executable = std::env::current_exe().unwrap();
    let phase = failure.map_or("none", |(_, phase)| match phase {
        Failure::Prepare => "prepare",
        Failure::Compute => "compute",
        Failure::Complete => "complete",
    });
    let output = std::process::Command::new(executable)
        .args([
            "--exact",
            "checkpoint_recovery::required_chain::concurrent_waves::history::a_real_diamond_agrees_across_widths_and_failure_phases",
            "--nocapture",
        ])
        .env("WORTH_WAVE_CELL_WORKERS", workers.to_string())
        .env("WORTH_WAVE_CELL_PHASE", phase)
        .env("WORTH_WAVE_CELL_RANK", failure.map_or(0, |(rank, _)| rank).to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "width {workers}, failure {failure:?}: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("WORTH_WAVE_OBSERVATION:"))
        .expect("the cell returns its system observation")
        .to_owned()
}
#[test]
fn a_real_diamond_agrees_across_widths_and_failure_phases() {
    let _guard = checkpoint_recovery_test_guard();
    if let Ok(workers) = std::env::var("WORTH_WAVE_CELL_WORKERS") {
        let workers: usize = workers.parse().unwrap();
        let rank = std::env::var("WORTH_WAVE_CELL_RANK")
            .unwrap()
            .parse()
            .unwrap();
        let failure = match std::env::var("WORTH_WAVE_CELL_PHASE").unwrap().as_str() {
            "none" => None,
            "prepare" => Some((rank, Failure::Prepare)),
            "compute" => Some((rank, Failure::Compute)),
            "complete" => Some((rank, Failure::Complete)),
            phase => panic!("unknown declared failure phase {phase}"),
        };
        let posture = NonZeroUsize::new(workers).map_or(Placement::Serial, Placement::Leased);
        let observed = run(posture, failure);
        if failure.is_none() {
            assert!(
                observed.span < observed.work,
                "two positive independent members reduce structural span: {observed:?}"
            );
        }
        // Captured subprocess protocol, not a recorded expected value.
        println!("WORTH_WAVE_OBSERVATION:{observed:?}");
        return;
    }
    for failure in [
        None,
        Some((0, Failure::Prepare)),
        Some((1, Failure::Prepare)),
        Some((0, Failure::Compute)),
        Some((1, Failure::Compute)),
        Some((0, Failure::Complete)),
        Some((1, Failure::Complete)),
    ] {
        let serial = cell(0, failure);
        for workers in [1, 2, 4] {
            assert_eq!(
                cell(workers, failure),
                serial,
                "width {workers}, phase failure {failure:?}"
            );
        }
    }
}
