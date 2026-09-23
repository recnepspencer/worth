use super::decode::read_record;
use super::watch::{classify_revision, AdmittedRevisionRelation};
use super::{
    PlatformPulseExecutorGatePosture, PlatformPulseIntentInputEvent,
    PlatformPulseIntentInputInstallation, PlatformPulseIntentInputOperability,
    PlatformPulseIntentInputWatch, PlatformPulseIntentInputWatchDenial, CHANNEL_CAPACITY,
    INPUT_FILE,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const SUCCESSOR_DEADLINE: Duration = Duration::from_secs(5);
const SETTLE_WINDOW: Duration = Duration::from_millis(400);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

static NEXT_INSTALLATION: AtomicU64 = AtomicU64::new(1);

#[test]
fn intent_input_revision_admission_distinguishes_duplicate_stale_and_successor() {
    assert_eq!(classify_revision(7, 7), AdmittedRevisionRelation::Duplicate);
    assert_eq!(classify_revision(7, 6), AdmittedRevisionRelation::Stale);
    assert_eq!(classify_revision(7, 8), AdmittedRevisionRelation::Successor);
    assert_eq!(
        classify_revision(7, 70),
        AdmittedRevisionRelation::Successor
    );
}

#[test]
fn intent_samples_decode_to_distinct_typed_product_postures() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("intent_samples");
    let cases = [
        ("ready.json", PlatformPulseIntentInputOperability::Ready),
        (
            "confirmation-required.json",
            PlatformPulseIntentInputOperability::ConfirmationRequired,
        ),
        ("denied.json", PlatformPulseIntentInputOperability::Denied),
    ];
    for (file, posture) in cases {
        let record = read_record(&root.join(file)).expect("checked-in intent sample");
        assert_eq!(record.operability(), posture);
    }
}

#[test]
fn intent_input_rejects_unknown_fields_and_zero_revision() {
    let root = isolated_root();
    let target = root.join("platform-pulse-intent.json");
    std::fs::write(
        &target,
        br#"{"protocol":"worth-ui.platform-pulse.intent-source","schema_version":1,"revision":0,"operability":"ready","executor_gate":"held"}"#,
    )
    .expect("write invalid record");
    assert_eq!(
        read_record(&target),
        Err(PlatformPulseIntentInputWatchDenial::InvalidRevision)
    );
    std::fs::write(
        &target,
        br#"{"protocol":"worth-ui.platform-pulse.intent-source","schema_version":1,"revision":1,"operability":"ready","executor_gate":"held","forged":true}"#,
    )
    .expect("write hostile record");
    assert!(matches!(
        read_record(&target),
        Err(PlatformPulseIntentInputWatchDenial::Decode(_))
    ));
    std::fs::remove_dir_all(root).expect("remove intent input fixture");
}

#[test]
fn intent_installation_reads_once_before_starting_the_bounded_watch() {
    let root = isolated_root();
    std::fs::write(
        root.join("platform-pulse-intent.json"),
        include_bytes!("../../../intent_samples/ready.json"),
    )
    .expect("write initial intent input");
    let installation = PlatformPulseIntentInputInstallation::open(&root)
        .expect("open bounded intent input installation");
    let (initial, watch) = installation.into_parts();
    assert_eq!(initial.revision(), 1);
    assert_eq!(
        initial.executor_gate(),
        PlatformPulseExecutorGatePosture::Held
    );
    let shutdown = watch.shutdown().expect("shut down intent input watch");
    assert!(shutdown.worker_joined());
    assert_eq!(shutdown.pending_event_count(), 0);
    std::fs::remove_dir_all(root).expect("remove intent input fixture");
}

/// Falsifier for the ingress rule: the watcher must not observe its own reads, nor
/// sibling traffic in its root, as change. Before the shared predicate every
/// `std::fs::read` of the watched file re-entered the notify queue on inotify and
/// the worker stopped with `ChannelCapacityExceeded` during `query_application_launch`.
#[test]
fn intent_watch_survives_own_reads_and_sibling_traffic_then_admits_a_successor() {
    let root = isolated_root();
    let target = root.join(INPUT_FILE);
    std::fs::write(
        &target,
        include_bytes!("../../../intent_samples/ready.json"),
    )
    .expect("write initial intent input");
    let installation = PlatformPulseIntentInputInstallation::open(&root)
        .expect("open bounded intent input installation");
    let (initial, mut watch) = installation.into_parts();
    assert_eq!(initial.revision(), 1);
    for ordinal in 0..(CHANNEL_CAPACITY * 3) {
        std::fs::read(&target).expect("watched intent file stays readable");
        std::fs::write(root.join(format!("sibling-{ordinal}.json")), b"{}")
            .expect("write sibling traffic");
    }
    let settle_until = Instant::now() + SETTLE_WINDOW;
    while Instant::now() < settle_until {
        assert_no_event(&mut watch, "own reads and sibling traffic");
        std::thread::sleep(POLL_INTERVAL);
    }
    // Writers replace the record atomically (temporary + rename), as the worlds'
    // `atomic_replacement` does; a truncating in-place write is not the contract.
    let temporary = root.join("platform-pulse-intent.json.tmp");
    std::fs::write(
        &temporary,
        br#"{"protocol":"worth-ui.platform-pulse.intent-source","schema_version":1,"revision":2,"operability":"ready","executor_gate":"released"}"#,
    )
    .expect("write successor intent input");
    std::fs::rename(&temporary, &target).expect("replace intent input atomically");
    let successor = await_record(&mut watch);
    assert_eq!(successor.revision(), 2);
    assert_eq!(
        successor.executor_gate(),
        PlatformPulseExecutorGatePosture::Released
    );
    let shutdown = watch.shutdown().expect("shut down intent input watch");
    assert!(shutdown.worker_joined());
    assert_eq!(shutdown.pending_event_count(), 0);
    std::fs::remove_dir_all(root).expect("remove intent input fixture");
}

fn assert_no_event(watch: &mut PlatformPulseIntentInputWatch, phase: &str) {
    match watch.try_next() {
        None => {}
        Some(PlatformPulseIntentInputEvent::Record(record)) => {
            panic!(
                "{phase} must not surface a record, saw revision {}",
                record.revision()
            )
        }
        Some(PlatformPulseIntentInputEvent::Failed(denial)) => {
            panic!("{phase} must not stop the watch, saw {denial}")
        }
    }
}

fn await_record(
    watch: &mut PlatformPulseIntentInputWatch,
) -> super::PlatformPulseIntentInputRecord {
    let deadline = Instant::now() + SUCCESSOR_DEADLINE;
    while Instant::now() < deadline {
        match watch.try_next() {
            None => std::thread::sleep(POLL_INTERVAL),
            Some(PlatformPulseIntentInputEvent::Record(record)) => return record,
            Some(PlatformPulseIntentInputEvent::Failed(denial)) => {
                panic!("successor write must not stop the watch, saw {denial}")
            }
        }
    }
    panic!("successor intent record did not arrive within {SUCCESSOR_DEADLINE:?}")
}

fn isolated_root() -> std::path::PathBuf {
    let ordinal = NEXT_INSTALLATION.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "worth-ui-platform-pulse-intent-input-{}-{ordinal}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("create intent input fixture");
    root
}
