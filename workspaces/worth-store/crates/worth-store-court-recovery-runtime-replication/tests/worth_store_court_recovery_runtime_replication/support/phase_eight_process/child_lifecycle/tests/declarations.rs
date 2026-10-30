pub(crate) use std::io::Write;
pub(crate) use std::time::Duration;
pub(crate) use {std::process::Command, std::process::Stdio};
pub(crate) use {
    worth_store_testkit_recovery_runtime::child_lifecycle::output_capture::MAXIMUM_PIPE_BYTES,
    worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard,
};

pub(crate) const CHILD_ROLE: &str = "WORTH_C8_PIPE_CAPTURE_TEST";

pub(crate) fn child(mode: &str) -> ProcessChildGuard {
    worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard::new(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "process::recovery_runtime::phase_eight_process::child_lifecycle::tests::output_child",
                "--nocapture",
            ])
            .env(CHILD_ROLE, mode)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap(),
    )
}
