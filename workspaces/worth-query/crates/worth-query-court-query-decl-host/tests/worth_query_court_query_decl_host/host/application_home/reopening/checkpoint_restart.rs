//! Real Host images and tails survive OS termination of the original owner.
use super::*;
use worth_query_host::facade::WorthQueryApplicationHomeCheckpointLimits;
use worth_query_testkit_host::facade::host::process_deadline;

const CHILD_HOME: &str = "WORTH_COMPLETE_HOME_RESTART_PATH";
mod writer;

#[derive(Clone, Copy)]
enum Case {
    Image,
    JournalOnly,
    ImageAndTail,
    ColdRecapture,
    InterruptedAck,
    BoundedTail,
}

#[test]
fn journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing() {
    restart(
        Case::JournalOnly,
        "journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing",
    );
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
fn cold_image_plus_tail_recapture_reads_only_the_uncovered_original_frames() {
    restart(
        Case::ColdRecapture,
        "cold_image_plus_tail_recapture_reads_only_the_uncovered_original_frames",
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
    assert!(*head >= *covered);
    match case {
        Case::JournalOnly => {
            assert_eq!(*generation, 0);
            assert_eq!(*covered, 0);
            assert!(*head > 0 && *tail > 0);
        }
        Case::Image | Case::InterruptedAck => {
            assert!(*covered > 0);
            assert_eq!(head, covered);
        }
        Case::ImageAndTail | Case::ColdRecapture => {
            assert!(*covered > 0 && head > covered && *tail > 0)
        }
        Case::BoundedTail => assert!(*covered > 0 && head >= covered),
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
    .unwrap_or_else(|refused| {
        use worth_query_host::facade::WorthQueryApplicationHomeOpenRefused as Refused;
        if let Refused::ExistingNativeFinalization { .. } = &refused {
            opening_failed("restart latest-Native family readmission", &refused);
        }
        if let Refused::ExistingQueryAttachment { .. } = &refused {
            opening_failed("restart selected-World Query attachment", &refused);
        }
        opening_failed("selected Complete restart", &refused)
    });
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
    let before_capture = matches!(case, Case::ColdRecapture).then(|| {
        home.inspect_checkpoint(&execution)
            .unwrap()
            .native_canonical_body_reads
    });
    let next = home.checkpoint(&execution).unwrap();
    assert!(next.generation() > *generation);
    assert!(next.covered().0 > *head);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let settled = home.inspect_checkpoint(&execution).unwrap();
    if matches!(case, Case::ColdRecapture) {
        assert_eq!(
            settled.native_canonical_body_reads
                - before_capture.expect("cold recapture counter recorded"),
            head - covered,
            "each uncovered cold Journal frame is demanded once; covered image frames stay cold"
        );
    }
    assert!(settled.pending.is_none());
    assert!(settled
        .journals
        .iter()
        .all(|journal| journal.head == journal.covered));
    // This court proves restart/selection/ACK, not Query's separate close lane.
    drop(home);
}
