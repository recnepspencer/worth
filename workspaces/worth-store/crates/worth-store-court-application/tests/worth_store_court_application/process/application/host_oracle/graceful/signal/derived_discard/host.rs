//! Full Query Host D14 derived-cache discard across graceful close and kill.
use super::super::{opening, operations, session::Session, FIRST, SECOND};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use worth_query_host::facade::{
    primary_graph::WorthQueryApplicationCommitOutcome, WorthQueryApplicationHomeCheckpointDenial,
    WorthQueryApplicationHomeCloseOutcome, WorthQueryApplicationHomeLocation,
};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::{
    ApplicationBuildIdentity, ContactCounters,
};
use worth_store_application::ApplicationHomeCheckpointDenial;

const CHILD: &str =
    "process::application::host_oracle::graceful::signal::derived_discard::host::derived_discard_host_child";

#[derive(serde::Serialize, serde::Deserialize)]
enum Command {
    PrepareCheckpointDiscard,
    ReplayCommitted,
    Continue,
    Close { expect_discarded: bool },
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Reply {
    states: Vec<operations::State>,
    frontiers: Vec<(u64, u64, u64)>,
    initialized: usize,
    contacts: (usize, usize, usize, usize),
    covered: [u64; 5],
    heads: [u64; 5],
    discard_generation: Option<u64>,
    close_checkpoint_generation: Option<u64>,
    replayed_recovered: usize,
    closed: bool,
}

#[test]
fn discard_derived_graceful_close_preserves_host_truth_custody_and_idempotency() {
    for build in [FIRST, SECOND] {
        run_host_discard(false, build);
    }
}

#[test]
fn discard_derived_process_kill_preserves_host_truth_custody_and_idempotency() {
    for build in [FIRST, SECOND] {
        run_host_discard(true, build);
    }
}

fn run_host_discard(kill: bool, reopen_build: &str) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home");
    let request = super::super::super::super::opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = ContactCounters::default();
    let reference = opening::open(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        ApplicationBuildIdentity::parse(FIRST).unwrap(),
        initialized.clone(),
        contacts.clone(),
        &execution,
    );
    let reference_branches = operations::branches(&reference);
    operations::mixed(&reference, reference_branches[0], 1);
    operations::definition(&reference, reference_branches[1]);
    operations::mixed(&reference, reference_branches[2], 2);
    let expected = operations::observe(&reference, &reference_branches);

    let (mut session, opened): (Session, Reply) =
        Session::start_child(&path, &directory.path().join("first"), FIRST, CHILD);
    assert_eq!(opened.initialized, 1);
    assert_eq!(opened.contacts, (0, 0, 0, 0));

    let discarded: Reply = session.exchange(&Command::PrepareCheckpointDiscard);
    assert_eq!(discarded.states, expected);
    assert_eq!(discarded.covered[0], 0, "native coverage cleared by discard");
    let discard_generation = discarded
        .discard_generation
        .expect("discard_derived_for_test reports new anchor generation");
    assert!(discard_generation >= 2);
    if kill {
        session.kill();
    } else {
        let closed: Reply = session.exchange(&Command::Close {
            expect_discarded: true,
        });
        assert!(closed.closed);
        assert_eq!(closed.close_checkpoint_generation, None);
        session.finish();
    }

    let (mut session, recovered): (Session, Reply) =
        Session::start_child(&path, &directory.path().join("second"), reopen_build, CHILD);
    assert_eq!(recovered.initialized, 0, "reopen never initializes again");
    assert_eq!(recovered.contacts, (0, 0, 0, 0), "no eager provider calls");
    assert_eq!(
        recovered.covered, [0, 0, 0, 0, 0],
        "reopen after discard reads no checkpoint coverage"
    );
    assert!(recovered.heads[0] > 0 && recovered.heads[2] > 0);
    assert_eq!(recovered.frontiers, discarded.frontiers);
    assert_eq!(recovered.states, expected);

    let replayed: Reply = session.exchange(&Command::ReplayCommitted);
    assert_eq!(replayed.replayed_recovered, 2);
    assert_eq!(
        replayed.contacts,
        (0, 0, 0, 0),
        "replaying committed mixed keys causes zero provider contacts"
    );
    assert_eq!(replayed.frontiers, discarded.frontiers);
    assert_eq!(replayed.states, expected);

    operations::mixed(&reference, reference_branches[1], 3);
    let continued: Reply = session.exchange(&Command::Continue);
    assert_eq!(
        continued.states,
        operations::observe(&reference, &reference_branches)
    );
    assert_eq!(continued.initialized, 0);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);

    let final_close: Reply = session.exchange(&Command::Close {
        expect_discarded: false,
    });
    assert!(final_close.closed);
    assert_eq!(
        final_close.close_checkpoint_generation,
        Some(discard_generation + 1)
    );
    session.finish();

    assert!(matches!(
        reference.discard_derived_for_test(&execution),
        Err(WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired)
    ));
    let refused = reference
        .close(&execution)
        .err()
        .expect("MemoryOnly preserves lifecycle refusal");
    drop(refused.resume().unwrap());
}

fn inspect_arrays(
    home: &opening::Home,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> ([u64; 5], [u64; 5]) {
    let inspection = home.inspect_checkpoint(execution).unwrap();
    let covered = std::array::from_fn(|index| inspection.journals[index].covered);
    let heads = std::array::from_fn(|index| inspection.journals[index].head);
    (covered, heads)
}

#[test]
fn derived_discard_host_child() {
    let Some(channel) = std::env::var_os("WORTH_SIGNAL_HOME_CHANNEL").map(PathBuf::from) else {
        return;
    };
    let path = PathBuf::from(std::env::var_os("WORTH_SIGNAL_HOME_PATH").unwrap());
    let build = ApplicationBuildIdentity::parse(&std::env::var("WORTH_SIGNAL_HOME_BUILD").unwrap())
        .unwrap();
    let request = super::super::super::super::opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = ContactCounters::default();
    let mut home = Some(opening::open(
        WorthQueryApplicationHomeLocation::Directory(path),
        build,
        initialized.clone(),
        contacts.clone(),
        &execution,
    ));
    let mut branches = operations::installed(home.as_ref().unwrap());
    let respond = |reply: Reply| {
        crate::killed_child::announce(
            &channel.join("response"),
            &serde_json::to_vec(&reply).unwrap(),
        );
    };
    let (covered, heads) = inspect_arrays(home.as_ref().unwrap(), &execution);
    respond(Reply {
        states: operations::observe(home.as_ref().unwrap(), &branches),
        frontiers: operations::frontiers(home.as_ref().unwrap(), &branches),
        initialized: initialized.load(Ordering::SeqCst),
        contacts: contacts.snapshot(),
        covered,
        heads,
        discard_generation: None,
        close_checkpoint_generation: None,
        replayed_recovered: 0,
        closed: false,
    });
    loop {
        let req_path = channel.join("request");
        if !req_path.is_file() {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let command: Command =
            serde_json::from_slice(&std::fs::read(&req_path).unwrap()).unwrap();
        std::fs::remove_file(req_path).unwrap();
        let mut discard_generation = None;
        let mut replayed_recovered = 0;
        match command {
            Command::PrepareCheckpointDiscard => {
                let home = home.as_ref().unwrap();
                branches = operations::branches(home);
                operations::mixed(home, branches[0], 1);
                operations::definition(home, branches[1]);
                operations::mixed(home, branches[2], 2);
                let checkpoint = home.checkpoint(&execution).unwrap();
                assert!(checkpoint.generation() >= 1);
                let (before_cov, before_head) = inspect_arrays(home, &execution);
                assert!(before_cov[0] > 0 && before_cov[0] == before_head[0]);
                let discarded = home.discard_derived_for_test(&execution).unwrap();
                assert_eq!(discarded, checkpoint.generation() + 1);
                discard_generation = Some(discarded);
            }
            Command::ReplayCommitted => {
                let home = home.as_ref().unwrap();
                for (branch, ordinal) in [(branches[0], 1_u8), (branches[2], 2_u8)] {
                    match operations::commit_mixed(home, branch, ordinal) {
                        WorthQueryApplicationCommitOutcome::RecoveredCommitted(receipt) => {
                            assert!(receipt.world_journal_position().is_some());
                            assert!(receipt.native_journal_position().0 > 0);
                            replayed_recovered += 1;
                        }
                        other => panic!(
                            "expected RecoveredCommitted for ordinal {ordinal}, got {other:?}"
                        ),
                    }
                }
            }
            Command::Continue => operations::mixed(home.as_ref().unwrap(), branches[1], 3),
            Command::Close { expect_discarded } => {
                let closed = home
                    .take()
                    .unwrap()
                    .close(&execution)
                    .unwrap_or_else(|cause| panic!("host Home close: {cause:?}"));
                assert!(closed.is_closed());
                let close_checkpoint_generation = match closed.outcome() {
                    WorthQueryApplicationHomeCloseOutcome::Closed {
                        checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
                        ..
                    } if expect_discarded => None,
                    WorthQueryApplicationHomeCloseOutcome::Closed {
                        checkpoint: Ok(checkpoint),
                        ..
                    } if !expect_discarded => Some(checkpoint.generation()),
                    WorthQueryApplicationHomeCloseOutcome::Closed { checkpoint, .. } => {
                        panic!(
                            "unexpected close checkpoint (expect_discarded={expect_discarded}): {checkpoint:?}"
                        )
                    }
                    WorthQueryApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
                        panic!(
                            "unexpected CloseRefused (expect_discarded={expect_discarded}): {reason:?}"
                        )
                    }
                };
                closed.reclaim();
                respond(Reply {
                    states: Vec::new(),
                    frontiers: Vec::new(),
                    initialized: initialized.load(Ordering::SeqCst),
                    contacts: contacts.snapshot(),
                    covered: [0; 5],
                    heads: [0; 5],
                    discard_generation: None,
                    close_checkpoint_generation,
                    replayed_recovered: 0,
                    closed: true,
                });
                return;
            }
        }
        let (covered, heads) = inspect_arrays(home.as_ref().unwrap(), &execution);
        respond(Reply {
            states: operations::observe(home.as_ref().unwrap(), &branches),
            frontiers: operations::frontiers(home.as_ref().unwrap(), &branches),
            initialized: initialized.load(Ordering::SeqCst),
            contacts: contacts.snapshot(),
            covered,
            heads,
            discard_generation,
            close_checkpoint_generation: None,
            replayed_recovered,
            closed: false,
        });
    }
}
