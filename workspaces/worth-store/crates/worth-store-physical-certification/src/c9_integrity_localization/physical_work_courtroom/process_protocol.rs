use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::super::process_execution::hex;
use super::super::process_protocol::{executable_sha256, write_create_new};
use super::super::ExternalReportPaths;
use super::corruption::Operator;

pub(super) const REQUEST_ENV: &str = "WORTH_C9_PHYSICAL_WORK_REQUEST";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) enum Role {
    PendingProducer,
    Editor(Operator),
    RuntimeObserver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Request {
    pub version: u16,
    pub role: Role,
    pub root: PathBuf,
    pub report: PathBuf,
    pub scenario: [u8; 32],
    pub run: [u8; 32],
}

pub(super) fn spawn(request: &Request) -> Child {
    ExternalReportPaths::authorize(&request.root, request.report.clone()).unwrap();
    let path = request.report.with_extension("request");
    write_create_new(&path, request).unwrap();
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c9_integrity_localization::c9_root_process_subject",
            "--nocapture",
        ])
        .env(REQUEST_ENV, path)
        .spawn()
        .expect("launch independent PW subject")
}

pub(super) fn execute(request: &Request) -> Value {
    let mut child = spawn(request);
    let pid = child.id();
    assert_ne!(pid, std::process::id());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "PW subject failed: {status}");
            break;
        }
        if std::time::Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("PW subject exceeded its process deadline");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let report = read_report(&request.report);
    assert_eq!(report["process_id"], pid);
    assert_eq!(report["scenario"], hex(request.scenario));
    assert_eq!(report["run"], hex(request.run));
    assert_eq!(report["protocol_version"], 1);
    assert_eq!(
        report["role"],
        match request.role {
            Role::PendingProducer => "pending-obligation-producer",
            Role::Editor(_) => "physical-work-artifact-editor",
            Role::RuntimeObserver => "physical-work-runtime-observer",
        }
    );
    assert_eq!(
        report["executable_sha256"],
        hex(executable_sha256(&std::env::current_exe().unwrap()).unwrap())
    );
    report
}

pub(super) fn read_report(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap();
    assert!(bytes.len() <= 1024 * 1024);
    serde_json::from_slice(&bytes).unwrap()
}

pub(super) fn emit(request: &Request, payload: Value) {
    use std::io::Write;
    let mut report = json!({
        "protocol_version": 1,
        "process_id": std::process::id(),
        "executable_sha256": hex(executable_sha256(&std::env::current_exe().unwrap()).unwrap()),
        "scenario": hex(request.scenario),
        "run": hex(request.run),
        "role": match request.role {
            Role::PendingProducer => "pending-obligation-producer",
            Role::Editor(_) => "physical-work-artifact-editor",
            Role::RuntimeObserver => "physical-work-runtime-observer",
        },
    });
    report
        .as_object_mut()
        .unwrap()
        .extend(payload.as_object().unwrap().clone());
    let bytes = serde_json::to_vec_pretty(&report).unwrap();
    assert!(bytes.len() <= 1024 * 1024);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&request.report)
        .unwrap();
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(request.report.with_extension("ready"))
        .unwrap()
        .sync_all()
        .unwrap();
}
