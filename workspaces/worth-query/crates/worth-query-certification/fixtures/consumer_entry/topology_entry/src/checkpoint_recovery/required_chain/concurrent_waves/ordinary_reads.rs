//! Ordinary reads have no declared prerequisite; the application still rejects stale plans.
use super::*;
use computation::{arm, Schedule};
use std::{collections::BTreeMap, num::NonZeroUsize};
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test as observations,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryAdvancementEventKind as Kind, WorthQueryAdvancementIdentity as Identity,
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
    values: [u64; 2],
}
fn run(workers: usize) -> History {
    let _restore = Restore(place(
        NonZeroUsize::new(workers).map_or(Placement::Serial, Placement::Leased),
    ));
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<ordinary_read_application::Program>(
        None,
        profile,
        4096,
        128 * 1024 * 1024,
        profile.limits().source_currentness_work() as u64,
        diamond::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    macro_rules! start {
        ($key:expr) => {
            request
                .demand(PlanarOutputDemand::new($key))
                .start_in_program::<ordinary_read_application::Program, program::ChainRoot>(
                    &application,
                )
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
    for (rank, key) in roots.iter().enumerate() {
        let selected = request
            .query(PlanarRead {
                body_key: key.clone(),
            })
            .execute()
            .unwrap();
        request
            .mutate(PlanarSourceAdjustment {
                scope_key: key.clone(),
                replacement_y: length(5 + rank as u64),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&(0x9176_7720_u64 + rank as u64))
            .execute_performed::<ordinary_read_application::Program, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    let schedule = Schedule::new(roots.clone(), None, false);
    arm(Some(schedule.clone()));
    observations();
    for _ in 0..8 {
        caller.advance(&request).unwrap();
    }
    arm(None);
    assert_eq!(
        schedule.state.lock().unwrap().runs,
        [1, 2],
        "A publishes once; that publication pays exactly one reinspection of stale M"
    );
    let records = observations();
    let commits: Vec<_> = records
        .into_iter()
        .flat_map(|record| record.applied)
        .filter_map(|event| match event.identity {
            Identity::Producer { root, .. } if event.kind == Kind::Commit => {
                Some(names[&root].clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        commits, roots,
        "A commits, then M commits from a rechecked ordinary read"
    );
    let values = roots.map(|key| {
        let selected = request
            .query(PlanarOutputRead { body_key: key })
            .execute()
            .unwrap();
        PositiveLength::get(&selected.rows()[0].value)
    });
    assert_eq!(
        values,
        [5 + 1, (5 + 1) + 1],
        "M reflects A's write, not the value it read before the wave"
    );
    History { commits, values }
}
#[test]
fn an_ordinary_read_is_rechecked_after_an_earlier_member_writes_it() {
    let _guard = checkpoint_recovery_test_guard();
    if let Ok(workers) = std::env::var("WORTH_ORDINARY_READ_WIDTH") {
        println!(
            "WORTH_ORDINARY_READ_HISTORY:{:?}",
            run(workers.parse().unwrap())
        );
        return;
    }
    let mut serial = None;
    for workers in [0, 1, 2, 4] {
        let output = std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact", "checkpoint_recovery::required_chain::concurrent_waves::ordinary_reads::an_ordinary_read_is_rechecked_after_an_earlier_member_writes_it", "--nocapture"]).env("WORTH_ORDINARY_READ_WIDTH", workers.to_string()).output().unwrap();
        assert!(
            output.status.success(),
            "width {workers}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let actual = stdout
            .lines()
            .find_map(|line| line.strip_prefix("WORTH_ORDINARY_READ_HISTORY:"))
            .unwrap()
            .to_owned();
        assert_eq!(actual, *serial.get_or_insert_with(|| actual.clone()));
    }
}
