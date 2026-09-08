use std::path::Path;
use std::time::{Duration, Instant};

use super::super::process_execution::{fresh_identity, hex, run_offline_observer, run_subject};
use super::super::process_manifest::ProcessTreeSnapshot;
use super::super::process_protocol::{
    executable_sha256, ProcessReportPayload, ProcessSubjectRequest,
};
use super::super::ClosedStoreProcessManifest;
use super::artifact_manifest::PendingArtifact;
use super::corruption::Operator;
use super::process_protocol::{execute, read_report, spawn, Request, Role};

pub(in crate::c9_integrity_localization) fn run(observer_executable: &Path) {
    let world = tempfile::tempdir().unwrap().keep();
    println!("C9 pending-obligation retained world={}", world.display());
    let reports = world.join("reports");
    std::fs::create_dir(&reports).unwrap();
    let baseline = world.join("closed-baseline");
    let scenario = fresh_identity("PW-process-matrix", &world);
    let parent_executable = std::env::current_exe().unwrap();
    assert_ne!(
        executable_sha256(observer_executable).unwrap(),
        executable_sha256(&parent_executable).unwrap()
    );
    let produced = run_subject(
        &parent_executable,
        &reports,
        "clean-producer",
        ProcessSubjectRequest::producer(
            scenario,
            fresh_identity("clean-producer", &world),
            baseline.clone(),
            reports.join("clean-producer.report"),
        ),
    );
    let ProcessReportPayload::Produced(manifest) = produced.report.payload() else {
        panic!("production baseline report")
    };
    produced
        .report
        .require(
            super::super::RootWireRole::Producer,
            scenario,
            fresh_identity("clean-producer", &world),
            manifest.store_identity(),
            produced.process_id,
            executable_sha256(&parent_executable).unwrap(),
        )
        .unwrap();
    // Persisted durability admission binds the real root directory identity.
    // Reopen that same directory; a copied root is intentionally a different
    // policy basis and cannot stand in for an ordinary production reopen.
    let pending = baseline.clone();
    let request = Request {
        version: 1,
        role: Role::PendingProducer,
        root: pending.clone(),
        report: reports.join("pending-producer.json"),
        scenario,
        run: fresh_identity("pending-producer", &world),
    };
    let mut producer_process = PendingProducerProcess {
        child: spawn(&request),
    };
    let child = &mut producer_process.child;
    let pid = child.id();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if request.report.with_extension("ready").is_file() {
            break;
        }
        if let Some(status) = child.try_wait().unwrap() {
            panic!("pending producer exited before gate: {status}");
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("pending producer failed to reach durable journal/target boundary");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let producer = read_report(&request.report);
    assert_eq!(producer["process_id"], pid);
    assert_eq!(producer["protocol_version"], 1);
    assert_eq!(producer["role"], "pending-obligation-producer");
    assert_eq!(
        producer["executable_sha256"],
        hex(executable_sha256(&parent_executable).unwrap())
    );
    assert_eq!(producer["scenario"], hex(scenario));
    assert_eq!(producer["run"], hex(request.run));
    assert_eq!(producer["matching_target_operations"], 1);
    assert_eq!(
        producer["boundary"],
        "before-identified-target-positioned-write-after-journal-sync"
    );
    let artifact = PendingArtifact::observe(&pending);
    assert_eq!(artifact.store, manifest.store_identity());
    assert_eq!(
        serde_json::to_value(&artifact).unwrap(),
        producer["pending"]
    );
    assert!(child.try_wait().unwrap().is_none());
    child.kill().unwrap();
    assert!(
        !child.wait().unwrap().success(),
        "producer must die without close/settlement"
    );
    artifact.require_unchanged(&pending);
    let pending_manifest = ClosedStoreProcessManifest::observe(&pending).unwrap();
    let pending_reference = world.join("pending-reference");
    pending_manifest
        .copy_to(&pending, &pending_reference)
        .unwrap();

    for operator in Operator::ALL {
        let root = pending.clone();
        pending_manifest.require_unchanged(&root).unwrap();
        let editor = Request {
            version: 1,
            role: Role::Editor(operator),
            root: root.clone(),
            report: reports.join(format!("{}.editor.json", operator.label())),
            scenario,
            run: fresh_identity(&format!("{}-editor", operator.label()), &world),
        };
        let edited = execute(&editor);
        assert_eq!(edited["operator"], operator.label());
        super::corruption::audit(
            &pending_reference,
            &root,
            &pending_manifest,
            &artifact,
            operator,
        );
        let snapshot = ProcessTreeSnapshot::observe(&root).unwrap();
        let lease_path = root.join("namespace/mutation.lock");
        let lease_before = std::fs::read(&lease_path).unwrap();
        let offline_run = fresh_identity(&format!("{}-offline", operator.label()), &world);
        let offline = run_offline_observer(
            observer_executable,
            &root,
            &reports.join(format!("{}.offline.json", operator.label())),
            offline_run,
            scenario,
        );
        assert_eq!(offline.report["store"], hex(artifact.store));
        assert_eq!(offline.report["role"], "offline-root-observer");
        assert_eq!(offline.report["version"], 1);
        assert_eq!(offline.report["completeness"], "complete");
        assert_eq!(
            offline.executable_sha256,
            executable_sha256(observer_executable).unwrap()
        );
        assert_eq!(offline.report["run"], hex(offline_run));
        assert_eq!(offline.report["scenario"], hex(scenario));
        assert_eq!(offline.report["process"], offline.process_id.to_string());
        snapshot
            .require_unchanged(&root)
            .expect("offline PW diagnosis is byte-unchanged");
        let runtime = execute(&Request {
            version: 1,
            role: Role::RuntimeObserver,
            root: root.clone(),
            report: reports.join(format!("{}.runtime.json", operator.label())),
            scenario,
            run: fresh_identity(&format!("{}-runtime", operator.label()), &world),
        });
        assert_eq!(runtime["store"], hex(artifact.store));
        let lease_after = std::fs::read(&lease_path).unwrap();
        // Ordinary Store admission updates this one preexisting OS lease
        // payload. It is explicitly not a C9 integrity family. Once that child
        // has exited, restore only this exact payload before the full-tree
        // assertion; any other changed byte, missing entry or new entry fails.
        restore_lease_metadata(&lease_path, &lease_before);
        snapshot
            .require_unchanged(&root)
            .expect("Store PW diagnosis changes only owner-lease metadata");
        super::expectations::require(&artifact, operator, &runtime, &offline.report);
        write_parent_audit(
            &reports.join(format!("{}.parent-audit.json", operator.label())),
            &lease_before,
            &lease_after,
            &artifact,
            operator,
        );
        super::corruption::restore(&root, &artifact, operator);
        pending_manifest.require_unchanged(&root).unwrap();
        println!("C9 physical-work {} passed", operator.label());
    }
    pending_manifest
        .require_unchanged(&pending_reference)
        .unwrap();
    pending_manifest.require_unchanged(&pending).unwrap();
}

fn restore_lease_metadata(path: &Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn write_parent_audit(
    path: &Path,
    before: &[u8],
    after: &[u8],
    artifact: &PendingArtifact,
    operator: Operator,
) {
    use sha2::{Digest, Sha256};
    let output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(
        output,
        &serde_json::json!({
            "operator": operator.label(),
            "producer_artifact": artifact,
            "ordinary_admission_metadata_path": "namespace/mutation.lock",
            "ordinary_admission_metadata_changed": before != after,
            "lease_before_sha256": hex(Sha256::digest(before)),
            "lease_after_sha256": hex(Sha256::digest(after)),
            "all_other_files_and_namespace_entries_unchanged": true,
            "offline_whole_store_byte_exact": true,
        }),
    )
    .unwrap();
}

struct PendingProducerProcess {
    child: std::process::Child,
}

impl Drop for PendingProducerProcess {
    fn drop(&mut self) {
        if self.child.try_wait().is_ok_and(|status| status.is_none()) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
