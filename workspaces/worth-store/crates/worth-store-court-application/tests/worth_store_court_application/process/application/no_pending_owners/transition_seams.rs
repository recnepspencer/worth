//! C1.4: a real Host program transition killed after each durable media
//! barrier (intent, every step and binding) reopens through the same Host
//! request, resumes exactly its missing suffix and ends where the
//! uninterrupted transition ends.
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    application_installation::{ApplicationBuildIdentity, WorthQueryProgramApplicationRuntime},
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
    program_transition_successor as successor,
};
use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};

const LOCATION: &str = "WORTH_TRANSITION_SEAM_HOME";
const ORDINAL: &str = "WORTH_TRANSITION_SEAM_ORDINAL";
const MARKERS: &str = "WORTH_TRANSITION_SEAM_MARKERS";
const CHILD: &str =
    "process::application::no_pending_owners::transition_seams::transition_seam_child";
type B =
    WorthQueryProgramApplicationRuntime<successor::SuccessorSchema, successor::ExtendedProgram>;

fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap()
}

/// A real A Home, initialized and closed.
fn predecessor(location: &std::path::Path) {
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let request = fixture::request(
        location.to_owned(),
        build(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    let home = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| Box::new(request.open_in_advancement(&phase).unwrap()),
    )
    .unwrap();
    (*home)
        .close(&worth_execution::ExecutionRequest::leased(&lease))
        .unwrap_or_else(|cause| panic!("A closes: {cause:?}"));
}

fn balance(runtime: &B) -> u64 {
    let (_, scope) = scope();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    runtime
        .request(&principal, &scope)
        .query(successor::balance_query::AccountRead("account-001".into()))
        .execute()
        .unwrap()
        .rows()[0]
}

fn scope() -> (
    worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource,
    worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) {
    use worth_query_host::facade::admission::authenticated_principal::*;
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    (cancellation, scope)
}

/// What a resumed open observed: World's installed commits (one per bootstrap
/// and per transition step occurrence, so a repeated or lost step changes it)
/// and how many times this process ran the plan author.
#[derive(Debug, PartialEq, Eq)]
struct Terminal {
    installed_commits: usize,
    authored: usize,
}

/// Reopen through the same Host transition request; the transition must
/// resume to B's terminal state, then B serves a write and reopens plainly.
fn resumes_to_the_uninterrupted_end(location: &std::path::Path, seam: u64) -> Terminal {
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let contacts = Arc::new(AtomicUsize::new(0));
    let authored = Arc::new(AtomicUsize::new(0));
    let request = successor::transition_request(
        location.to_owned(),
        build(),
        contacts.clone(),
        authored.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("seam {seam}: the transition resumes: {cause:?}")),
        )
    })
    .unwrap();
    assert_eq!(
        balance(home.runtime()),
        10,
        "seam {seam}: terminal transition state"
    );
    let terminal = Terminal {
        installed_commits: home
            .runtime()
            .runtime()
            .world_history_snapshot_for_test()
            .installed_commits(),
        authored: authored.load(std::sync::atomic::Ordering::SeqCst),
    };
    let (_, scope) = scope();
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &scope,
    );
    let written = home
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(successor::native_mutation::SetAccountBalance {
            account: "account-001".into(),
            balance: 11,
        })
        .without_source()
        .idempotency(&(0x5ea0_0000 + seam))
        .execute_in_program(
            home.runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            written,
            WorthQueryApplicationMutationOutcome::Committed { result: 11, .. }
        ),
        "seam {seam}: B serves after the resumed transition"
    );
    drop(written);
    assert_eq!(contacts.load(std::sync::atomic::Ordering::SeqCst), 1);
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("seam {seam}: B closes: {cause:?}"));
    let request = successor::request(location.to_owned(), build(), Arc::default());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("seam {seam}: ordinary B reopens: {cause:?}")),
        )
    })
    .unwrap();
    assert_eq!(balance(home.runtime()), 11, "seam {seam}");
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("seam {seam}: ordinary B closes: {cause:?}"));
    terminal
}

/// The first durable barrier of each durable state of the open (the barrier
/// map in target/lane-evidence/c1-progress.md), and whether resuming from it
/// runs the plan author again (every state before the binding is durable).
const DURABLE_STATES: [(u64, &str, bool); 12] = [
    (1, "before the durable intent", true),
    (24, "intent durable", true),
    (44, "step 0 open durable", true),
    (64, "step 0 Native effect durable", true),
    (85, "step 0 durable", true),
    (105, "target step open durable", true),
    (125, "target step Native effect durable", true),
    (152, "target step durable", true),
    (172, "step 2 open durable", true),
    (192, "step 2 Native effect durable", true),
    (213, "all steps durable, binding pending", true),
    (233, "binding durable", false),
];
/// Far beyond every barrier: the child completes the open uninterrupted.
const UNINTERRUPTED: u64 = 1_000_000;

#[test]
fn a_transition_killed_in_each_durable_state_resumes_to_its_uninterrupted_end() {
    let (end, control) = killed_at(UNINTERRUPTED);
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    assert_eq!(
        control.authored, 0,
        "a completed transition is never re-planned"
    );
    for (barrier, state, reauthored) in DURABLE_STATES {
        let (end, resumed) = killed_at(barrier);
        assert_eq!(
            end,
            crate::killed_child::ChildEnd::KilledAtPark,
            "{state}: barrier {barrier} is no longer inside the open; refresh the map"
        );
        assert_eq!(
            resumed.installed_commits, control.installed_commits,
            "{state}: every step occurs exactly once, as in the uninterrupted run"
        );
        assert_eq!(resumed.authored, usize::from(reauthored), "{state}");
    }
}

/// Kill a fresh A Home's transition after `barrier`, then resume it.
fn killed_at(barrier: u64) -> (crate::killed_child::ChildEnd, Terminal) {
    let (_directory, location, end) = killed_home(barrier);
    (end, resumes_to_the_uninterrupted_end(&location, barrier))
}

/// Kill a fresh A Home's transition after `barrier`; nothing reopens it yet.
fn killed_home(
    barrier: u64,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    crate::killed_child::ChildEnd,
) {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    predecessor(&location);
    let ordinal = std::path::PathBuf::from(barrier.to_string());
    let end = crate::killed_child::run_and_kill(
        CHILD,
        &[
            (LOCATION, location.as_path()),
            (MARKERS, markers.as_path()),
            (ORDINAL, ordinal.as_path()),
        ],
        &markers.join("ready"),
        &markers.join("finished"),
    );
    (directory, location, end)
}

/// A resumed plan whose recomputed body differs from the recorded one refuses
/// with TransitionPlanDrift before any new effect: resuming afterwards with
/// the original request still installs exactly the uninterrupted commits.
#[test]
fn a_resumed_transition_refuses_a_changed_plan_without_extending_its_prefix() {
    use worth_query_host::facade::{
        WorthQueryHomeProgramTransitionDenial as Denial,
        WorthQueryHomeProgramTransitionOpenPhase as Phase,
    };
    use worth_store_application::{
        ApplicationHomeTransitionIntentDenial as Intent,
        ApplicationHomeTransitionResumptionDenial as Resumption,
    };
    let (_, control) = killed_at(UNINTERRUPTED);
    // Step 0's Native effect is durable (DURABLE_STATES).
    let (_directory, location, end) = killed_home(64);
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark);
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let changed = successor::authored_transition_request(
        location.clone(),
        build(),
        Arc::default(),
        Arc::default(),
        7,
    );
    let refused = match with_bootstrap_advancement_from_request(execution, |phase| {
        changed.open_in_advancement(&phase).map(Box::new)
    })
    .unwrap()
    {
        Ok(_) => panic!("a changed plan cannot resume the recorded transition"),
        Err(refused) => refused,
    };
    let Phase::Transition { cause, .. } = refused.phase() else {
        panic!("drift refuses inside the transition phase: {refused:?}");
    };
    assert!(
        matches!(
            cause,
            Denial::Resumption(Resumption::Intent(Intent::TransitionPlanDrift))
        ),
        "{cause:?}"
    );
    drop(refused);
    let resumed = resumes_to_the_uninterrupted_end(&location, 64);
    assert_eq!(
        resumed.installed_commits, control.installed_commits,
        "the refused drift extended nothing"
    );
}

#[test]
fn transition_seam_child() {
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
    let request =
        successor::transition_request(location.into(), build(), Arc::default(), Arc::default())
            .with_opening_control_for_test(control);
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    // The request stays on this thread; the watcher holds only the media gate.
    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let _watcher = {
        let (done, markers) = (done.clone(), markers.clone());
        std::thread::spawn(move || {
            while !done.load(std::sync::atomic::Ordering::SeqCst) {
                if gate.reached_context().is_some() {
                    crate::killed_child::announce(&markers.join("ready"), b"durable barrier held");
                    crate::killed_child::park();
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        })
    };
    with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| {
            let _home = request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("uninterrupted transition: {cause:?}"));
            // The sweep covers exactly the transition open: B is never closed
            // here, so no close checkpoint's barriers enter the sweep.
            done.store(true, std::sync::atomic::Ordering::SeqCst);
            crate::killed_child::announce(&markers.join("finished"), b"transition completed");
            std::process::exit(0);
        },
    )
    .unwrap();
}
