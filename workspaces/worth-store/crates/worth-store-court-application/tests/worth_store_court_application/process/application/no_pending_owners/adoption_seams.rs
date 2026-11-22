//! C3.4 / Phase 6: real process kills at each exact durable owner-record
//! adoption boundary, stopped branch-set kills, and support retirement across kills.
mod retirement;
mod single_branch;
mod stopped_and_retirement;

use std::num::{NonZeroU64, NonZeroUsize};
use std::path::Path;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::Duration;
use worth_foundational::facade::DurableCompositeCommitIdentity;
use worth_query_execution::facade::application_home_integration::WorthQueryBranchAdoptionStageForTest as AdoptionStage;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
        WorthQueryPerformedBranchAdoption,
    },
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationProgramRoster, WorthQueryProgramOwner,
        WorthQueryProgramSupportRetirementFailure,
    },
    declaration::application_program::ApplicationProgramRevision,
    domain::WorthQueryProgramSupportRetirementDenial,
    product::{
        WorthQueryProductBranch, WorthQueryProductBranchCreateError,
        WorthQueryProductBranchCreationContinuation, WorthQueryProductBranchRecoveryDenial,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location, WorthQueryApplicationHomeRequest,
};
use worth_runtime_world::facade::CompositeCommitIdentity;
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, seed_host, SEED_RETENTION},
    model_lifecycle::validated_model_lifecycle_program,
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    programs::{
        validated_first_program, validated_removed_operation_program, validated_second_program,
        RetentionProgramP0, RetentionProgramP1,
    },
    readback::read_retention,
    schema::DocumentRetentionSchema as Schema,
    settled_verdict::{settle, RetentionVerdict},
};
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_store::physical_runtime::certification::{
    CertificationMediaFaultActivation, MediaFaultDirective, MediaOperationRole,
};
use worth_store_application::ApplicationHomeOpeningControlForTest;

const LOCATION: &str = "WORTH_ADOPTION_SEAM_HOME";
const MODE: &str = "WORTH_ADOPTION_SEAM_MODE";
const MARKERS: &str = "WORTH_ADOPTION_SEAM_MARKERS";
const CHILD: &str =
    "process::application::no_pending_owners::adoption_seams::adoption_seam_child";

pub(super) type Home = WorthQueryApplicationHome<Schema, RetentionProgramP0>;

pub(super) enum ExpectedAdoptionCommit<'a> {
    Any,
    Live(&'a CompositeCommitIdentity),
    Durable(DurableCompositeCommitIdentity),
}

pub(super) fn open_home(
    location: &Path,
    initialize: bool,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    control: Option<ApplicationHomeOpeningControlForTest>,
) -> Home {
    let mut request = WorthQueryApplicationHomeRequest::new(
        Location::Directory(location.to_owned()),
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_removed_operation_program())
            .support(validated_actor_program())
            .support(validated_model_lifecycle_program()),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        host_limits(WorthQueryAdmissionClock::system()),
        move |phase, graph, schema| {
            assert!(initialize, "reopen must not re-bootstrap an existing Home");
            seed_host(phase, graph, schema)
        },
    );
    if let Some(control) = control {
        request = request.with_opening_control_for_test(control);
    }
    *with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("adoption Home open (init={initialize}): {cause:?}")),
        )
    })
    .unwrap()
}

pub(super) fn fork_branch(home: &Home, source: WorthQueryProductBranch) -> WorthQueryProductBranch {
    let host = home.runtime();
    host.with_host_advancement(|phase| {
        let mut recovery = match host
            .runtime()
            .branches()
            .fork(source)
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
        {
            Ok(branch) => return branch,
            Err(WorthQueryProductBranchCreateError::ProductUnpublished(r)) => r,
            Err(cause) => panic!("fork branch: {cause:?}"),
        };
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            assert!(std::time::Instant::now() < deadline, "fork must settle");
            match recovery.continue_in_advancement(&phase) {
                Ok(WorthQueryProductBranchCreationContinuation::Performed(branch)) => return branch,
                Ok(other) => panic!("unexpected fork terminal: {other:?}"),
                Err(failure) => {
                    assert_eq!(failure.denial(), WorthQueryProductBranchRecoveryDenial::Busy);
                    recovery = failure.into_recovery();
                    std::thread::yield_now();
                }
            }
        }
    })
    .unwrap()
}

pub(super) fn predecessor(location: &Path, with_fork: bool) {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_home(location, true, execution, None);
    if with_fork {
        let root = home.runtime().current_world();
        let _child = fork_branch(&home, root);
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("adoption predecessor closes: {cause:?}"))
        .reclaim();
}

pub(super) fn run_child(location: &Path, mode: &str) -> (crate::killed_child::ChildEnd, String) {
    let markers_dir = tempfile::tempdir().unwrap();
    let mode_path = std::path::PathBuf::from(mode);
    let (ready, finished) = (markers_dir.path().join("ready"), markers_dir.path().join("finished"));
    let end = crate::killed_child::run_and_kill(
        CHILD,
        &[(LOCATION, location), (MARKERS, markers_dir.path()), (MODE, mode_path.as_path())],
        &ready,
        &finished,
    );
    let detail = std::fs::read_to_string(&ready).or_else(|_| std::fs::read_to_string(&finished)).unwrap_or_default();
    (end, detail)
}

pub(super) fn assert_branch_revision(
    home: &Home,
    branch: WorthQueryProductBranch,
    expected: &ApplicationProgramRevision,
    label: &str,
) {
    let host = home.runtime();
    host.with_host_advancement(|phase| {
        let selected = host.runtime().on_branch(branch).select(&phase).unwrap();
        assert_eq!(selected.inspect_selected_program().unwrap().revision(), expected, "{label}");
    })
    .unwrap();
}

/// Traverses durable branch-local history oldest-to-newest and asserts exactly
/// one P0 -> P1 adoption transition matching `expected_commit`.
pub(super) fn assert_single_p0_to_p1_occurrence(
    home: &Home,
    branch: WorthQueryProductBranch,
    expected_history_len: Option<usize>,
    expected_commit: ExpectedAdoptionCommit<'_>,
    label: &str,
) -> (usize, CompositeCommitIdentity) {
    let host = home.runtime();
    let p0 = *host.owned_revision();
    let p1 = *host.supported_program::<RetentionProgramP1>().unwrap().owned_revision();
    let mut oldest_first: Vec<(CompositeCommitIdentity, ApplicationProgramRevision)> = host
        .with_host_advancement(|phase| {
            let history = host.runtime().branches().history(&phase, branch, None).unwrap();
            assert!(history.is_complete(), "{label}: full branch history must be traversed");
            history
                .entries()
                .map(|entry| {
                    let commit = entry.selected_commit().clone();
                    let rev = *history.select(&phase, &entry).unwrap().inspect_selected_program().unwrap().revision();
                    (commit, rev)
                })
                .collect()
        })
        .unwrap();
    oldest_first.reverse();
    if let Some(expected_len) = expected_history_len {
        assert_eq!(oldest_first.len(), expected_len, "{label}: branch history length parity");
    }
    assert_eq!(oldest_first.first().map(|(_, r)| r), Some(&p0), "{label}: branch starts on P0");
    assert_eq!(oldest_first.last().map(|(_, r)| r), Some(&p1), "{label}: branch ends on P1");
    let mut transitions = Vec::new();
    for pair in oldest_first.windows(2) {
        if pair[0].1 != pair[1].1 {
            transitions.push((pair[0].1, pair[1].1, pair[1].0.clone()));
        }
    }
    assert_eq!(transitions.len(), 1, "{label}: expected exactly one program transition: {transitions:?}");
    let (from, to, commit) = &transitions[0];
    assert_eq!((from, to), (&p0, &p1), "{label}: single transition must be P0 -> P1");
    match expected_commit {
        ExpectedAdoptionCommit::Any => {}
        ExpectedAdoptionCommit::Live(exp) => assert_eq!(commit, exp, "{label}: commit matches receipt"),
        ExpectedAdoptionCommit::Durable(exp) => assert_eq!(
            (commit.owner_identity().durable(), commit.ordinal()),
            (Some(exp.owner()), exp.ordinal()),
            "{label}: commit matches recorded prefix"
        ),
    }
    (oldest_first.len(), commit.clone())
}

pub(super) fn adopt_branch_to_p1(home: &Home) -> WorthQueryPerformedBranchAdoption {
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let p0 = *host.owned_revision();
    let target = *host.supported_program::<RetentionProgramP1>().unwrap().owned_revision();
    let programs = host.runtime().request(&principal, &scope).on_branch(host.current_world()).programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs.adopt(&requirements).expect("adoption admits").prepare(64).expect("adoption prepares");
    let WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) =
        prepared.publish(host.runtime(), &principal, &scope)
    else {
        panic!("single-branch adoption must publish");
    };
    assert_eq!((performed.source(), performed.target()), (&p0, &target));
    performed
}

pub(super) fn verify_p1_and_close(
    home: Home,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    location: &Path,
    expected_commits: usize,
    expected_history_len: usize,
    expected_adoption_commit: ExpectedAdoptionCommit<'_>,
    label: &str,
) {
    let host = home.runtime();
    let branch = host.current_world();
    let p0 = *host.owned_revision();
    let p1 = *host.supported_program::<RetentionProgramP1>().unwrap().owned_revision();
    assert_eq!(read_retention(host.runtime(), branch), SEED_RETENTION, "{label}: seeded value before probes");
    assert_branch_revision(&home, branch, &p1, label);
    assert_eq!(
        host.runtime().world_history_snapshot_for_test().installed_commits(),
        expected_commits,
        "{label}: installed commit parity before probes"
    );
    let (_, adopted_commit) = assert_single_p0_to_p1_occurrence(
        &home, branch, Some(expected_history_len), expected_adoption_commit, label,
    );
    home.close(&execution).unwrap_or_else(|cause| panic!("{label}: pre-probe close: {cause:?}")).reclaim();

    // Reopen BEFORE overwriting to verify recovered data, revision and single occurrence survive cleanly.
    let durable_adopted = DurableCompositeCommitIdentity::from_recorded(
        adopted_commit.owner_identity().durable().expect("Home adoption commit is durable"),
        adopted_commit.ordinal(),
    );
    let verified = open_home(location, false, execution, None);
    let v_host = verified.runtime();
    let v_branch = v_host.current_world();
    assert_eq!(read_retention(v_host.runtime(), v_branch), SEED_RETENTION, "{label}: seeded value after reopen before probes");
    assert_branch_revision(&verified, v_branch, &p1, label);
    assert_single_p0_to_p1_occurrence(
        &verified, v_branch, Some(expected_history_len), ExpectedAdoptionCommit::Durable(durable_adopted), label,
    );
    assert!(v_host.retire_program_support(&execution, &p0).is_ok(), "{label}: P0 retires after adoption");
    assert_eq!(settle(set_retention(v_host, v_branch, 3, 0xA400)), RetentionVerdict::violated("document-retention-v2"), "{label}");
    assert_eq!(settle(set_retention(v_host, v_branch, 15, 0xA401)), RetentionVerdict::Performed(15), "{label}");
    verified.close(&execution).unwrap_or_else(|cause| panic!("{label}: post-probe close: {cause:?}")).reclaim();

    let reopened = open_home(location, false, execution, None);
    assert_eq!(read_retention(reopened.runtime().runtime(), reopened.runtime().current_world()), 15, "{label}");
    reopened.close(&execution).unwrap_or_else(|cause| panic!("{label}: final close: {cause:?}")).reclaim();
}

fn reopen_barrier_control(
    nth: NonZeroU64,
    markers: Option<&Path>,
    done: Option<&Arc<AtomicBool>>,
) -> (ApplicationHomeOpeningControlForTest, CertificationMediaFaultActivation, Option<std::thread::JoinHandle<()>>) {
    use worth_store_testkit_application_oracle::facade::host_interruption::opening::control;
    let media = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = media.fault_schedule_authority();
    let (activation, gate) = (authority.one_shot_activation(), authority.pause_gate());
    let schedule = authority.schedule(vec![authority
        .rule(MediaOperationRole::SynchronizeFileState, 1, MediaFaultDirective::PauseAfter(gate.clone()))
        .for_nth_identified_operation_after_activation(activation.clone(), nth)]).unwrap();
    let watcher = markers.zip(done).map(|(markers, done)| {
        let (done, ready) = (done.clone(), markers.join("ready"));
        std::thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                if let Some(ctx) = gate.reached_context() {
                    let note = format!(
                        "nth={nth} role={:?} path={:?} op={:?} id_ord={:?}",
                        ctx.role(), ctx.path_role(), ctx.operation(), ctx.identified_operation_ordinal()
                    );
                    crate::killed_child::announce(&ready, note.as_bytes());
                    crate::killed_child::park();
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        })
    });
    (control(|_| {}, Some(media.with_fault_schedule(schedule)), None), activation, watcher)
}

fn park_at_stage(location: &Path, markers: &Path, execution: worth_execution::ExecutionRequest<'_, '_>, name: &str) -> ! {
    let home = open_home(location, false, execution, None);
    let (ready, label) = (markers.join("ready"), name.to_owned());
    let wait_and_park = move |reached: bool| {
        assert!(reached, "stage {label} must pause");
        crate::killed_child::announce(&ready, format!("stage={label}").as_bytes());
        crate::killed_child::park();
    };
    if name == "before-product-compare" {
        let pause = home.runtime().runtime().world_operation_control_for_test().pause_before_product_compare_for_test(NonZeroUsize::MIN);
        std::thread::spawn(move || wait_and_park(pause.wait_until_reached(Duration::from_secs(60))));
    } else {
        let stage = match name {
            "before-retain" => AdoptionStage::BeforeRetainBranch,
            "after-retain" => AdoptionStage::AfterRetainBranch,
            "after-world-open" => AdoptionStage::AfterWorldOpen,
            "after-world-published" => AdoptionStage::AfterWorldPublished,
            "after-obligation-released" => AdoptionStage::AfterObligationReleased,
            other => panic!("unknown stage: {other}"),
        };
        let pause = home.runtime().runtime().pause_branch_adoption_stage_for_test(stage);
        std::thread::spawn(move || wait_and_park(pause.wait_until_reached(Duration::from_secs(60))));
    }
    let _ = adopt_branch_to_p1(&home);
    unreachable!("child must be killed while parked at stage {name}");
}

#[test]
fn adoption_seam_child() {
    let Some(location) = std::env::var_os(LOCATION) else { return; };
    let (location, markers, mode) = (
        std::path::PathBuf::from(location),
        std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap()),
        std::env::var(MODE).unwrap(),
    );
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    if let Some(stage) = mode.strip_prefix("stage:") {
        park_at_stage(&location, &markers, execution, stage);
    }
    if let Some(nth) = mode.strip_prefix("reopen-kill:") {
        let done = Arc::new(AtomicBool::new(false));
        let (control, activation, watcher) = reopen_barrier_control(
            NonZeroU64::new(nth.parse().unwrap()).unwrap(), Some(&markers), Some(&done),
        );
        activation.arm().expect("arm reopen barrier before Home open");
        let home = open_home(&location, false, execution, Some(control));
        done.store(true, Ordering::SeqCst);
        let _ = watcher.unwrap().join();
        home.close(&execution).unwrap().reclaim();
        crate::killed_child::announce(&markers.join("finished"), b"reopen finished");
        std::process::exit(0);
    }
    match mode.as_str() {
        "reopen-count" => {
            let (control, activation, _) = reopen_barrier_control(NonZeroU64::MAX, None, None);
            activation.arm().expect("arm reopen sync counter before open");
            let home = open_home(&location, false, execution, Some(control));
            let total = activation.matching_operation_count();
            home.close(&execution).unwrap().reclaim();
            crate::killed_child::announce(&markers.join("finished"), total.to_string().as_bytes());
            std::process::exit(0);
        }
        "stopped-branch-0" => stopped_and_retirement::child_stopped_prefix(&location, &markers, execution, false),
        "stopped-branch-1" => stopped_and_retirement::child_stopped_prefix(&location, &markers, execution, true),
        "retire-and-kill" => retirement::child_retire_and_kill(&location, &markers, execution),
        other => panic!("unknown adoption seam mode: {other}"),
    }
}
