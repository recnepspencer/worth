use super::*;

fn fixture(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "tooling::process_deadline::tests::child_fixture",
            "--nocapture",
        ])
        .env("WORTH_HARNESS_DEADLINE_FIXTURE", mode);
    command
}

#[test]
fn child_fixture() {
    match std::env::var("WORTH_HARNESS_DEADLINE_FIXTURE").as_deref() {
        Ok("hang") => {
            println!("stdout retained before hang");
            eprintln!("stderr retained before hang");
            loop {
                std::thread::park();
            }
        }
        Ok("pipes") => {
            println!("{}", "o".repeat(512 * 1024));
            eprintln!("{}", "e".repeat(512 * 1024));
        }
        _ => {}
    }
}

#[test]
fn deadline_captures_output_and_reaps_a_hung_child() {
    let failure = std::panic::catch_unwind(|| {
        output_with_deadline(&mut fixture("hang"), Duration::from_secs(5)).unwrap()
    })
    .unwrap_err();
    let text = failure.downcast_ref::<String>().unwrap();
    for expected in [
        "child exceeded 5s",
        "elapsed=",
        "kill=Ok",
        "reap=Ok(Some(",
        "stdout retained before hang",
        "stderr retained before hang",
    ] {
        assert!(text.contains(expected), "{text}");
    }
}

#[test]
fn deadline_drains_both_pipes_before_waiting() {
    let output = output(&mut fixture("pipes")).unwrap();
    assert!(output.status.success());
    assert!(output.stdout.len() > 512 * 1024);
    assert!(output.stderr.len() > 512 * 1024);
}

#[test]
fn failed_kill_is_retained_in_the_expiry_report() {
    // Check the proposed natural failure case under the same bounded reap.
    let child = fixture("exited")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut running = RunningChild {
        child: Some(child),
        bound: DEFAULT_CHILD_PROCESS_DEADLINE,
    };
    let child = running.child.as_mut().unwrap();
    assert!(
        wait_for_exit(child, Instant::now() + DEFAULT_CHILD_PROCESS_DEADLINE)
            .unwrap()
            .is_some()
    );
    #[cfg(windows)]
    assert!(
        child.kill().is_ok(),
        "Windows treats a reaped child kill as success"
    );
    // An exited child cannot produce the kill error here. Supply an OS-shaped
    // error at the actual diagnostic boundary, without an unkillable process.
    let killed = Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "kill denied",
    ));
    let text = expiry_report(
        Duration::from_secs(300),
        Duration::from_secs(301),
        &killed,
        &Ok(None),
        b"before kill stdout",
        b"before kill stderr",
    );
    for expected in [
        "kill=Err",
        "kill denied",
        "elapsed=301s",
        "reap=Ok(None)",
        "before kill stdout",
        "before kill stderr",
    ] {
        assert!(text.contains(expected), "{text}");
    }
}
