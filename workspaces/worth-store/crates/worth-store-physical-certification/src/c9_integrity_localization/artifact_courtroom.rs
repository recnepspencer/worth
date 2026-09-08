use super::{
    artifact_inventory::ArtifactInventory,
    artifact_process::{ArtifactProcessRole, ArtifactRequest, REQUEST_ENV},
    process_execution::{fresh_identity, hex, run_offline_observer},
    process_manifest::ProcessTreeSnapshot,
    production_profile::ProductionWorldProfile,
    ClosedStoreProcessManifest,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::Path,
    process::{Child, Command},
    time::{Duration, Instant},
};

mod disagreement;

pub(super) fn run(
    observer: &Path,
    baseline: &Path,
    stores: &Path,
    reports: &Path,
    profile: ProductionWorldProfile,
) {
    run_selected(observer,baseline,stores,reports,profile,false);
}

pub(super) fn run_journals(observer:&Path,baseline:&Path,stores:&Path,reports:&Path) {
    run_selected(observer,baseline,stores,reports,ProductionWorldProfile::Primary16KiB,true);
}

fn run_selected(observer:&Path,baseline:&Path,stores:&Path,reports:&Path,profile:ProductionWorldProfile,journals_only:bool) {
    let inventory = ArtifactInventory::observe(baseline);
    let manifest = ClosedStoreProcessManifest::observe(baseline).unwrap();
    let frozen = stores.join(format!("{}-immutable-bytes", profile.label()));
    manifest.copy_to(baseline, &frozen).unwrap();
    let executable = std::env::current_exe().unwrap();
    let mut families = BTreeSet::new();
    let selected = inventory
        .granules
        .iter()
        .enumerate()
        .filter_map(|(index, granule)| {
            let applicable = if journals_only {
                granule.grammar != super::artifact_inventory::FrameGrammar::Common
            } else {
                profile == ProductionWorldProfile::Primary16KiB || granule.family == "inline_page"
            };
            (applicable && families.insert(granule.family)).then_some(index)
        })
        .collect::<Vec<_>>();
    let rows = std::iter::once((None, super::artifact_edit::ArtifactOperator::CoveredByte)).chain(
        selected.into_iter().flat_map(|index| {
            super::artifact_edit::operators(&inventory.granules[index])
                .into_iter()
                .map(move |operator| (Some(index), operator))
        }),
    );
    if profile == ProductionWorldProfile::Primary16KiB {
        for family in ["wal_frame", "checkpoint_stream_header", "checkpoint_dirty_basis",
            "checkpoint_binding_compaction", "checkpoint_binding", "checkpoint_footer"] {
            assert!(families.contains(family), "production matrix must select {family}");
        }
    }
    for (poison, operator) in rows {
        let family = poison.map_or("clean", |index| inventory.granules[index].family);
        let label = format!(
            "{}-{family}-{}",
            profile.label(),
            if poison.is_some() {
                operator.label()
            } else {
                "control"
            }
        );
        // The durability policy is bound to the real directory identity. Restore
        // each byte-isolated row into that existing root only after prior children exit.
        let row = baseline.to_owned();
        restore_bytes(&manifest, &frozen, &row);
        let scenario = fresh_identity(&label, stores);
        let run = fresh_identity(&format!("{label}-runtime"), reports);
        let runtime_report = reports.join(format!("{label}-runtime.json"));
        let request = ArtifactRequest {
            role: ArtifactProcessRole::RuntimeInspector,
            baseline: frozen.clone(),
            root: row.clone(),
            profile,
            ready: reports.join(format!("{label}.ready")),
            proceed: reports.join(format!("{label}.go")),
            report: runtime_report.clone(),
            scenario: hex(scenario),
            run: hex(run),
            poison,
            operator,
        };
        let mut runtime = launch(&executable, reports, &format!("{label}-runtime"), &request);
        wait_ready(&mut runtime, &request.ready);
        let before_editor = ProcessTreeSnapshot::observe_live_diagnostic(&row).unwrap();
        if let Some(index) = poison {
            let mut editor_request = request.clone();
            editor_request.role = ArtifactProcessRole::Editor;
            let mut editor = launch(
                &executable,
                reports,
                &format!("{label}-editor"),
                &editor_request,
            );
            wait(&mut editor);
            let target = &inventory.granules[index];
            let mut allowed = super::artifact_edit::enclosing_paths(&inventory, &frozen, index, operator);
            allowed.push(target.path.clone());
            let (before, after) = before_editor
                .require_only_files_delta(&row, &allowed, &target.path)
                .unwrap();
            super::artifact_edit::audit(&before, &after, target, operator);
            super::artifact_edit::audit_enclosures(&row, &frozen, &inventory, index, operator);
        }
        let unchanged = ProcessTreeSnapshot::observe_live_diagnostic(&row).unwrap();
        let offline_path = reports.join(format!("{label}-offline.json"));
        let offline = run_offline_observer(
            observer,
            &row,
            &offline_path,
            fresh_identity(&format!("{label}-offline"), reports),
            scenario,
        );
        unchanged.require_unchanged(&row).unwrap();
        super::artifact_process::create_marker(&request.proceed);
        wait_ready(&mut runtime, &request.report.with_extension("complete"));
        unchanged.require_unchanged(&row).unwrap();
        super::artifact_process::create_marker(&request.report.with_extension("release"));
        wait(&mut runtime);
        let runtime_wire: Value =
            serde_json::from_slice(&std::fs::read(&runtime_report).unwrap()).unwrap();
        assert_eq!(runtime_wire["role"], "runtime-integrity-observer");
        assert_eq!(runtime_wire["process"], runtime.child.id().to_string());
        assert_eq!(runtime_wire["store"], hex(manifest.store_identity()));
        assert_eq!(runtime_wire["scenario"], hex(scenario));
        assert_eq!(runtime_wire["run"], hex(run));
        assert_ne!(runtime_wire["process"], offline.report["process"]);
        assert_ne!(runtime_wire["executable"], offline.report["executable"]);
        if let Some(index) = poison {
            let target = &inventory.granules[index];
            for wire in [&runtime_wire, &offline.report] {
                let artifact = find(wire, target);
                super::artifact_expectation::require(artifact, target, operator, &label, wire["role"].as_str().unwrap());
                if operator==super::artifact_edit::ArtifactOperator::SelectiveAggregate {
                    let footer=inventory.granules.iter().find(|g|
                        g.path==target.path && g.family=="checkpoint_footer").unwrap();
                    super::artifact_expectation::require_aggregate(wire,target,footer,&label);
                }
            }
        } else {
            assert_eq!(runtime_wire["completeness"], "complete", "{runtime_wire}");
            for target in &inventory.granules {
                assert_eq!(
                    find(&runtime_wire, target)["outcome"]["posture"],
                    "intact",
                    "runtime {}",
                    target.family
                );
                assert_eq!(
                    find(&offline.report, target)["outcome"]["posture"],
                    "intact",
                    "offline {}",
                    target.family
                );
            }
        }
        compare(
            observer,
            &runtime_report,
            &offline_path,
            &reports.join(format!("{label}-comparison.json")),
        );
        if poison.is_none() && profile==ProductionWorldProfile::Primary16KiB && !journals_only {
            disagreement::require_actual_disagreement(observer,&executable,reports,&request,&inventory);
        }
        println!("C9 artifact courtroom {label} passed actual runtime/offline observations");
    }
    restore_bytes(&manifest, &frozen, baseline);
    manifest.require_unchanged(&frozen).unwrap();
}

fn restore_bytes(manifest: &ClosedStoreProcessManifest, frozen: &Path, root: &Path) {
    manifest.require_unchanged(frozen).unwrap();
    assert!(root.is_dir());
    for relative in manifest.paths() {
        // Deliberately never remove/recreate the Store root directory: its C4 media
        // identity participates in the persisted C7 durability-policy binding.
        std::fs::copy(frozen.join(relative), root.join(relative)).unwrap();
    }
    manifest.require_unchanged(root).unwrap();
}

struct ArtifactProcessChild {
    child: Child,
}
impl Drop for ArtifactProcessChild {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
fn launch(
    executable: &Path,
    reports: &Path,
    label: &str,
    request: &ArtifactRequest,
) -> ArtifactProcessChild {
    let request_path = reports.join(format!("{label}.artifact-request"));
    super::process_protocol::write_create_new(&request_path, request).unwrap();
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
fn wait(process: &mut ArtifactProcessChild) {
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
fn wait_ready(process: &mut ArtifactProcessChild, ready: &Path) {
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
fn find<'a>(wire: &'a Value, target: &super::artifact_inventory::ArtifactGranule) -> &'a Value {
    let path = target.path.to_string_lossy().replace('\\', "/");
    wire["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|artifact| {
            artifact["path"] == path && artifact["range"]["offset"] == target.offset() as u64
        })
        .unwrap_or_else(|| panic!("missing {}@{} in {}", path, target.offset(), wire))
}
fn compare(observer: &Path, runtime: &Path, offline: &Path, output: &Path) {
    let status = Command::new(observer)
        .args(["compare", "--runtime-observation"])
        .arg(runtime)
        .arg("--offline-observation")
        .arg(offline)
        .arg("--report")
        .arg(output)
        .status()
        .unwrap();
    assert!(status.success());
    let compared: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(compared["protocol"], "store.physical.integrity-comparison");
    // The offline inventory additionally names historical/unreachable paths; unmatched
    // requested scopes stay explicit disagreements, never edited out of either input.
}
