//! C4b.1: Real OS kills at durable I/O barriers (`SynchronizeFileState` and
//! `SynchronizeDirectoryPublication`) of an external-dispatching Host mutation
//! (`DispatchHomeSchema`), verifying cross-process zero-duplicate dispatch,
//! same-open post-continuation correlation lookup, and MemoryOnly parity.
use super::super::opening;
use super::{
    all_barriers, assert_barrier_killed, build_id, role_barrier_placement, RestorableHome,
    BUILD_PRIMARY,
};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        QueryDispatchRecordedObservation, WorthQueryExternalDispatchRequest,
        WorthQueryExternalEffectTransport, WorthQueryExternalTransportOutcome as TransportOutcome,
    },
    runtime::ExecutionAllocationPolicy,
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_host::facade::host::{
    application_runtime_surface as identity,
    dispatch_home::{
        self as dispatch, notify::NotifyAccountBalance, schema::DispatchHomeSchema,
        DispatchHomeProgram,
    },
};
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation,
};

type DispatchHome = WorthQueryApplicationHome<DispatchHomeSchema, DispatchHomeProgram>;

const CHILD: &str =
    "process::application::host_oracle::adversarial::dispatch_io_kills::dispatch_io_kill_child";
const ENV_HOME: &str = "WORTH_ADV_DISPATCH_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_DISPATCH_MARKERS";
const ENV_SPEC: &str = "WORTH_ADV_DISPATCH_SPEC";
const ENV_TRANSPORT_LOG: &str = "WORTH_ADV_DISPATCH_LOG";

const DISPATCH_KEY: u64 = 0xD150_0001;
const DISPATCH_BALANCE: u64 = 777;

struct DurableTransportLog {
    log_path: Option<PathBuf>,
    local_calls: AtomicUsize,
}
impl DurableTransportLog {
    fn file_backed(path: PathBuf) -> Self { Self { log_path: Some(path), local_calls: AtomicUsize::new(0) } }
    fn memory_only() -> Self { Self { log_path: None, local_calls: AtomicUsize::new(0) } }
}
impl WorthQueryExternalEffectTransport for DurableTransportLog {
    fn dispatch(&self, _request: WorthQueryExternalDispatchRequest<'_>) -> TransportOutcome {
        self.local_calls.fetch_add(1, Ordering::SeqCst);
        if let Some(path) = &self.log_path {
            let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
            writeln!(f, "dispatch").unwrap();
            f.sync_data().unwrap();
        }
        TransportOutcome::Completed
    }
}

fn durable_dispatch_count(log_path: &Path) -> usize {
    match std::fs::read_to_string(log_path) {
        Ok(s) => s.lines().filter(|l| !l.is_empty()).count(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
        Err(e) => panic!("durable transport log read failed: {e}"),
    }
}

struct DispatchOpen {
    location: Location,
    seed: bool,
    handler_contacts: Arc<AtomicUsize>,
    placement: Option<AdversarialPlacement>,
    transport: Arc<DurableTransportLog>,
}

fn open_dispatch_home(cfg: DispatchOpen, execution: &ExecutionRequest<'_, '_>) -> DispatchHome {
    let seed = cfg.seed;
    let request = dispatch::request_at_location(cfg.location, build_id(BUILD_PRIMARY), cfg.handler_contacts, seed);
    let request = match cfg.placement {
        Some(p) => host_observation::opening::attach_placement(request, p).0,
        None => request,
    };
    let home = with_bootstrap_advancement_from_request(*execution, |p| request.open_in_advancement(&p))
        .unwrap()
        .unwrap_or_else(|e| panic!("dispatch Home open (seed={seed}): {e:?}"));
    home.runtime().runtime().install_external_effect_transport(cfg.transport).expect("transport installs once");
    home
}

fn scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token(), WorthQueryAdmissionClock::system())
}

fn notify(home: &DispatchHome, key: u64, balance: u64) -> Outcome<(), u64> {
    let req_scope = scope();
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &req_scope,
    );
    home.runtime()
        .request(&principal, &req_scope)
        .advanced()
        .mutate(NotifyAccountBalance { balance })
        .without_source()
        .idempotency(&key)
        .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

struct DispatchRef {
    balance: u64,
    payload: Vec<u8>,
}

fn build_dispatch_reference(execution: &ExecutionRequest<'_, '_>) -> DispatchRef {
    let transport = Arc::new(DurableTransportLog::memory_only());
    let home = open_dispatch_home(
        DispatchOpen {
            location: Location::MemoryOnly, seed: true, handler_contacts: Arc::new(AtomicUsize::new(0)),
            placement: None, transport: transport.clone(),
        },
        execution,
    );
    let Outcome::Committed { receipt, result } = notify(&home, DISPATCH_KEY, DISPATCH_BALANCE) else {
        panic!("MemoryOnly dispatch reference must commit");
    };
    let payload = receipt.dispatch_outbox().unwrap().payload().to_vec();
    assert_eq!(transport.local_calls.load(Ordering::SeqCst), 1);
    drop(receipt);
    drop(home.close(execution).err().unwrap().resume().unwrap());
    DispatchRef { balance: result, payload }
}

#[test]
fn external_dispatch_custody_io_boundary_kill_sweep() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_dispatch_reference(&execution);
    let saved = RestorableHome::prepare(|loc| {
        let home = open_dispatch_home(
            DispatchOpen {
                location: Location::Directory(loc.to_path_buf()), seed: true,
                handler_contacts: Arc::new(AtomicUsize::new(0)), placement: None,
                transport: Arc::new(DurableTransportLog::memory_only()),
            },
            &execution,
        );
        home.close(&execution).unwrap().reclaim();
    });
    let mut saw_custody = false;
    for (tag, role) in [
        ("sync", MediaOperationRole::SynchronizeFileState),
        ("dir_sync", MediaOperationRole::SynchronizeDirectoryPublication),
    ] {
        let total = measure_dispatch_role(&saved, tag);
        if total == 0 {
            assert_ne!(role, MediaOperationRole::SynchronizeFileState);
            continue;
        }
        assert_eq!(total, measure_dispatch_role(&saved, tag), "dispatch {tag} calibration must be deterministic");
        for ordinal in all_barriers(total) {
            saved.restore();
            let log_dir = tempfile::tempdir().unwrap();
            let log_path = log_dir.path().join("transport.log");
            kill_dispatch_at((saved.location(), &log_path), (tag, role, ordinal));
            saw_custody |= verify_dispatch_reopen((saved.location(), &log_path), &reference, (tag, ordinal), &execution);
        }
    }
    assert!(saw_custody, "dispatch barrier sweep must exercise UnpublishedCustody continuation");
}

fn measure_dispatch_role(saved: &RestorableHome, tag: &str) -> u64 {
    saved.restore();
    let markers = tempfile::tempdir().unwrap();
    let log_path = markers.path().join("count.log");
    let spec = PathBuf::from(format!("count:{tag}"));
    let env = [
        (ENV_HOME, saved.location()),
        (ENV_MARKERS, markers.path()),
        (ENV_SPEC, &spec),
        (ENV_TRANSPORT_LOG, &log_path),
    ];
    let end = crate::killed_child::run_and_kill(CHILD, &env, &markers.path().join("ready"), &markers.path().join("finished"));
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    std::fs::read_to_string(markers.path().join("finished")).unwrap().trim().parse().unwrap()
}

fn kill_dispatch_at(paths: (&Path, &Path), target: (&str, MediaOperationRole, u64)) {
    let (location, log_path) = paths;
    let (tag, role, ordinal) = target;
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("{tag}:{ordinal}"));
    let ready = markers.path().join("ready");
    let env = [(ENV_HOME, location), (ENV_MARKERS, markers.path()), (ENV_SPEC, &spec), (ENV_TRANSPORT_LOG, log_path)];
    let end = crate::killed_child::run_and_kill(CHILD, &env, &ready, &markers.path().join("finished"));
    assert_barrier_killed(end, &ready, role, ordinal, &format!("dispatch {tag}#{ordinal}"));
}

fn verify_dispatch_reopen(
    paths: (&Path, &Path),
    reference: &DispatchRef,
    label: (&str, u64),
    execution: &ExecutionRequest<'_, '_>,
) -> bool {
    let (location, log_path) = paths;
    let (tag, ordinal) = label;
    let handler_contacts = Arc::new(AtomicUsize::new(0));
    let transport = Arc::new(DurableTransportLog::file_backed(log_path.to_path_buf()));
    let home = open_dispatch_home(
        DispatchOpen {
            location: Location::Directory(location.to_path_buf()),
            seed: false,
            handler_contacts: handler_contacts.clone(),
            placement: None,
            transport: transport.clone(),
        },
        execution,
    );
    assert_eq!(handler_contacts.load(Ordering::SeqCst), 0, "{tag}#{ordinal}: no reopen handler");
    assert_eq!(transport.local_calls.load(Ordering::SeqCst), 0, "{tag}#{ordinal}: no eager dispatch");
    assert!(durable_dispatch_count(log_path) <= 1, "{tag}#{ordinal}: killed child <= 1 dispatch");
    let had_custody = settle_dispatch_custody(&home, &handler_contacts, label, execution);
    verify_dispatch_retry(&home, (&handler_contacts, log_path), reference, label);
    home.close(execution)
        .unwrap_or_else(|e| panic!("{tag}#{ordinal}: dispatch close: {e:?}"))
        .reclaim();
    had_custody
}

fn settle_dispatch_custody(
    home: &DispatchHome,
    handler_contacts: &AtomicUsize,
    label: (&str, u64),
    execution: &ExecutionRequest<'_, '_>,
) -> bool {
    let (tag, ordinal) = label;
    let handles: Vec<_> = home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .iter()
        .map(|row| row.handle().clone())
        .collect();
    if handles.is_empty() {
        return false;
    }
    assert_eq!(handles.len(), 1, "{tag}#{ordinal}: at most one unpublished original");
    let recovery = home.runtime().runtime().readmit_product_publication_recovery(&handles[0]).unwrap();
    loop {
        match recovery.continue_original_in_request(execution).unwrap() {
            WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
            WorldRecoveredContinuationOutcome::Published { .. } => break,
            other => panic!("{tag}#{ordinal}: dispatch custody continue: {other:?}"),
        }
    }
    assert_eq!(handler_contacts.load(Ordering::SeqCst), 0);
    true
}

fn verify_dispatch_retry(
    home: &DispatchHome,
    obs: (&AtomicUsize, &Path),
    reference: &DispatchRef,
    label: (&str, u64),
) {
    let (handler_contacts, log_path) = obs;
    let (tag, ordinal) = label;
    let outbox = match notify(home, DISPATCH_KEY, DISPATCH_BALANCE) {
        Outcome::Committed { receipt, result } => {
            assert_eq!(result, reference.balance);
            assert_eq!(handler_contacts.load(Ordering::SeqCst), 1, "{tag}#{ordinal}");
            assert_eq!(durable_dispatch_count(log_path), 1, "{tag}#{ordinal}: exact 1 send");
            let row = receipt.dispatch_outbox().unwrap().clone();
            let custody = home.runtime().runtime().durable_dispatch_custody(row.correlation().bytes()).unwrap();
            assert_eq!(
                custody.observation(),
                Some(QueryDispatchRecordedObservation::Completed),
                "{tag}#{ordinal}: durable observation Completed"
            );
            drop(receipt);
            assert!(matches!(
                notify(home, DISPATCH_KEY, DISPATCH_BALANCE),
                Outcome::AlreadyCommitted(_) | Outcome::RecoveredCommitted(_)
            ));
            assert_eq!(durable_dispatch_count(log_path), 1, "{tag}#{ordinal}: retry no 2nd send");
            row
        }
        Outcome::RecoveredCommitted(receipt) => {
            assert_eq!(handler_contacts.load(Ordering::SeqCst), 0, "{tag}#{ordinal}: zero replay");
            let row = receipt.dispatch_outbox().unwrap().readmitted_record().unwrap();
            let custody = home
                .runtime()
                .runtime()
                .durable_dispatch_custody(row.correlation().bytes());
            let observed_completed = custody.and_then(|c| c.observation())
                == Some(QueryDispatchRecordedObservation::Completed);
            let sends = durable_dispatch_count(log_path);
            if observed_completed {
                assert_eq!(sends, 1, "{tag}#{ordinal}: Completed custody requires exact 1 send");
            } else {
                assert!(sends <= 1, "{tag}#{ordinal}: total sends <= 1");
            }
            row
        }
        other => panic!("{tag}#{ordinal}: unexpected dispatch retry outcome: {other:?}"),
    };
    assert_eq!(outbox.payload(), reference.payload.as_slice(), "{tag}#{ordinal}: MemoryOnly payload");
    let req_scope = scope();
    let observed = home
        .runtime()
        .runtime()
        .with_application_advancement(&req_scope, |phase| {
            home.runtime()
                .runtime()
                .observe_committed_dispatch_outbox_by_correlation(&phase, outbox.correlation())
        })
        .unwrap()
        .unwrap_or_else(|e| panic!("{tag}#{ordinal}: same-open correlation lookup failed: {e:?}"));
    assert_eq!(observed.record().payload(), reference.payload.as_slice());
    // Follow-up mutation on the same Account proves recovered graph state accepts fresh mutations.
    let Outcome::Committed { result: follow_bal, receipt: follow_rec } =
        notify(home, DISPATCH_KEY + 1, DISPATCH_BALANCE + 1)
    else {
        panic!("{tag}#{ordinal}: post-recovery follow-up dispatch must commit");
    };
    assert_eq!(follow_bal, DISPATCH_BALANCE + 1);
    drop(follow_rec);
}

fn decode_role(tag: &str) -> MediaOperationRole {
    match tag {
        "sync" => MediaOperationRole::SynchronizeFileState,
        "dir_sync" => MediaOperationRole::SynchronizeDirectoryPublication,
        other => panic!("unknown role tag {other}"),
    }
}

#[test]
fn dispatch_io_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let spec = std::env::var(ENV_SPEC).unwrap();
    let log_path = PathBuf::from(std::env::var_os(ENV_TRANSPORT_LOG).unwrap());
    let mut parts = spec.split(':');
    let first = parts.next().unwrap();
    let second = parts.next().unwrap();
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) = if first == "count" {
        role_barrier_placement(decode_role(second), u64::MAX, None, None)
    } else {
        let nth: u64 = second.parse().unwrap();
        role_barrier_placement(decode_role(first), nth, Some(markers.join("ready")), Some(done.clone()))
    };
    let home = open_dispatch_home(
        DispatchOpen {
            location: Location::Directory(location),
            seed: false,
            handler_contacts: Arc::new(AtomicUsize::new(0)),
            placement: Some(placement),
            transport: Arc::new(DurableTransportLog::file_backed(log_path)),
        },
        &execution,
    );
    activation.arm().unwrap();
    let _ = notify(&home, DISPATCH_KEY, DISPATCH_BALANCE);
    done.store(true, Ordering::SeqCst);
    if let Some(handle) = watcher {
        let _ = handle.join();
        crate::killed_child::announce(&markers.join("finished"), b"missed");
        return;
    }
    let count = activation.matching_operation_count();
    home.close(&execution).unwrap().reclaim();
    crate::killed_child::announce(&markers.join("finished"), count.to_string().as_bytes());
}
