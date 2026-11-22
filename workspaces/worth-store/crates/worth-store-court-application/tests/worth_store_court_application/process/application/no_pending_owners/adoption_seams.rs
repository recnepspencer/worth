//! C3.4 / Phase 6: real process kills at the first durable barrier of each
//! adoption stage, stopped branch-set kills, and support retirement across kills.
mod retirement;
mod single_branch;
mod stopped_and_retirement;

use std::num::NonZeroU64;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
    },
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationProgramRoster, WorthQueryProgramOwner,
        WorthQueryProgramSupportRetirementFailure,
    },
    domain::WorthQueryProgramSupportRetirementDenial,
    product::{
        WorthQueryProductBranch, WorthQueryProductBranchCreateError,
        WorthQueryProductBranchCreationContinuation, WorthQueryProductBranchRecoveryDenial,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, seed_host},
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
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
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
    let ready = markers_dir.path().join("ready");
    let finished = markers_dir.path().join("finished");
    let end = crate::killed_child::run_and_kill(
        CHILD,
        &[
            (LOCATION, location),
            (MARKERS, markers_dir.path()),
            (MODE, mode_path.as_path()),
        ],
        &ready,
        &finished,
    );
    let detail = std::fs::read_to_string(&ready)
        .or_else(|_| std::fs::read_to_string(&finished))
        .unwrap_or_default();
    (end, detail)
}

fn activation_barrier_control(
    nth: NonZeroU64,
    markers: &Path,
    done: &Arc<AtomicBool>,
) -> (
    ApplicationHomeOpeningControlForTest,
    CertificationMediaFaultActivation,
    std::thread::JoinHandle<()>,
) {
    use worth_store_testkit_application_oracle::facade::host_interruption::opening::control;
    let media = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = media.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                MediaOperationRole::SynchronizeFileState,
                1,
                MediaFaultDirective::PauseAfter(gate.clone()),
            )
            .for_nth_identified_operation_after_activation(activation.clone(), nth)])
        .unwrap();
    let (done, ready) = (done.clone(), markers.join("ready"));
    let watcher = std::thread::spawn(move || {
        while !done.load(Ordering::SeqCst) {
            if let Some(ctx) = gate.reached_context() {
                let note = format!(
                    "nth={} role={:?} owner={:?} path={:?} op={:?} id_ord={:?} role_ord={}",
                    nth,
                    ctx.role(),
                    ctx.owner(),
                    ctx.path_role(),
                    ctx.operation(),
                    ctx.identified_operation_ordinal(),
                    ctx.role_ordinal()
                );
                crate::killed_child::announce(&ready, note.as_bytes());
                crate::killed_child::park();
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
    (
        control(|_| {}, Some(media.with_fault_schedule(schedule)), None),
        activation,
        watcher,
    )
}

fn counting_sync_control() -> (
    ApplicationHomeOpeningControlForTest,
    CertificationMediaFaultActivation,
) {
    use worth_store_testkit_application_oracle::facade::host_interruption::opening::control;
    let media = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = media.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                MediaOperationRole::SynchronizeFileState,
                1,
                MediaFaultDirective::PauseAfter(gate),
            )
            .for_nth_identified_operation_after_activation(activation.clone(), NonZeroU64::MAX)])
        .unwrap();
    (
        control(|_| {}, Some(media.with_fault_schedule(schedule)), None),
        activation,
    )
}

pub(super) fn adopt_branch_to_p1(home: &Home) {
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let target = *host.supported_program::<RetentionProgramP1>().unwrap().owned_revision();
    let programs = host.runtime().request(&principal, &scope).on_branch(host.current_world()).programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs.adopt(&requirements).expect("adoption admits").prepare(64).expect("adoption prepares");
    assert!(matches!(
        prepared.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}

pub(super) fn verify_p1_and_close(
    home: Home,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    location: &Path,
    expected_commits: usize,
    label: &str,
) {
    let host = home.runtime();
    assert_eq!(
        host.runtime().world_history_snapshot_for_test().installed_commits(),
        expected_commits,
        "{label}: branch adoption must apply exactly once"
    );
    let p0 = *host.owned_revision();
    assert!(host.retire_program_support(&execution, &p0).is_ok(), "{label}: P0 retires after adoption");
    let branch = host.current_world();
    assert_eq!(settle(set_retention(host, branch, 3, 0xA400)), RetentionVerdict::violated("document-retention-v2"), "{label}");
    assert_eq!(settle(set_retention(host, branch, 15, 0xA401)), RetentionVerdict::Performed(15), "{label}");
    home.close(&execution).unwrap_or_else(|cause| panic!("{label}: graceful close: {cause:?}")).reclaim();
    let reopened = open_home(location, false, execution, None);
    assert_eq!(read_retention(reopened.runtime().runtime(), reopened.runtime().current_world()), 15, "{label}");
    reopened.close(&execution).unwrap_or_else(|cause| panic!("{label}: final close: {cause:?}")).reclaim();
}

#[test]
fn adoption_seam_child() {
    let Some(location) = std::env::var_os(LOCATION) else {
        return;
    };
    let location = std::path::PathBuf::from(location);
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let mode = std::env::var(MODE).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    if let Some(nth) = mode.strip_prefix("single:") {
        let done = Arc::new(AtomicBool::new(false));
        let (control, activation, watcher) =
            activation_barrier_control(NonZeroU64::new(nth.parse().unwrap()).unwrap(), &markers, &done);
        let home = open_home(&location, false, execution, Some(control));
        activation.arm().expect("arm adoption barrier after Home open");
        adopt_branch_to_p1(&home);
        done.store(true, Ordering::SeqCst);
        let _ = watcher.join();
        crate::killed_child::announce(&markers.join("finished"), b"single adoption finished");
        std::process::exit(0);
    }
    if let Some(nth) = mode.strip_prefix("reopen-kill:") {
        let done = Arc::new(AtomicBool::new(false));
        let (control, activation, watcher) =
            activation_barrier_control(NonZeroU64::new(nth.parse().unwrap()).unwrap(), &markers, &done);
        activation.arm().expect("arm reopen barrier before Home open");
        let home = open_home(&location, false, execution, Some(control));
        done.store(true, Ordering::SeqCst);
        let _ = watcher.join();
        home.close(&execution).unwrap().reclaim();
        crate::killed_child::announce(&markers.join("finished"), b"reopen finished");
        std::process::exit(0);
    }
    match mode.as_str() {
        "single-uninterrupted" => {
            let (control, activation) = counting_sync_control();
            let home = open_home(&location, false, execution, Some(control));
            activation.arm().expect("arm adoption sync counter after open");
            adopt_branch_to_p1(&home);
            let total = activation.matching_operation_count();
            home.close(&execution).unwrap().reclaim();
            crate::killed_child::announce(&markers.join("finished"), total.to_string().as_bytes());
            std::process::exit(0);
        }
        "reopen-count" => {
            let (control, activation) = counting_sync_control();
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
