pub(crate) use {worth_store::physical_runtime::MediaShutdownOutcome, std::path::Path, worth_proof::TransitionOutcome, crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission};

pub(crate) const CHILD_TEST: &str = "process_contender::child_media_contender";

pub(crate) fn run_contender(root: &Path, expectation: &str) {
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env("WORTH_STORE_C4_CHILD_ROOT", root)
        .env("WORTH_STORE_C4_CHILD_EXPECTATION", expectation)
        .status()
        .unwrap();
    assert!(status.success(), "child contender failed: {expectation}");
}

pub(crate) fn spawn_lease_holder(root: &Path) -> std::process::Child {
    use {std::io::BufRead};

    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env("WORTH_STORE_C4_CHILD_ROOT", root)
        .env("WORTH_STORE_C4_CHILD_EXPECTATION", "hold")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = std::io::BufReader::new(child.stdout.take().unwrap());
    loop {
        let mut line = String::new();
        assert_ne!(output.read_line(&mut line).unwrap(), 0);
        if line.trim() == "media-lease-ready" {
            break;
        }
    }
    std::thread::spawn(move || {
        let _ = std::io::copy(&mut output, &mut std::io::sink());
    });
    child
}