//! A real writer is killed with its resource request in flight. The fresh Home
//! cold-releases the request before service, keeps its original, discharges
//! it without contacting the replacement provider and closes gracefully.
use super::*;
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};
use worth_foundational::facade::{HomeIdentity, OwnedAsyncObligationIdentity};

const MODE: &str = "WORTH_RESOURCE_COLD_RELEASE_MODE";
const LOCATION: &str = "WORTH_RESOURCE_COLD_RELEASE_LOCATION";
const READY: &str = "RESOURCE_COLD_RELEASE_WRITER_READY";
const TEST: &str =
    "in_flight_resource_request_at_kill_is_cold_released_with_its_original_then_discharged";

#[test]
fn in_flight_resource_request_at_kill_is_cold_released_with_its_original_then_discharged() {
    match std::env::var(MODE).as_deref() {
        Ok("writer") => writer(),
        Ok("reader") => reader(),
        Ok(mode) => panic!("unknown resource child mode {mode}"),
        Err(_) => kill_writer_then_read(),
    }
}

fn location() -> std::path::PathBuf {
    std::env::var_os(LOCATION).unwrap().into()
}
/// The original identity, recorded by the writer for the reader process.
fn identity_file() -> std::path::PathBuf {
    location().with_extension("resource-identity")
}
fn execution_request() -> worth_execution::SerialRequest {
    let budget = worth_execution::SerialMemoryBudget::new(Some(256 << 20));
    let cancellation = worth_execution::CancellationSource::new();
    worth_execution::SerialRequest::from_memory(budget, cancellation.token(), None)
}
fn open(
    contacts: Arc<Contacts>,
    initialized: Arc<AtomicUsize>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Home {
    with_bootstrap_advancement_from_request(execution, |phase| {
        request(location(), contacts, initialized).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("resource Home: {cause:?}"))
}

fn child(mode: &str, location: &std::path::Path) -> Command {
    let module = module_path!().split_once("::").unwrap().1;
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", &format!("{module}::{TEST}"), "--nocapture"])
        .env(MODE, mode)
        .env(LOCATION, location);
    command
}
fn kill_writer_then_read() {
    let location = std::env::temp_dir().join(format!(
        "worth-resource-cold-release-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut writer = child("writer", &location)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("writer child starts");
    let stdout = writer.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let observation = std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            if line.unwrap().split_whitespace().last() == Some(READY) {
                let _ = sender.send(());
                return;
            }
        }
    });
    let ready = receiver.recv_timeout(std::time::Duration::from_secs(120));
    // Always reap the child, including timeout and premature exit.
    let killed = writer.kill();
    let status = writer.wait().unwrap();
    observation.join().unwrap();
    assert!(ready.is_ok(), "the writer did not reach its kill point");
    killed.expect("kill the writer without closing its Home");
    assert!(!status.success());
    let reader = child("reader", &location).output().expect("reader starts");
    assert!(
        reader.status.success(),
        "reader exited {}: {}\n{}",
        reader.status,
        String::from_utf8_lossy(&reader.stdout),
        String::from_utf8_lossy(&reader.stderr)
    );
    assert!(String::from_utf8_lossy(&reader.stdout).contains("1 passed; 0 failed"));
}

fn writer() {
    let host = execution_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let contacts = Arc::new(Contacts::default());
    let home = open(contacts.clone(), Arc::new(AtomicUsize::new(0)), execution);
    let scope = request_scope();
    let actor = authenticate(&home, 1, &scope);
    let application = home.runtime().runtime();
    let resources = application.application_resources(&actor, &scope, application.current_world());
    let handle = resources.start::<Resource>(41).unwrap_or_else(|stop| {
        let (_, cause) = stop.into_parts();
        panic!("real start: {cause:?}")
    });
    assert_eq!(contacts.resource.input(0), 41);
    let identity = handle
        .obligation_identity()
        .expect("actual acknowledged original resource identity");
    let mut recorded = identity.home().bytes().to_vec();
    recorded.extend_from_slice(&identity.acquire_position().get().to_le_bytes());
    std::fs::write(identity_file(), recorded).unwrap();
    println!("{READY}");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    panic!("the writer must be killed with its request in flight");
}

fn reader() {
    let recorded = std::fs::read(identity_file()).unwrap();
    let (home_bytes, position) = recorded.split_at(16);
    let original = OwnedAsyncObligationIdentity::from_recorded(
        HomeIdentity::from_observed_bytes(home_bytes.try_into().unwrap()).unwrap(),
        std::num::NonZeroU64::new(u64::from_le_bytes(position.try_into().unwrap())).unwrap(),
    );
    let host = execution_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    // A replayed initializer would see 1 and fail.
    let initialized = Arc::new(AtomicUsize::new(1));
    let fresh = Arc::new(Contacts::default());
    let home = open(fresh.clone(), initialized.clone(), execution);
    {
        let scope = request_scope();
        let actor = authenticate(&home, 1, &scope);
        let application = home.runtime().runtime();
        let resources =
            application.application_resources(&actor, &scope, application.current_world());
        let state = resources
            .retained_state_governed::<Resource, _, _, _>(
                &41,
                original,
                metadata_access(&home, &actor, &scope),
            )
            .unwrap();
        assert_eq!(state.identity, original, "the original identity survives");
        assert!(!state.released, "no live release is invented");
        assert_eq!(state.terminal, None);
        assert_eq!(
            state.cold_release,
            Some(WorthQueryResourceColdRelease::Abandoned),
            "the request ended with its incarnation"
        );
        assert_eq!(
            resources
                .with_retained_result_governed::<Resource, _, _, _, _>(
                    &41,
                    original,
                    metadata_access(&home, &actor, &scope),
                    |value| *value
                )
                .unwrap(),
            None,
            "no result is invented"
        );
        resources
            .discharge_retained::<Resource>(&41, original)
            .unwrap();
        let absent = resources
            .retained_state_governed::<Resource, _, _, _>(
                &41,
                original,
                metadata_access(&home, &actor, &scope),
            )
            .unwrap_err();
        assert_eq!(absent.kind(), WorthQueryResourceStopKind::Unavailable);
    }
    assert_eq!(
        fresh.resource.counts(),
        (0, 0, 0),
        "neither cold release nor discharge contacts the replacement provider"
    );
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("cold-released resource close: {cause:?}"))
        .reclaim();
    let home = open(fresh.clone(), initialized, execution);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("reopened resource Home close: {cause:?}"))
        .reclaim();
    assert_eq!(fresh.resource.counts(), (0, 0, 0));
}
