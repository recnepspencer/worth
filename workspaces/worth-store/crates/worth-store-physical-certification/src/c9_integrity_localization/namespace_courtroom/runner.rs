use super::super::process_execution::{fresh_identity, hex, run_offline_observer, run_subject};
use super::super::process_manifest::ProcessTreeSnapshot;
use super::super::process_protocol::{
    executable_sha256, ProcessReportPayload, ProcessSubjectRequest,
};
use super::corruption::{self, Operator, IDENTITY};
use super::process_roles::{execute, Request, Role};
use sha2::{Digest, Sha256};
use std::path::Path;

pub(in crate::c9_integrity_localization) fn run(observer: &Path) {
    let temporary = tempfile::tempdir().unwrap();
    let world = temporary.path();
    let root = world.join("store");
    let reports = world.join("reports");
    std::fs::create_dir(&reports).unwrap();
    let executable = std::env::current_exe().unwrap();
    assert_ne!(
        executable_sha256(observer).unwrap(),
        executable_sha256(&executable).unwrap()
    );
    let scenario = fresh_identity("namespace-matrix", &world);
    let producer_run = fresh_identity("namespace-producer", &world);
    let produced = run_subject(
        &executable,
        &reports,
        "producer",
        ProcessSubjectRequest::producer(
            scenario,
            producer_run,
            root.clone(),
            reports.join("producer.report"),
        ),
    );
    let ProcessReportPayload::Produced(manifest) = produced.report.payload() else {
        panic!("production world");
    };
    produced
        .report
        .require(
            super::super::RootWireRole::Producer,
            scenario,
            producer_run,
            manifest.store_identity(),
            produced.process_id,
            executable_sha256(&executable).unwrap(),
        )
        .unwrap();
    let pristine = std::fs::read(root.join(IDENTITY)).unwrap();
    assert_eq!(pristine.len(), 72);
    assert_eq!(&pristine[..8], b"WSTNSID\0");
    assert_eq!(&pristine[24..40], &manifest.store_identity());
    assert_eq!(&pristine[40..72], &Sha256::digest(&pristine[..40])[..]);
    let pristine_tree = ProcessTreeSnapshot::observe(&root).unwrap();

    for operator in Operator::ALL {
        manifest.require_unchanged(&root).unwrap();
        pristine_tree.require_unchanged(&root).unwrap();
        let request_for = |suffix: &str, role| Request {
            root: root.clone(),
            role,
            report: reports.join(format!("{}.{suffix}.json", operator.label())),
            scenario,
            run: fresh_identity(&format!("{}-{suffix}", operator.label()), &world),
        };
        let edited = execute(request_for("editor", Role::Editor(operator)));
        assert_eq!(edited["operator"], operator.label());
        corruption::audit(&root, &pristine, operator);
        let mutated = std::fs::read(root.join(IDENTITY)).ok();
        // Restore just the independently audited edit to prove all unrelated
        // Store bytes and namespace entries remained exactly the producer's.
        corruption::write(&root.join(IDENTITY), &pristine);
        manifest.require_unchanged(&root).unwrap();
        pristine_tree.require_unchanged(&root).unwrap();
        match &mutated {
            Some(bytes) => corruption::write(&root.join(IDENTITY), bytes),
            None => std::fs::remove_file(root.join(IDENTITY)).unwrap(),
        }
        let before = ProcessTreeSnapshot::observe(&root).unwrap();
        let offline_run = fresh_identity(&format!("{}-offline", operator.label()), &world);
        let offline = run_offline_observer(
            observer,
            &root,
            &reports.join(format!("{}.offline.json", operator.label())),
            offline_run,
            scenario,
        );
        assert_eq!(offline.report["role"], "offline-root-observer");
        assert_eq!(offline.report["version"], 1);
        assert_eq!(offline.report["process"], offline.process_id.to_string());
        assert_eq!(offline.report["scenario"], hex(scenario));
        assert_eq!(offline.report["run"], hex(offline_run));
        assert_eq!(
            offline.executable_sha256,
            executable_sha256(observer).unwrap()
        );
        before.require_unchanged(&root).unwrap();
        let lease = root.join("namespace/mutation.lock");
        let lease_before = std::fs::read(&lease).unwrap();
        let runtime = execute(request_for("runtime", Role::C4Observer));
        corruption::write(&lease, &lease_before);
        before
            .require_unchanged(&root)
            .expect("C4 diagnostic changes only the exact existing owner-lease payload");
        super::expectations::require(
            operator,
            &hex(manifest.store_identity()),
            &runtime,
            &offline.report,
        );
        corruption::write(&root.join(IDENTITY), &pristine);
        manifest.require_unchanged(&root).unwrap();
        pristine_tree.require_unchanged(&root).unwrap();
        println!("C9 namespace {} passed", operator.label());
    }
}
