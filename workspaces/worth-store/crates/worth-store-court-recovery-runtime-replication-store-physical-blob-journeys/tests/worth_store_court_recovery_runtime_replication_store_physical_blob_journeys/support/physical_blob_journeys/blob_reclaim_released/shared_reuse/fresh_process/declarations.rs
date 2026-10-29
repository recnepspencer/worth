//! A genuinely separate process also denies unsealed Serving construction.

pub(crate) use {std::path::Path, std::process::Command};

pub(crate) use {crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::denied_reopen};

pub(crate) const CHILD_TEST: &str = "blob_reclaim_released::shared_reuse::released_reuse_child";
pub(crate) const ROOT_ENV: &str = "WORTH_C11_RELEASE_CHILD_ROOT";

pub(crate) fn assert_open_denied(root: &Path) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROOT_ENV, root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh-process unsealed open did not deny correctly: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let child_pid = stdout
        .lines()
        .find_map(|line| line.split_once("WORTH_RELEASE_CHILD_PID "))
        .map(|(_, value)| {
            value
                .split_whitespace()
                .next()
                .unwrap()
                .parse::<u32>()
                .unwrap()
        })
        .expect("fresh unsealed-open worker executed");
    assert_ne!(child_pid, std::process::id());
}

pub(crate) fn run_child() {
    let Ok(root) = std::env::var(ROOT_ENV) else {
        return;
    };
    crate::physical_blob_journeys::blob_reclaim_released::shared_reuse::denied_reopen::assert_open_requires_c8_custody(std::path::Path::new(&root));
    println!("WORTH_RELEASE_CHILD_PID {}", std::process::id());
}