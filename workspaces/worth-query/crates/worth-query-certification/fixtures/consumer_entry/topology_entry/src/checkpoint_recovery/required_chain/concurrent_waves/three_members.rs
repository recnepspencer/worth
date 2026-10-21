//! Canonical application of three members differs from submission and witnessed completion.
use super::*;
use computation::{arm, Schedule};
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
fn run(workers: usize) -> Vec<String> {
    let _restore = Restore(place(
        NonZeroUsize::new(workers).map_or(Placement::Serial, Placement::Leased),
    ));
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application =
        support::install_program_with_seed::<application::Program>(None, profile, 4096, |graph| {
            diamond::seed(graph);
            support::seed_cycle(graph);
        });
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
    let mut join = start!("diamond-join");
    let mut caller = start!("anchor-a");
    for demand in [&mut right, &mut join, &mut left, &mut caller] {
        assert!(matches!(
            demand.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
    }
    let mut names = BTreeMap::new();
    for key in ["diamond-left", "diamond-right", "diamond-join"] {
        let selected = request
            .query(PlanarRead {
                body_key: key.to_owned(),
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
    assert_eq!(keys.len(), 3);
    let roots: [String; 3] = keys
        .into_iter()
        .map(|(_, name)| name)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let submission = [2, 0, 1];
    for (ordinal, rank) in submission.into_iter().enumerate() {
        let key = &roots[rank];
        let y = match key.as_str() {
            "diamond-left" => 5,
            "diamond-right" => 54,
            "diamond-join" => 104,
            _ => unreachable!(),
        };
        let id = 0x9176_7730_u64 + ordinal as u64;
        let selected = request
            .query(PlanarRead {
                body_key: key.clone(),
            })
            .execute()
            .unwrap();
        request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: key.clone(),
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
    let schedule = Schedule::declared(roots.to_vec(), vec![11, 3, 5], None, true);
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
    for pair in records[0].applied.chunks(2) {
        assert_eq!(pair.len(), 2);
        assert_eq!(pair[0].kind, Kind::Commit);
        assert_eq!(pair[1].kind, Kind::Publication);
        assert_eq!(pair[0].identity, pair[1].identity);
    }
    let committed = commits(records);
    assert_eq!(
        committed, roots,
        "producer then source epoch orders canonical apply"
    );
    let state = schedule.state.lock().unwrap();
    assert!(
        !state.watchdog,
        "both later roots must finish before the first root"
    );
    assert_eq!(state.entered, [true, true, true]);
    assert_eq!(state.runs, [1, 1, 1]);
    assert_eq!(state.computed.last(), Some(&0));
    assert_ne!(
        state.computed,
        [0, 1, 2],
        "completion order differs from canonical order"
    );
    assert_ne!(
        state.computed, submission,
        "completion order differs from submission order"
    );
    drop(state);
    arm(None);
    committed
}
#[test]
fn three_members_apply_by_producer_and_source_epoch_under_a_constructed_schedule() {
    let _guard = checkpoint_recovery_test_guard();
    if let Ok(width) = std::env::var("WORTH_THREE_MEMBER_WIDTH") {
        println!(
            "WORTH_THREE_MEMBER_HISTORY:{:?}",
            run(width.parse().unwrap())
        );
        return;
    }
    let mut first = None;
    for width in [2, 4] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "checkpoint_recovery::required_chain::concurrent_waves::three_members::three_members_apply_by_producer_and_source_epoch_under_a_constructed_schedule", "--nocapture"])
            .env("WORTH_THREE_MEMBER_WIDTH", width.to_string()).output().unwrap();
        assert!(
            output.status.success(),
            "width {width}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let actual = stdout
            .lines()
            .find_map(|line| line.strip_prefix("WORTH_THREE_MEMBER_HISTORY:"))
            .unwrap()
            .to_owned();
        assert_eq!(actual, *first.get_or_insert_with(|| actual.clone()));
    }
}
