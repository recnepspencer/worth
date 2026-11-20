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

/// Reopen through the same Host transition request; the transition must
/// resume to B's terminal state, then B serves a write and reopens plainly.
fn resumes_to_the_uninterrupted_end(location: &std::path::Path, seam: usize) {
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let contacts = Arc::new(AtomicUsize::new(0));
    let request = successor::transition_request(
        location.to_owned(),
        build(),
        contacts.clone(),
        Arc::new(AtomicUsize::new(0)),
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
        .idempotency(&(0x5ea0_0000 + seam as u64))
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
}

// One sweep exceeds a single case's runtime guard, so consecutive ranges
// cover every barrier. Barriers after the durable binding belong to B's
// timing-dependent service start, so the total varies between runs and any
// range may reach the uninterrupted completion; the last range is open.
#[test]
fn a_transition_killed_at_barriers_1_to_50_resumes_to_its_uninterrupted_end() {
    sweep(1, Some(50));
}
#[test]
fn a_transition_killed_at_barriers_51_to_100_resumes_to_its_uninterrupted_end() {
    sweep(51, Some(100));
}
#[test]
fn a_transition_killed_at_barriers_101_to_150_resumes_to_its_uninterrupted_end() {
    sweep(101, Some(150));
}
#[test]
fn a_transition_killed_at_barriers_151_to_200_resumes_to_its_uninterrupted_end() {
    sweep(151, Some(200));
}
#[test]
fn a_transition_killed_at_barriers_201_to_250_resumes_to_its_uninterrupted_end() {
    sweep(201, Some(250));
}
#[test]
fn a_transition_killed_at_barriers_251_to_300_resumes_to_its_uninterrupted_end() {
    sweep(251, Some(300));
}
#[test]
fn a_transition_killed_at_barriers_301_to_350_resumes_to_its_uninterrupted_end() {
    sweep(301, Some(350));
}
#[test]
fn a_transition_killed_at_every_later_barrier_resumes_to_its_uninterrupted_end() {
    sweep(351, None);
}

/// Kill at each barrier of `first..=last` (open when `last` is `None`); any
/// range ends at the barrier the uninterrupted transition completes before.
fn sweep(first: usize, last: Option<usize>) {
    for seam in first.. {
        if last.is_some_and(|last| seam > last) {
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let location = directory.path().join("home");
        let markers = directory.path().join("markers");
        std::fs::create_dir(&markers).unwrap();
        predecessor(&location);
        let ordinal = std::path::PathBuf::from(seam.to_string());
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
        resumes_to_the_uninterrupted_end(&location, seam);
        if end == crate::killed_child::ChildEnd::Finished {
            // The uninterrupted control: the transition completed before
            // this barrier, and its end matches every killed prefix's end.
            println!("swept {} durable transition barriers", seam - 1);
            assert!(seam > 1, "at least one durable barrier precedes completion");
            return;
        }
    }
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
