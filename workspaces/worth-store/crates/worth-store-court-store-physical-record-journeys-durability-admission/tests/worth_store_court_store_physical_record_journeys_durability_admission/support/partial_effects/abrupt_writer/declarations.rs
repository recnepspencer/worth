pub(crate) use {std::io::BufRead, std::io::Write};
pub(crate) use {std::path::Path};
pub(crate) use {std::process::Stdio};
pub(crate) use {std::sync::mpsc};
pub(crate) use {std::time::Duration};

pub(crate) const CHILD_TEST: &str = "child_dispatch::journey_child_role";
pub(crate) const EVENT_PREFIX: &str = "C4_EVENT ";

pub(crate) fn run_abrupt_writer(root: &Path, case: &str) -> String {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env("WORTH_STORE_C4_CHILD_ROLE", "faulted-media-writer")
        .env("WORTH_STORE_C4_CHILD_ROOT", root)
        .env("WORTH_STORE_C4_FAULT_CASE", case)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .unwrap();
    let output = child.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(output).lines() {
            let line = line.unwrap();
            if let Some(event) = line.strip_prefix(EVENT_PREFIX) {
                sender.send(event.to_owned()).unwrap();
            }
        }
    });
    let boundary = receiver.recv_timeout(std::time::Duration::from_secs(30)).unwrap();
    assert!(boundary.starts_with("fault-boundary;"));
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());
    boundary
}

pub(crate) fn event(message: &str) {
    println!("{EVENT_PREFIX}{message}");
    std::io::stdout().flush().unwrap();
}