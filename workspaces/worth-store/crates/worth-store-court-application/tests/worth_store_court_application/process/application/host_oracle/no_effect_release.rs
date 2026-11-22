//! A real cut after World's durable Open and before any Native effect: the
//! reopened Home's Query preparation durably releases that no-effect original,
//! the same key then executes fresh exactly once, and a later reopen answers
//! it from the record without running the handler again.
use super::*;
use host_observation::{projection::StateProjection, requests};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome as Outcome,
    primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryApplicationHomeLocation,
};
const LOCATION: &str = "WORTH_HOST_NO_EFFECT_HOME";
const MARKERS: &str = "WORTH_HOST_NO_EFFECT_MARKERS";
const CHILD: &str =
    "process::application::host_oracle::no_effect_release::durable_open_before_native_child";
const CUT: &str = "WORTH_HOST_NO_EFFECT_CUT";
const KEY: u64 = 0x9922;
const VALUE: u64 = 90_992;
/// The child parks World's own Open append (not durable), or the set's next
/// append (the Open durable and acknowledged, Native not yet published).
const OPEN_APPEND: &str = "open-append";
const AFTER_OPEN: &str = "after-open";

/// The negative twin parks the Open before it is durable: the same public
/// journey follows, but only the durable Open consumed an attempt identity.
#[test]
fn a_released_no_effect_original_lets_its_key_execute_once_across_reopen() {
    let unopened = fresh_attempt_after_kill_at(OPEN_APPEND);
    let released = fresh_attempt_after_kill_at(AFTER_OPEN);
    assert_eq!(
        released,
        unopened + 1,
        "the released durable Open keeps its attempt; the fresh execution takes the next"
    );
}

/// Kill the child at `cut`, reopen, execute the key fresh once and reopen
/// again. Returns the fresh execution's World attempt ordinal.
fn fresh_attempt_after_kill_at(cut: &str) -> u64 {
    let root = tempfile::tempdir().unwrap();
    let location = root.path().join("home");
    let cut_path = std::path::PathBuf::from(cut);
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[
                (LOCATION, &location),
                (MARKERS, root.path()),
                (CUT, &cut_path)
            ],
            &root.path().join("ready"),
            &root.path().join("finished")
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let before: StateProjection =
        serde_json::from_slice(&std::fs::read(root.path().join("before.json")).unwrap()).unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let open = |contacts: &Arc<AtomicUsize>| {
        host_observation::opening::open(
            WorthQueryApplicationHomeLocation::Directory(location.clone()),
            opening::build(),
            Arc::new(AtomicUsize::new(0)),
            contacts.clone(),
            None,
            Default::default(),
            &execution,
        )
    };
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(&contacts);
    let recovered = host_observation::projection::observe(home.runtime(), &execution);
    assert_eq!(recovered.head, before.head);
    assert_eq!(
        serde_json::to_value(&recovered.entities).unwrap(),
        serde_json::to_value(&before.entities).unwrap(),
        "complete Native discovery found no owner effect"
    );
    assert_eq!(
        recovery_rows(home.runtime()),
        0,
        "Query preparation released the empty original before service"
    );
    let resolution = requests::resolve_original(home.runtime(), KEY, VALUE).unwrap();
    assert!(
        matches!(resolution, Resolution::Unseen),
        "a released no-effect original leaves its key unseen: {resolution:?}"
    );
    let first = requests::set(home.runtime(), KEY, VALUE);
    assert!(
        matches!(first, Outcome::Committed { result: VALUE, .. }),
        "the released key executes fresh: {first:?}"
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let after = host_observation::projection::observe(home.runtime(), &execution);
    assert_eq!(after.occurrences.len(), before.occurrences.len() + 1);
    let again = requests::set(home.runtime(), KEY, VALUE);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(
        host_observation::projection::observe(home.runtime(), &execution).head,
        after.head
    );
    let fresh = host_observation::receipts::observe(&first, &before.head);
    let attempt = fresh.receipt.as_ref().expect("committed receipt").attempt;
    assert_eq!(
        host_observation::receipts::observe(&again, &before.head),
        fresh
    );
    drop((first, again));
    close_after_cold_release(home, &execution);
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(&contacts);
    assert_eq!(recovery_rows(home.runtime()), 0);
    let recorded = requests::set(home.runtime(), KEY, VALUE);
    assert!(
        matches!(recorded, Outcome::RecoveredCommitted(_)),
        "reopen answers the fresh original from its record: {recorded:?}"
    );
    assert_eq!(
        host_observation::receipts::observe(&recorded, &before.head),
        fresh
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(requests::balance(home.runtime()), VALUE);
    drop(recorded);
    close_after_cold_release(home, &execution);
    attempt
}

fn recovery_rows(runtime: &host_observation::OracleRuntime) -> usize {
    runtime
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .len()
}

/// The killed writer's managed run was cold-released before service, so the
/// Home closes gracefully.
fn close_after_cold_release(
    home: worth_query_host::facade::WorthQueryApplicationHome<
        worth_query_testkit_host::facade::host::application_home::schema::HomeIdentitySchema,
        worth_query_testkit_host::facade::host::application_home::HomeProgram,
    >,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let runs = home.runtime().runtime().managed_run_obligations();
    assert!(
        runs.iter().any(|run| run.cold_release.is_some()),
        "the killed writer's managed run is cold-released before service: {runs:?}"
    );
    home.close(execution)
        .unwrap_or_else(|refused| panic!("cold-released custody closes: {refused:?}"))
        .reclaim();
}

#[test]
fn durable_open_before_native_child() {
    let Some(location) = std::env::var_os(LOCATION) else {
        return;
    };
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let cut = std::env::var(CUT).unwrap();
    let past_open = match cut.as_str() {
        OPEN_APPEND => false,
        AFTER_OPEN => true,
        other => panic!("unknown no-effect cut {other}"),
    };
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let control = host_interruption::opening::control(|_| {}, None, None);
    let physical = control.physical_control_for_test();
    let home = host_observation::opening::open(
        WorthQueryApplicationHomeLocation::Directory(location.into()),
        opening::build(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        Some(control),
        Default::default(),
        &execution,
    );
    let before = host_observation::projection::observe(home.runtime(), &execution);
    std::fs::write(
        markers.join("before.json"),
        serde_json::to_vec(&before).unwrap(),
    )
    .unwrap();
    // World registers the original's custody row before its Open reaches
    // the WAL: the first append with that row is the Open. The set's next
    // append starts only after the Open is durable and acknowledged; Native
    // publishes no earlier.
    let seam = || {
        physical
            .pause_mutation_for_test(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend)
            .unwrap()
    };
    let mut gate = seam();
    std::thread::scope(|scope| {
        let worker = scope.spawn(requests::set_worker(home.runtime(), KEY, VALUE));
        let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
        let mut open_seen = false;
        loop {
            assert!(crate::process_kill::await_mutation_seam(
                &gate, &worker, deadline
            ));
            if open_seen {
                break;
            }
            open_seen = recovery_rows(home.runtime()) == 1;
            if open_seen && !past_open {
                break;
            }
            let next = seam();
            gate.release();
            gate = next;
        }
        crate::killed_child::announce(&markers.join("ready"), cut.as_bytes());
        crate::killed_child::park();
    });
}
