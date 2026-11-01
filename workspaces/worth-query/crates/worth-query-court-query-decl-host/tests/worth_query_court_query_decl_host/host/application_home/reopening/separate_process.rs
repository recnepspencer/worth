//! Children share only a physical path; the parent compares observed public facts.
use super::*;
const MODE: &str = "WORTH_QUERY_DIRECTORY_REOPEN_PROOF_MODE";
const PATH: &str = "WORTH_QUERY_DIRECTORY_REOPEN_PROOF_PATH";

#[test]
fn separate_processes_readmit_old_keys_and_restore_the_actual_outcome_frontier() {
    match std::env::var(MODE).as_deref() {
        Ok("writer") => writer(),
        Ok("reader") => reader(),
        Ok(mode) => panic!("unknown proof process {mode}"),
        Err(_) => {
            let path = fixture::location();
            let writer = child(&path, "writer");
            let reader = child(&path, "reader");
            assert_eq!(
                observed(&writer, "HOME_WRITER_COMMIT="),
                observed(&reader, "HOME_READER_OLD_COMMIT=")
            );
            assert!(
                observed(&reader, "HOME_READER_NEW_OUTCOME=")
                    > observed(&writer, "HOME_WRITER_OUTCOME_MAX=")
            );
        }
    }
}

fn child(path: &std::path::Path, mode: &str) -> String {
    let test = format!(
        "{}::separate_processes_readmit_old_keys_and_restore_the_actual_outcome_frontier",
        module_path!().split_once("::").unwrap().1
    );
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test, "--nocapture"])
        .env(MODE, mode)
        .env(PATH, path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{mode}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = String::from_utf8(output.stdout).unwrap();
    assert!(output.contains("1 passed; 0 failed"));
    output
}
fn observed(output: &str, marker: &str) -> u64 {
    let values: Vec<_> = output
        .lines()
        .filter_map(|line| {
            line.split_once(marker)
                .map(|(_, value)| value.trim().parse::<u64>().unwrap())
        })
        .collect();
    let [value] = values.as_slice() else {
        panic!("one actual observation {marker}: {output}")
    };
    *value
}
fn writer() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let initialized = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(
        std::env::var_os(PATH).unwrap().into(),
        ApplicationBuildIdentity::parse("00000000-0000-4000-8000-000000000001").unwrap(),
        initialized.clone(),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("writer Home: {refused:?}"));
    let (_, live) = scope();
    let mut maximum = 0;
    let mut last_commit = None;
    for ordinal in 0..8 {
        match set(home.runtime(), &live, 0x8800 + ordinal, 8).unwrap() {
            WorthQueryApplicationMutationOutcome::Committed { receipt, result: 8 } => {
                maximum = maximum.max(receipt.outcome_identity().unwrap().get());
                last_commit = Some(receipt.commit_reference().commit_id);
            }
            other => panic!("writer must publish: {other:?}"),
        }
    }
    assert_eq!(balance(home.runtime(), &live), 8);
    assert_eq!(contacts.load(Ordering::SeqCst), 8);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("writer close: {refused:?}"));
    println!("HOME_WRITER_COMMIT={}", last_commit.unwrap().0);
    println!("HOME_WRITER_OUTCOME_MAX={maximum}");
}
fn reader() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request_with_initializer(
        std::env::var_os(PATH).unwrap().into(),
        ApplicationBuildIdentity::parse("00000000-0000-4000-8000-000000000002").unwrap(),
        contacts.clone(),
        |_, _, _| panic!("new process must not initialize"),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("reader Home", &refused));
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), 8);
    match set(home.runtime(), &live, 0x8807, 8).unwrap() {
        WorthQueryApplicationMutationOutcome::PreviouslyCommitted(observed) => {
            println!("HOME_READER_OLD_COMMIT={}", observed.commit_id().0)
        }
        other => panic!("old key must preserve only its historical fact: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(balance(home.runtime(), &live), 8);
    match set(home.runtime(), &live, 0x9000, 9).unwrap() {
        WorthQueryApplicationMutationOutcome::Committed { receipt, result: 9 } => println!(
            "HOME_READER_NEW_OUTCOME={}",
            receipt.outcome_identity().unwrap().get()
        ),
        other => panic!("new key must publish: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 9);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("reader close: {refused:?}"));
}
