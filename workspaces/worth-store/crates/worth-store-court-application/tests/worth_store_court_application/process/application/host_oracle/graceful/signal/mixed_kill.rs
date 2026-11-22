//! A real Home is killed inside one mixed conditional-definition commit, after
//! Native's effect is durable and again after both owner effects; a fresh
//! process continues the original once, without re-executing either owner.
use super::{opening, operations, session::Session};
use std::path::PathBuf;
use std::sync::{atomic::AtomicUsize, Arc};
use std::time::Duration;
use worth_query_host::facade::{primary_graph, WorthQueryApplicationHomeLocation};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::{ApplicationBuildIdentity, ContactCounters};
use worth_runtime_world::facade::{ProductUnpublishedNextAction, WorldRecoveredContinuationOutcome};

const CHILD: &str =
    "process::application::host_oracle::graceful::signal::mixed_kill::signal_mixed_kill_child";
const BUILD: &str = "11111111-1111-4111-8111-111111111111";

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
enum Cut {
    /// Native's effect is durable; Signal's planned leg has not begun its write.
    NativeDurable,
    /// Both owner effects are durable; World's occurrence is not.
    BothDurable,
}
#[derive(serde::Serialize, serde::Deserialize)]
enum Request {
    Commit(u8),
    Cut(Cut, u8),
    Recover(u8),
    Close,
}
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct Observation {
    states: Vec<operations::State>,
    frontiers: Vec<(u64, u64, u64)>,
    recovered_rows: usize,
}
/// The recovered original as Query and World answer it in the fresh process.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct Recovered {
    owner_effects: usize,
    continue_offered: bool,
    /// Both-durable cut only: a fresh key's answer before any recovery.
    fresh_key: Option<FreshKey>,
    duplicate_key_is_original: bool,
    published: bool,
    key_answers_recovered_commit: bool,
    after: Observation,
}

/// A fresh key while Signal's leg is durable but unpublished: Bridge serves
/// the successor, so the product's basis has no installed predecessor.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct FreshKey {
    denied_product_basis_stale_at_bridge_planning: bool,
    only_the_original_recovered: bool,
    frontiers_unchanged: bool,
}

#[test]
fn killed_with_only_native_durable_the_mixed_original_continues_its_signal_leg_once() {
    journey(Cut::NativeDurable, 1);
}
#[test]
fn killed_with_both_owner_effects_durable_the_mixed_original_publishes_once() {
    journey(Cut::BothDurable, 2);
}

fn journey(cut: Cut, acting_owners: usize) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home");
    let expected = reference();
    let mut generation = 0;
    let start = |generation: &mut usize| -> (Session, Observation) {
        *generation += 1;
        let channel = directory.path().join(format!("session-{generation}"));
        Session::start_child(&path, &channel, BUILD, CHILD)
    };
    let (mut session, fresh) = start(&mut generation);
    let first: Observation = session.exchange(&Request::Commit(1));
    assert_eq!(first.states, expected[0]);
    session.kill();
    // Positive control: a quiescent kill leaves nothing to recover.
    let (mut session, quiet) = start(&mut generation);
    assert_eq!(quiet, first);
    let parked: bool = session.exchange(&Request::Cut(cut, 2));
    assert!(parked, "the original reached its cut");
    session.kill();
    let (mut session, reopened) = start(&mut generation);
    assert_eq!(
        reopened.recovered_rows, 1,
        "exactly the original is recovered"
    );
    assert_eq!(
        reopened.states, first.states,
        "the product stays at its predecessor"
    );
    assert_eq!(reopened.frontiers, first.frontiers);
    let recovered: Recovered = session.exchange(&Request::Recover(2));
    assert_eq!(
        recovered.owner_effects, acting_owners,
        "exactly the acting owners"
    );
    assert!(recovered.continue_offered);
    assert_eq!(
        recovered.fresh_key,
        (acting_owners == 2).then_some(FreshKey {
            denied_product_basis_stale_at_bridge_planning: true,
            only_the_original_recovered: true,
            frontiers_unchanged: true,
        }),
        "a fresh key is refused before any effect, after its key resolves"
    );
    assert!(
        recovered.duplicate_key_is_original,
        "the original key answers its retained original"
    );
    assert!(recovered.published, "the original continues once");
    assert!(recovered.key_answers_recovered_commit);
    assert_eq!(
        recovered.after.states, expected[1],
        "one Native and one Signal effect, as uninterrupted"
    );
    assert_eq!(recovered.after.recovered_rows, 0);
    for ((before, after), (initial, once)) in first
        .frontiers
        .iter()
        .zip(&recovered.after.frontiers)
        .zip(fresh.frontiers.iter().zip(&first.frontiers))
    {
        assert_eq!(
            (after.0 - before.0, after.1 - before.1, after.2 - before.2),
            (once.0 - initial.0, once.1 - initial.1, once.2 - initial.2),
            "the continuation moves each owner exactly as one uninterrupted commit"
        );
    }
    session.kill();
    let (mut session, again) = start(&mut generation);
    assert_eq!(
        again.states, recovered.after.states,
        "a second reopen serves the same definitions"
    );
    assert_eq!(again.frontiers, recovered.after.frontiers);
    assert_eq!(again.recovered_rows, 0);
    // The killed writer's managed run was cold-released before service.
    let closed_gracefully: bool = session.exchange(&Request::Close);
    assert!(closed_gracefully, "cold-released custody closes gracefully");
    session.finish();
    let (mut session, after_close) = start(&mut generation);
    assert_eq!(
        after_close, again,
        "a graceful reopen serves the same state"
    );
    let closed_gracefully: bool = session.exchange(&Request::Close);
    assert!(closed_gracefully, "the reopened Home closes gracefully");
    session.finish();
}

/// The same two commits on an uninterrupted MemoryOnly Home.
fn reference() -> [Vec<operations::State>; 2] {
    let request = super::super::super::opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = opening::open(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        ApplicationBuildIdentity::parse(BUILD).unwrap(),
        Arc::new(AtomicUsize::new(0)),
        ContactCounters::default(),
        &execution,
    );
    let branches = operations::installed(&home);
    operations::mixed(&home, branches[0], 1);
    let first = operations::observe(&home, &branches);
    operations::mixed(&home, branches[0], 2);
    let second = operations::observe(&home, &branches);
    drop(home.close(&execution).err().unwrap().resume().unwrap());
    [first, second]
}

#[test]
fn signal_mixed_kill_child() {
    let Some(channel) = std::env::var_os("WORTH_SIGNAL_HOME_CHANNEL").map(PathBuf::from) else {
        return;
    };
    let path = PathBuf::from(std::env::var_os("WORTH_SIGNAL_HOME_PATH").unwrap());
    let build = ApplicationBuildIdentity::parse(&std::env::var("WORTH_SIGNAL_HOME_BUILD").unwrap())
        .unwrap();
    let request = super::super::super::opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = opening::open(
        WorthQueryApplicationHomeLocation::Directory(path),
        build,
        Arc::new(AtomicUsize::new(0)),
        ContactCounters::default(),
        &execution,
    );
    let main = operations::installed(&home)[0];
    let observe = |home: &opening::Home| Observation {
        states: operations::observe(home, &[main]),
        frontiers: operations::frontiers(home, &[main]),
        recovered_rows: home
            .runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::new(16).unwrap())
            .unwrap()
            .rows()
            .len(),
    };
    let respond = |bytes: Vec<u8>| crate::killed_child::announce(&channel.join("response"), &bytes);
    let answer = |home: &opening::Home| serde_json::to_vec(&observe(home)).unwrap();
    respond(answer(&home));
    loop {
        let path = channel.join("request");
        if !path.is_file() {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let command: Request = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        match command {
            Request::Commit(ordinal) => {
                operations::mixed(&home, main, ordinal);
                respond(answer(&home));
            }
            Request::Cut(cut, ordinal) => {
                // The parked original holds its branch's publication gate, so
                // only the arrival is answered; the product is observed on reopen.
                park(&home, main, cut, ordinal, || {
                    respond(serde_json::to_vec(&true).unwrap())
                })
            }
            Request::Recover(ordinal) => {
                let recovered = recover(&home, main, ordinal, &execution, &observe);
                respond(serde_json::to_vec(&recovered).unwrap());
            }
            Request::Close => {
                respond(serde_json::to_vec(&close(home, &execution)).unwrap());
                return;
            }
        }
    }
}

/// Close gracefully: the killed writer's managed run was cold-released before
/// service. Returns whether the Home closed.
fn close(home: opening::Home, execution: &worth_execution::ExecutionRequest<'_, '_>) -> bool {
    let closed = home
        .close(execution)
        .unwrap_or_else(|refused| panic!("cold-released custody closes: {refused:?}"));
    let is_closed = closed.is_closed();
    closed.reclaim();
    is_closed
}

/// Run the commit until the cut's real owner boundary, answer, and stay parked
/// until the parent kills this process.
fn park(
    home: &opening::Home,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    cut: Cut,
    ordinal: u8,
    parked: impl FnOnce(),
) {
    let app = home.runtime();
    let bound = Duration::from_secs(120);
    std::thread::scope(|scope| {
        let reached = match cut {
            Cut::NativeDurable => {
                let control = app.signal_operation_control_for_test().unwrap();
                let pause =
                    worth_store_testkit_application_oracle::host_interruption::signal_write::pause_before_signal_durable_write(&control);
                scope.spawn(|| operations::commit_mixed(home, branch, ordinal));
                let reached = pause.wait_until_reached(bound);
                std::mem::forget(pause);
                reached
            }
            Cut::BothDurable => {
                let pause = app
                    .world_operation_control_for_test()
                    .pause_before_product_compare_for_test(std::num::NonZeroUsize::MIN);
                scope.spawn(|| operations::commit_mixed(home, branch, ordinal));
                let reached = pause.wait_until_reached(bound);
                std::mem::forget(pause);
                reached
            }
        };
        assert!(reached, "the mixed commit reaches its cut");
        parked();
        let until = std::time::Instant::now() + bound;
        while std::time::Instant::now() < until {
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("parent did not kill the parked mixed commit");
    });
}

fn recover(
    home: &opening::Home,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    ordinal: u8,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    observe: &dyn Fn(&opening::Home) -> Observation,
) -> Recovered {
    let app = home.runtime();
    let page = app
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    let handle = page.rows()[0].handle().clone();
    let recovery = app.readmit_product_publication_recovery(&handle).unwrap();
    let original = recovery.inspect().unwrap();
    let owner_effects = original.owner_effect_count();
    let continue_offered = original
        .next_actions()
        .contains(&ProductUnpublishedNextAction::ContinueOriginalPublication);
    drop(original);
    let fresh_key = (owner_effects == 2).then(|| fresh_key(home, branch, ordinal, &handle));
    // The same request: its key resolves before any Bridge basis check, so
    // it answers the retained original in both cuts.
    let duplicate_key_is_original = match operations::commit_mixed(home, branch, ordinal) {
        primary_graph::WorthQueryApplicationCommitOutcome::ProductUnpublished(original) => {
            original.into_recovery().record_handle() == &handle
        }
        other => panic!("the original key answers its retained original: {other:?}"),
    };
    let published = loop {
        match recovery.continue_original_in_request(execution).unwrap() {
            WorldRecoveredContinuationOutcome::Pending(custody) => drop(custody),
            WorldRecoveredContinuationOutcome::Published { .. } => break true,
            other => panic!("the recovered mixed original continues: {other:?}"),
        }
    };
    let key_answers_recovered_commit = matches!(
        operations::commit_mixed(home, branch, ordinal),
        primary_graph::WorthQueryApplicationCommitOutcome::RecoveredCommitted(_)
    );
    Recovered {
        owner_effects,
        continue_offered,
        fresh_key,
        duplicate_key_is_original,
        published,
        key_answers_recovered_commit,
        after: observe(home),
    }
}

/// Another key's mixed commit on the same product, before any recovery.
fn fresh_key(
    home: &opening::Home,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    ordinal: u8,
    original: &worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
) -> FreshKey {
    let before = operations::frontiers(home, &[branch]);
    let denied = match operations::commit_mixed(home, branch, ordinal + 100) {
        primary_graph::WorthQueryApplicationCommitOutcome::Denied(denial) => {
            denial.kind() == primary_graph::WorthQueryApplicationCommitDenialKind::ProductBasisStale
                && denial.stage()
                    == primary_graph::WorthQueryApplicationCommitDenialStage::BridgePlanning
        }
        other => panic!("a fresh key refuses before any effect: {other:?}"),
    };
    let page = home
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(16).unwrap())
        .unwrap();
    FreshKey {
        denied_product_basis_stale_at_bridge_planning: denied,
        only_the_original_recovered: page.rows().len() == 1 && page.rows()[0].handle() == original,
        frontiers_unchanged: operations::frontiers(home, &[branch]) == before,
    }
}
