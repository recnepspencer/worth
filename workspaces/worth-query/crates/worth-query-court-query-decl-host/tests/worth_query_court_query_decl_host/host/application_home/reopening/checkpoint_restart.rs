//! Real Host images and tails survive OS termination of the original owner.
use super::*;
use worth_query_host::facade::WorthQueryApplicationHomeCheckpointLimits;
use worth_query_testkit_host::facade::host::process_deadline;

const CHILD_HOME: &str = "WORTH_COMPLETE_HOME_RESTART_PATH";
mod writer;

#[derive(Clone, Copy)]
enum Case {
    Image,
    ImageAndTail,
    InterruptedAck,
    BoundedTail,
}

#[test]
fn selected_complete_home_reopens_and_selects_another_complete_checkpoint() {
    restart(
        Case::Image,
        "selected_complete_home_reopens_and_selects_another_complete_checkpoint",
    );
}
#[test]
fn killed_home_reopens_normalized_image_plus_nonempty_tail() {
    restart(
        Case::ImageAndTail,
        "killed_home_reopens_normalized_image_plus_nonempty_tail",
    );
}
#[test]
fn interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection() {
    restart(
        Case::InterruptedAck,
        "interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection",
    );
}
#[test]
fn automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom() {
    restart(
        Case::BoundedTail,
        "automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom",
    );
}

fn restart(case: Case, test: &str) {
    if let Some(path) = std::env::var_os(CHILD_HOME) {
        writer::run(case, path.into());
    }
    let path = fixture::location();
    let ready = path.with_extension("complete-home-ready");
    let output = path.with_extension("complete-home-child.log");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let transcript = std::fs::File::create(&output).unwrap();
    let test = format!("{}::{test}", module_path!().split_once("::").unwrap().1);
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test, "--nocapture"])
        .env(CHILD_HOME, &path)
        .stdout(transcript.try_clone().unwrap())
        .stderr(transcript)
        .spawn()
        .unwrap();
    let bound = Duration::from_secs(120);
    let mut running = process_deadline::RunningChild {
        child: Some(child),
        bound,
    };
    let deadline = Instant::now() + bound;
    loop {
        let child = running.child.as_mut().unwrap();
        if ready.is_file() {
            child.kill().unwrap();
            let status = process_deadline::wait_for_exit(child, Instant::now() + bound)
                .unwrap()
                .expect("OS termination must complete");
            assert!(
                !status.success(),
                "the writer must be killed with its Home still owned"
            );
            running.child.take();
            break;
        }
        assert!(
            child.try_wait().unwrap().is_none(),
            "writer exited before its durable observation: {}",
            std::fs::read_to_string(&output).unwrap()
        );
        assert!(
            Instant::now() < deadline,
            "writer did not reach its durable observation: {}",
            std::fs::read_to_string(&output).unwrap()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let observed = std::fs::read_to_string(ready).unwrap();
    let observed: Vec<u64> = observed
        .split_whitespace()
        .map(|field| field.parse().unwrap())
        .collect();
    let [generation, covered, head, value, tail] = observed.as_slice() else {
        panic!("five actual writer facts required");
    };
    assert!(*covered > 0 && *head >= *covered);
    match case {
        Case::Image | Case::InterruptedAck => assert_eq!(head, covered),
        Case::ImageAndTail => assert!(head > covered && *tail > 0),
        Case::BoundedTail => assert!(head >= covered),
    }
    let contacts = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request_with_initializer(path, build, contacts.clone(), |_, _, _| {
        panic!("a selected existing Home cannot replay its initializer")
    });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("selected Complete restart", &refused));
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), *value);
    let inspection = home.inspect_checkpoint(&execution).unwrap();
    assert_eq!(inspection.journals[0].covered, *covered);
    assert_eq!(inspection.journals[0].head, *head);
    if matches!(case, Case::BoundedTail) {
        assert!(*tail <= 8);
        assert!(inspection.native_canonical_body_reads <= 8 + 32,
            "reopen journal-body reads must follow the bounded tail, with fixed headroom: {inspection:?}");
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert!(matches!(
        set(home.runtime(), &live, 0x52455354415254, *value + 1).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let next = home.checkpoint(&execution).unwrap();
    assert!(next.generation() > *generation);
    assert!(next.covered().0 > *head);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let settled = home.inspect_checkpoint(&execution).unwrap();
    assert!(settled.pending.is_none());
    assert!(settled
        .journals
        .iter()
        .all(|journal| journal.head == journal.covered));
    // This court proves restart/selection/ACK, not Query's separate close lane.
    drop(home);
}
