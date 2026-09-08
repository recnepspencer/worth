use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use worth_store_physical_backend::QualifiedRecoveryFilesystemMedia;

use super::super::process_execution::hex;
use super::super::process_protocol::{executable_sha256, write_create_new};
use super::super::ExternalReportPaths;
use super::corruption::Operator;

pub(super) const REQUEST_ENV: &str = "WORTH_C9_NAMESPACE_REQUEST";

#[derive(Clone, Serialize, Deserialize)]
pub(super) enum Role {
    Editor(Operator),
    C4Observer,
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Request {
    pub root: PathBuf,
    pub report: PathBuf,
    pub role: Role,
    pub scenario: [u8; 32],
    pub run: [u8; 32],
}

pub(super) fn execute(request: Request) -> Value {
    ExternalReportPaths::authorize(&request.root, request.report.clone()).unwrap();
    let path = request.report.with_extension("request");
    write_create_new(&path, &request).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c9_integrity_localization::c9_root_process_subject",
            "--nocapture",
        ])
        .env(REQUEST_ENV, path)
        .spawn()
        .unwrap();
    let pid = child.id();
    assert_ne!(pid, std::process::id());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if std::time::Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("namespace child exceeded deadline");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let bytes = std::fs::read(&request.report).unwrap();
    assert!(bytes.len() < 1024 * 1024);
    let report: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(report["process_id"], pid);
    assert_eq!(report["version"], 1);
    assert_eq!(report["scenario"], hex(request.scenario));
    assert_eq!(report["run"], hex(request.run));
    assert_eq!(report["role"], role_name(&request.role));
    assert_eq!(
        report["executable_sha256"],
        hex(executable_sha256(&std::env::current_exe().unwrap()).unwrap())
    );
    report
}

pub(super) fn run_subject(request: Request) {
    let payload = match request.role {
        Role::Editor(operator) => {
            super::corruption::edit(&request.root, operator);
            json!({"operator": operator.label()})
        }
        Role::C4Observer => observe_c4(&request.root),
    };
    let mut report = json!({"version": 1, "role": role_name(&request.role),
        "process_id": std::process::id(), "scenario": hex(request.scenario),
        "run": hex(request.run),
        "executable_sha256": hex(executable_sha256(&std::env::current_exe().unwrap()).unwrap())});
    report
        .as_object_mut()
        .unwrap()
        .extend(payload.as_object().unwrap().clone());
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&request.report)
        .unwrap();
    serde_json::to_writer_pretty(file, &report).unwrap();
}

fn role_name(role: &Role) -> &'static str {
    match role {
        Role::Editor(_) => "namespace-artifact-editor",
        Role::C4Observer => "c4-runtime-observer",
    }
}

fn observe_c4(root: &std::path::Path) -> Value {
    // Both existing-root classification and persisted identity admission belong
    // to C4. Preserve the real stopping stage; never call initializing admission
    // or reinterpret its denial through a new C9 namespace validator.
    let qualified = match QualifiedRecoveryFilesystemMedia::qualify_existing(root) {
        Ok(qualified) => qualified,
        Err(denial) => {
            return json!({"admission": "denied", "stage": "qualify_existing",
            "cause": format!("{denial:?}")})
        }
    };
    assert_eq!(qualified.recovery_effect_count(), 0);
    match qualified.admit_persisted_store() {
        Ok(media) => {
            assert_eq!(media.recovery_effect_count(), 0);
            json!({"admission": "admitted", "store": hex(media.store_identity().bytes()),
                "recovery_effects": media.recovery_effect_count()})
        }
        Err(denial) => json!({"admission": "denied", "stage": "admit_persisted_store",
            "cause": format!("{denial:?}")}),
    }
}
