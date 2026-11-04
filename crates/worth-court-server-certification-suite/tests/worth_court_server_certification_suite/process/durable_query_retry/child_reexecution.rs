//! Actual Server process death never supplies receipt or retry authority.
use super::*;
use std::{
    io::{BufRead, Write},
    process::{Command, Stdio},
    time::Duration,
};
const MODE: &str = "WORTH_SERVER_RECEIPT_MODE";
const LOCATION: &str = "WORTH_SERVER_RECEIPT_HOME";
const EXPECTED: &str = "WORTH_SERVER_RECEIPT_EXPECTED";
const READY: &str = "SERVER_ORIGINAL_RECEIPT_READY";

#[test]
fn killed_server_reopens_home_and_returns_its_original_query_receipt() {
    let mode = std::env::var(MODE).ok();
    match mode.as_deref() {
        Some("writer") => writer(),
        Some("reader") => reader(),
        Some(other) => panic!("unknown Server receipt role {other}"),
        None => journey(),
    }
}
fn child(location: &std::path::Path, expected: &std::path::Path, mode: &str) -> Command {
    let test = format!(
        "{}::killed_server_reopens_home_and_returns_its_original_query_receipt",
        module_path!().split_once("::").unwrap().1
    );
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", &test, "--nocapture"])
        .env(MODE, mode)
        .env(LOCATION, location)
        .env(EXPECTED, expected);
    command
}
fn journey() {
    let location = worth_query_testkit_host::facade::host::application_home::location();
    let expected = location.with_extension("server-original-receipt");
    let mut writer = child(&location, &expected, "writer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let stdout = writer.stdout.take().unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    let observation = std::thread::spawn(move || {
        let mut output = String::new();
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.unwrap();
            output.push_str(&line);
            output.push('\n');
            if line.split_whitespace().last() == Some(READY) {
                let _ = send.send(Ok(()));
                return;
            }
        }
        let _ = send.send(Err(output));
    });
    let ready = receive.recv_timeout(Duration::from_secs(60));
    let killed = writer.kill();
    let status = writer.wait().unwrap();
    observation.join().unwrap();
    ready
        .expect("Server writer reaches actual durable completion")
        .unwrap_or_else(|output| panic!("writer exited before durability ({status}): {output}"));
    killed.expect("parent kills its acknowledged Server process");
    let result = child(&location, &expected, "reader").output().unwrap();
    assert!(
        result.status.success(),
        "fresh Server process: {}
{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
fn writer() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open_at(
        std::env::var_os(LOCATION).unwrap().into(),
        contacts.clone(),
        true,
    );
    let server = server(home.runtime());
    assert!(matches!(
        execute(&server, "lost-response", 8, "writer-request").unwrap(),
        Outcome::Committed { .. }
    ));
    let original = match execute(&server, "lost-response", 8, "writer-read").unwrap() {
        Outcome::RecoveredCommitted(receipt) => receipt,
        other => panic!("the actual Query owner reads its sealed original: {other:?}"),
    };
    std::fs::write(std::env::var_os(EXPECTED).unwrap(), projection(&original)).unwrap();
    assert!(matches!(
        execute(&server, "later-key", 9, "writer-later").unwrap(),
        Outcome::Committed { .. }
    ));
    assert_contacts(&contacts, 2);
    println!("{READY}");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    panic!("Server writer must be killed with its real Home and lease alive");
}
fn reader() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open_at(
        std::env::var_os(LOCATION).unwrap().into(),
        contacts.clone(),
        false,
    );
    let server = server(home.runtime());
    let original = match execute(&server, "lost-response", 8, "reader-request").unwrap() {
        Outcome::RecoveredCommitted(receipt) => receipt,
        other => panic!("fresh process must read the original Query receipt: {other:?}"),
    };
    assert_eq!(
        projection(&original),
        std::fs::read_to_string(std::env::var_os(EXPECTED).unwrap()).unwrap()
    );
    assert_contacts(&contacts, 0);
    assert_eq!(balance(home.runtime()), 9);
    assert!(matches!(
        execute(&server, "reader-key", 11, "reader-later").unwrap(),
        Outcome::Committed { .. }
    ));
    assert_contacts(&contacts, 1);
    assert_eq!(balance(home.runtime()), 11);
}
fn projection(
    receipt: &worth_query_host::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt,
) -> String {
    assert!(receipt.dispatch_outbox().is_none());
    assert!(receipt.aftermath_causality().is_none());
    serde_json::to_string_pretty(&serde_json::json!({
        "native": receipt.semantic_result().semantic(), "native_position": receipt.native_journal_position(), "native_digest": receipt.native_artifact_digest(),
        "before": receipt.previous_recorded_component(), "after": receipt.recorded_component(),
        "old_head": format!("{:?}", receipt.old_product_head()), "new_head": format!("{:?}", receipt.new_product_head()),
        "attempt": format!("{:?}", receipt.attempt_identity()), "world_position": receipt.world_journal_position(), "world_digest": receipt.world_artifact_digest(),
        "publisher": format!("{:?}", receipt.publishing_incarnation()), "outcome": receipt.outcome_identity().get(), "emitted": receipt.emitted_effect_count(),
        "key": receipt.idempotency_key_identity(), "intent": receipt.recorded_intent(), "affinity": receipt.recorded_affinity(), "settlement": receipt.exact_output_settlement().recorded_code(),
    })).unwrap()
}
