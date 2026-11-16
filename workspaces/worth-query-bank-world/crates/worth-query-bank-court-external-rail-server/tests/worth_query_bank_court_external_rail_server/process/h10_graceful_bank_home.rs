//! One combined graceful journey; every Bank action runs on an executable main thread.
use std::{path::Path, process::{Command, Stdio}};
use worth_query_bank_testkit_server::facade::estate_operations::external_effect_dispatch::rail_transport::{
    rail_binary_path, spawn_rail,
};

#[test]
fn graceful_reopen_in_fresh_process() {
    let recipe = format!(
        "h10-graceful-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let directory = std::env::temp_dir().join(&recipe);
    std::fs::create_dir(&directory).expect("unique journey directory");
    let handoff = directory.join("graceful-data.json");
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
    let run = |stage: &str| {
        run_stage(
            &executable,
            stage,
            &home,
            &recipe,
            [
                rail.local_addr().to_string(),
                rail.test_control_addr().to_string(),
            ],
            &handoff,
        )
    };
    let preclose_pid = run("preclose");
    assert!(
        handoff
            .metadata()
            .expect("successful graceful handoff exists")
            .len()
            > 0
    );
    let reopen_pid = run("reopen");
    assert_ne!(
        preclose_pid, reopen_pid,
        "fresh process must not share preclose memory"
    );
    assert_ne!(preclose_pid, rail.pid());
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
