//! C1 (coordinator ruling 2026-11-20): a real Host Home killed after a
//! durable barrier of its graceful-close checkpoint publication reopens to its
//! pre-close durable state. A candidate root the close never published is
//! discarded by recovery, never settled as damage.
use super::super::query_readmission::native_artifacts::operations::{balance, fresh_closed, set};
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::WorthQueryApplicationMutationOutcome,
    application_installation::ApplicationBuildIdentity,
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};
use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};

const LOCATION: &str = "WORTH_CLOSE_CHECKPOINT_HOME";
const ORDINAL: &str = "WORTH_CLOSE_CHECKPOINT_ORDINAL";
const MARKERS: &str = "WORTH_CLOSE_CHECKPOINT_MARKERS";
const CHILD: &str = "process::application::interrupted_close::killed_checkpoint::close_kill_child";
/// The balance the child commits durably before it closes.
const COMMITTED: u64 = 4242;
/// The child's park announcement when the barrier fell inside its close.
const INSIDE_CLOSE: &[u8] = b"inside close";

/// File barriers of the close's checkpoint publication, by what a kill there
/// leaves on media (the barrier map in target/lane-evidence/c1-progress.md).
const PUBLICATION_STATES: [(u64, &str); 7] = [
    (
        131,
        "an anchored root's blocks written, its manifest pending",
    ),
    (
        146,
        "anchored roots published, the checkpoint's records in the WAL",
    ),
    (155, "the next anchored root's blocks written"),
    (176, "checkpoint frames entering the arena"),
    (221, "checkpoint stream candidate begun"),
    (233, "checkpoint stream candidate half written"),
    (
        245,
        "checkpoint stream written, checkpoint.current not replaced",
    ),
];
/// The checkpoint's candidate root: blocks, manifest and selector candidates
/// before the current selector names it. Every barrier, since the defect lived
/// here and the exact ordinals drift by a few between runs.
const CANDIDATE_ROOT: std::ops::RangeInclusive<u64> = 200..=212;
/// Far beyond every barrier: the child closes uninterrupted.
const UNINTERRUPTED: u64 = 1_000_000;

fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap()
}

#[test]
fn a_close_killed_in_its_checkpoint_publication_reopens_to_the_committed_state() {
    let (end, _) = killed_at(UNINTERRUPTED);
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    let mut unpublished_candidates = 0;
    let barriers = PUBLICATION_STATES
        .iter()
        .map(|(barrier, state)| (*barrier, *state))
        .chain(CANDIDATE_ROOT.map(|barrier| (barrier, "candidate root")));
    for (barrier, state) in barriers {
        let (end, unpublished) = killed_at(barrier);
        assert_eq!(
            end,
            crate::killed_child::ChildEnd::KilledAtPark,
            "{state}: barrier {barrier} is no longer inside the close; refresh the map"
        );
        unpublished_candidates += usize::from(unpublished);
    }
    assert!(
        unpublished_candidates > 0,
        "the sweep must leave a candidate root without its manifest; refresh the map"
    );
}

/// Kill a fresh Home's close after its `barrier`th file barrier, then reopen
/// it: the committed balance is served and the recovered Home closes. Returns
/// whether the kill left an unpublished candidate root behind.
fn killed_at(barrier: u64) -> (crate::killed_child::ChildEnd, bool) {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    fresh_closed(&location, execution);
    let ordinal = std::path::PathBuf::from(barrier.to_string());
    let ready = markers.join("ready");
    let end = crate::killed_child::run_and_kill(
        CHILD,
        &[
            (LOCATION, location.as_path()),
            (MARKERS, markers.as_path()),
            (ORDINAL, ordinal.as_path()),
        ],
        &ready,
        &markers.join("finished"),
    );
    if end == crate::killed_child::ChildEnd::KilledAtPark {
        assert_eq!(std::fs::read(&ready).unwrap(), INSIDE_CLOSE, "{barrier}");
    }
    let unpublished = unpublished_candidate_root(&location);
    let request = fixture::request(location, build(), Arc::default(), Arc::default());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("barrier {barrier}: the killed close reopens: {refused:?}"));
    assert_eq!(balance(home.runtime()), COMMITTED, "barrier {barrier}");
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("barrier {barrier}: recovered close: {refused:?}"));
    (end, unpublished)
}

/// Whether the newest root generation on media has routing blocks but no
/// manifest: a candidate root the killed close never published.
fn unpublished_candidate_root(location: &std::path::Path) -> bool {
    let roots = location.join("families").join("records").join("roots");
    let names: Vec<String> = std::fs::read_dir(roots)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    let newest = names
        .iter()
        .filter_map(|name| name.strip_prefix("root-")?.split('-').next())
        .filter_map(|generation| {
            u64::from_str_radix(generation.trim_end_matches(".manifest"), 16).ok()
        })
        .max();
    newest.is_some_and(|generation| !names.contains(&format!("root-{generation:016x}.manifest")))
}

#[test]
fn close_kill_child() {
    let Some(location) = std::env::var_os(LOCATION) else {
        return;
    };
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let ordinal: u64 = std::env::var(ORDINAL).unwrap().parse().unwrap();
    let media = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = media.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            // Each durable barrier: file data and metadata reach the medium.
            MediaOperationRole::SynchronizeFileState,
            ordinal,
            MediaFaultDirective::PauseAfter(gate.clone()),
        )])
        .unwrap();
    let control =
        worth_store_testkit_application_oracle::facade::host_interruption::opening::control(
            |_| {},
            Some(media.with_fault_schedule(schedule)),
            None,
        );
    let request = fixture::request(location.into(), build(), Arc::default(), Arc::default())
        .with_opening_control_for_test(control);
    let closing = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));
    let _watcher = {
        let (closing, done, markers) = (closing.clone(), done.clone(), markers.clone());
        std::thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                if gate.reached_context().is_some() {
                    let phase: &[u8] = if closing.load(Ordering::SeqCst) {
                        INSIDE_CLOSE
                    } else {
                        b"before close"
                    };
                    crate::killed_child::announce(&markers.join("ready"), phase);
                    crate::killed_child::park();
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        })
    };
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("the closed Home opens: {refused:?}"));
    assert!(matches!(
        set(home.runtime(), COMMITTED),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    closing.store(true, Ordering::SeqCst);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("uninterrupted close: {refused:?}"));
    done.store(true, Ordering::SeqCst);
    crate::killed_child::announce(&markers.join("finished"), b"closed");
    std::process::exit(0);
}
