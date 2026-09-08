use super::{ArtifactRequest, REQUEST_ENV};
use std::{
    path::Path,
    process::{Child, Command},
    time::{Duration, Instant},
};

pub(super) struct ArtifactProcessChild {
    pub(super) child: Child,
}
impl Drop for ArtifactProcessChild {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
pub(super) fn launch(
    executable: &Path,
    reports: &Path,
    label: &str,
    request: &ArtifactRequest,
) -> ArtifactProcessChild {
    let request_path = reports.join(format!("{label}.artifact-request"));
    super::super::process_protocol::write_create_new(&request_path, request).unwrap();
    ArtifactProcessChild {
        child: Command::new(executable)
            .args([
                "--exact",
                "c9_integrity_localization::c9_root_process_subject",
                "--nocapture",
            ])
            .env(REQUEST_ENV, request_path)
            .spawn()
            .unwrap(),
    }
}
pub(super) fn wait(process: &mut ArtifactProcessChild) {
    let child = &mut process.child;
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "artifact process: {status}");
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            panic!("bounded artifact process deadline exceeded");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
pub(super) fn wait_ready(process: &mut ArtifactProcessChild, ready: &Path) {
    let child = &mut process.child;
    let deadline = Instant::now() + Duration::from_secs(120);
    while !ready.is_file() {
        assert!(
            child.try_wait().unwrap().is_none(),
            "inspector exited before ready"
        );
        if Instant::now() >= deadline {
            child.kill().unwrap();
            panic!("inspector ready deadline exceeded");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
