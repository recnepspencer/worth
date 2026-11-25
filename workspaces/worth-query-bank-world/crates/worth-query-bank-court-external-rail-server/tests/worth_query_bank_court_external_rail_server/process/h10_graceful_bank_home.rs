//! One combined graceful journey; every Bank action runs on an executable main thread.
use std::{path::Path, process::{Command, Stdio}};
use worth_query_bank_testkit_server::facade::estate_operations::external_effect_dispatch::rail_transport::{
    rail_binary_path, spawn_rail,
};

#[test]
fn graceful_reopen_in_fresh_process() {
    run_journey("h10-graceful", None);
}

#[test]
fn full_journey_kill_reopen_before_first_checkpoint() {
    run_journey("h10-kill-prechk", Some("prekill-precheckpoint"));
}

#[test]
fn full_journey_kill_reopen_after_first_checkpoint() {
    run_journey("h10-kill-postchk", Some("prekill-postcheckpoint"));
}

fn run_journey(prefix: &str, kill_stage: Option<&str>) {
    let recipe = format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let directory = std::env::temp_dir().join(&recipe);
    std::fs::create_dir(&directory).expect("unique journey directory");
    let handoff = directory.join("handoff-data.json");
    let home = directory.join("home");
    let rail = spawn_rail();
    let executable = rail_binary_path()
        .parent()
        .unwrap()
        .join("examples")
        .join(format!(
            "h10_graceful_bank_home{}",
            std::env::consts::EXE_SUFFIX
        ));
    assert!(
        executable.is_file(),
        "explicitly build the bank-courtroom journey example"
    );
    let addresses = [
        rail.local_addr().to_string(),
        rail.test_control_addr().to_string(),
    ];
    let first_pid = match kill_stage {
        None => run_stage(
            &executable,
            "preclose",
            &home,
            &recipe,
            addresses.clone(),
            &handoff,
        ),
        Some(stage) => run_until_ready_then_kill(
            &executable,
            stage,
            &home,
            &recipe,
            addresses.clone(),
            &handoff,
        ),
    };
    assert!(
        handoff
            .metadata()
            .expect("descriptive handoff exists before reopen")
            .len()
            > 0
    );
    let reopen_pid = run_stage(&executable, "reopen", &home, &recipe, addresses, &handoff);
    assert_ne!(
        first_pid, reopen_pid,
        "fresh process must not share predecessor memory"
    );
    assert_ne!(first_pid, rail.pid());
    assert_ne!(reopen_pid, rail.pid());
    // The independent rail survives both stages and closes through its existing owner.
    let posture = rail
        .close()
        .expect("gracefully close independent rail after final Home close");
    assert_eq!(
        (
            posture.reserved,
            posture.pending,
            posture.exhausted,
            posture.permanently_denied
        ),
        (0, 0, 0, 0),
        "the retained rail must report no unresolved sender custody"
    );
    std::fs::remove_dir_all(directory).expect("reclaimed Home and DATA protocol can be removed");
}

fn run_until_ready_then_kill(
    executable: &Path,
    stage: &str,
    home: &Path,
    recipe: &str,
    addresses: [String; 2],
    handoff: &Path,
) -> u32 {
    use std::{
        io::{BufRead as _, BufReader},
        time::{Duration, Instant},
    };
    let mut child = Command::new(executable)
        .arg(stage)
        .arg(home)
        .arg(recipe)
        .args(addresses)
        .arg(handoff)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("start prekill journey on actual executable main/default stack");
    let pid = child.id();
    let expected_ready = format!("H10_PREKILL_READY {stage} PID {pid}");
    let stdout = child.stdout.take().expect("piped prekill stdout");
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut lines = Vec::new();
        for line in BufReader::new(stdout).lines() {
            let line = line.expect("UTF-8 stdout line");
            let ready = line == expected_ready;
            lines.push(line);
            if ready {
                let _ = tx.send(true);
                break;
            }
        }
        lines
    });
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut signaled = false;
    while Instant::now() < deadline {
        if rx.try_recv().is_ok() {
            signaled = true;
            break;
        }
        if let Some(status) = child.try_wait().expect("poll prekill child") {
            let lines = reader.join().unwrap_or_default();
            panic!("{stage} exited before H10_PREKILL_READY: {status:?}, stdout={lines:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        signaled,
        "{stage} did not reach H10_PREKILL_READY within 180s"
    );
    assert!(
        handoff.is_file(),
        "{stage} must write descriptive handoff DATA before signaling kill readiness"
    );
    child.kill().expect("OS kill live Bank Home child");
    let status = child.wait().expect("reap killed Bank Home child");
    assert!(
        !status.success(),
        "{stage} must terminate via OS kill, not graceful exit: {status:?}"
    );
    let _ = reader.join();
    pid
}

fn run_stage(
    executable: &Path,
    stage: &str,
    home: &Path,
    recipe: &str,
    addresses: [String; 2],
    handoff: &Path,
) -> u32 {
    let child = Command::new(executable)
        .arg(stage)
        .arg(home)
        .arg(recipe)
        .args(addresses)
        .arg(handoff)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start journey on actual executable main/default stack");
    let pid = child.id();
    let output = child
        .wait_with_output()
        .expect("wait for graceful stage exit");
    let stdout = String::from_utf8(output.stdout).expect("stage output is UTF-8");
    assert!(
        output.status.success(),
        "{stage} failed: {stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout
            .lines()
            .any(|line| line == format!("H10_GRACEFUL_STAGE {stage} PID {pid}")),
        "stage must acknowledge its actual PID only after Home close: {stdout}"
    );
    pid
}
