//! Child reexecution uses the public Host; the parent kills the ready writer.
use super::*;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::{Command, Stdio};

const MODE: &str = "WORTH_QUERY_ORIGINAL_RECEIPT_MODE";
const LOCATION: &str = "WORTH_QUERY_ORIGINAL_RECEIPT_LOCATION";
const EXPECTED: &str = "WORTH_QUERY_ORIGINAL_RECEIPT_EXPECTED";
const READY: &str = "ORIGINAL_RECEIPT_WRITER_READY";

pub(super) fn journey(after_checkpoint: bool) {
    let behavior = if after_checkpoint {
        "lost_response_kill_reopen_retry_preserves_receipt_after_checkpoint"
    } else {
        "lost_response_kill_reopen_retry_preserves_receipt_before_checkpoint"
    };
    drive(behavior, || writer(after_checkpoint), reader);
}

pub(super) fn drive(behavior: &str, writer: impl FnOnce(), reader: impl FnOnce()) {
    match std::env::var(MODE).as_deref() {
        Ok("writer") => writer(),
        Ok("reader") => reader(),
        Ok(mode) => panic!("unknown original receipt child mode {mode}"),
        Err(_) => kill_and_read(behavior),
    }
}
pub(super) fn location() -> std::path::PathBuf {
    std::env::var_os(LOCATION).unwrap().into()
}
pub(super) fn wait_for_kill() {
    println!("{READY}");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    panic!("writer must be killed; it cannot return or close normally");
}

fn child(location: &Path, expected: &Path, mode: &str, behavior: &str) -> Command {
    let parent = module_path!().rsplit_once("::").unwrap().0;
    let test = format!("{}::{behavior}", parent.split_once("::").unwrap().1);
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", &test, "--nocapture"])
        .env(MODE, mode)
        .env(LOCATION, location)
        .env(EXPECTED, expected);
    command
}

fn kill_and_read(behavior: &str) {
    let location = fixture::location();
    let expected = location.with_extension("receipt-expectation");
    let mut writer = child(&location, &expected, "writer", behavior)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("writer child starts");
    let stdout = writer.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let observation = std::thread::spawn(move || {
        let mut output = String::new();
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.unwrap();
            output.push_str(&line);
            output.push('\n');
            if line.split_whitespace().last() == Some(READY) {
                let _ = sender.send(Ok(()));
                return;
            }
        }
        let _ = sender.send(Err(output));
    });
    let ready = receiver.recv_timeout(Duration::from_secs(60));
    // Always reap the child, including timeout and premature-exit failures.
    let killed = writer.kill();
    let status = writer.wait().unwrap();
    observation.join().unwrap();
    assert!(
        matches!(&ready, Ok(Ok(()))),
        "writer did not acknowledge: {ready:?}"
    );
    killed.expect("kill the writer without closing the Home");
    assert!(!status.success());
    let reader = child(&location, &expected, "reader", behavior)
        .output()
        .expect("fresh reader process starts");
    assert!(
        reader.status.success(),
        "reader for {behavior}: {}\n{}",
        String::from_utf8_lossy(&reader.stdout),
        String::from_utf8_lossy(&reader.stderr)
    );
    assert!(String::from_utf8_lossy(&reader.stdout).contains("1 passed; 0 failed"));
}

fn writer(after_checkpoint: bool) {
    let location = std::env::var_os(LOCATION).unwrap().into();
    let contacts = Arc::new(AtomicUsize::new(0));
    let initialized = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(location, build(), initialized.clone(), contacts.clone())
        .with_checkpoint_limits(
            worth_query_host::facade::WorthQueryApplicationHomeCheckpointLimits::new(
                std::num::NonZeroU64::new(u64::MAX).unwrap(),
                std::num::NonZeroU64::new(u64::MAX).unwrap(),
            ),
        );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("receipt writer Home", &refused));
    let (_, live) = scope();
    let receipt = match set(home.runtime(), &live, 0x7265747279, 8).unwrap() {
        Outcome::Committed { receipt, result: 8 } => receipt,
        other => panic!("original key must durably publish: {other:?}"),
    };
    let recorded = match set(home.runtime(), &live, 0x7265747279, 8).unwrap() {
        Outcome::RecoveredCommitted(recorded) => recorded,
        other => panic!("the original durable source answers before response loss: {other:?}"),
    };
    receipt_fields::assert_live_agrees(&receipt, &recorded);
    std::fs::write(
        std::env::var_os(EXPECTED).unwrap(),
        receipt_fields::projection(&recorded),
    )
    .unwrap();
    drop(recorded);
    drop(receipt);
    assert!(matches!(
        set(home.runtime(), &live, 0x726574727a, 9).unwrap(),
        Outcome::Committed { result: 9, .. }
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    if after_checkpoint {
        let checkpoint = home
            .checkpoint(&execution)
            .expect("actual Complete Home checkpoint");
        assert!(checkpoint.covered().0 > 0);
    }
    wait_for_kill();
}

fn reader() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request_with_initializer(
        std::env::var_os(LOCATION).unwrap().into(),
        build(),
        contacts.clone(),
        |_, _, _| panic!("reopened receipt Home cannot initialize"),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("receipt reader Home", &refused));
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), 9);
    let expected = std::fs::read_to_string(std::env::var_os(EXPECTED).unwrap()).unwrap();
    assert_original(home.runtime(), &live, &expected);
    assert!(matches!(
        set(home.runtime(), &live, 0x7265747279, 10).unwrap(),
        Outcome::IdempotencyIntentDrift
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert!(matches!(
        set(home.runtime(), &live, 0x726574727b, 11).unwrap(),
        Outcome::Committed { result: 11, .. }
    ));
    assert_original(home.runtime(), &live, &expected);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 11);
    drop(home);
}

fn assert_original(
    runtime: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>,
    live: &WorthQueryRequestScope,
    expected: &str,
) {
    match set(runtime, live, 0x7265747279, 8).unwrap() {
        Outcome::RecoveredCommitted(receipt) => {
            assert_eq!(receipt_fields::projection(&receipt), expected);
            let original = receipt
                .new_product_head()
                .durable()
                .unwrap()
                .selected_commit();
            let principal = identity::inspection_authentication::authenticate(
                runtime.runtime().installed_schema(),
                live,
            );
            let historical = runtime
                .runtime()
                .with_application_advancement(live, |phase| {
                    runtime
                        .request(&principal, live)
                        .at_recorded_commit(&phase, original, None)
                        .unwrap()
                        .query(fixture::balance_query::AccountRead(
                            "account-001".to_owned(),
                        ))
                        .execute_in_advancement(&phase)
                        .unwrap()
                })
                .unwrap();
            assert_eq!(
                historical.rows(),
                &[8],
                "original data selects its actual historical state after later mutations"
            );
        }
        other => {
            panic!("retry must return the exact original receipt without a handler: {other:?}")
        }
    }
}

fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap()
}
