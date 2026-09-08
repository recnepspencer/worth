use super::{
    artifact_inventory::ArtifactInventory,
    artifact_process::{ArtifactProcessRole, ArtifactRequest, REQUEST_ENV},
    process_execution::{fresh_identity, hex, run_offline_observer},
    process_manifest::ProcessTreeSnapshot,
    production_profile::ProductionWorldProfile,
    ClosedStoreProcessManifest,
};
use serde_json::Value;
use std::path::Path;

mod comparison;
mod disagreement;
use comparison::compare;
mod process_control;
mod selection;
use process_control::{launch, wait, wait_ready};

pub(super) fn run(
    observer: &Path,
    baseline: &Path,
    stores: &Path,
    reports: &Path,
    profile: ProductionWorldProfile,
    produced: &ClosedStoreProcessManifest,
) {
    run_selected(observer, baseline, stores, reports, profile, None, produced);
}

pub(super) fn run_journals(
    observer: &Path,
    baseline: &Path,
    stores: &Path,
    reports: &Path,
    produced: &ClosedStoreProcessManifest,
) {
    run_selected(
        observer,
        baseline,
        stores,
        reports,
        ProductionWorldProfile::Primary16KiB,
        Some("journals"),
        produced,
    );
}

pub(super) fn run_family(
    observer: &Path,
    baseline: &Path,
    stores: &Path,
    reports: &Path,
    family: &str,
    produced: &ClosedStoreProcessManifest,
) {
    run_selected(
        observer,
        baseline,
        stores,
        reports,
        ProductionWorldProfile::Primary16KiB,
        Some(family),
        produced,
    );
}

fn run_selected(
    observer: &Path,
    baseline: &Path,
    stores: &Path,
    reports: &Path,
    profile: ProductionWorldProfile,
    selection: Option<&str>,
    manifest: &ClosedStoreProcessManifest,
) {
    let inventory = ArtifactInventory::observe(baseline);
    manifest.require_unchanged(baseline).unwrap();
    let frozen = stores.join(format!("{}-immutable-bytes", profile.label()));
    manifest.copy_to(baseline, &frozen).unwrap();
    let executable = std::env::current_exe().unwrap();
    let rows = selection::rows(&inventory, profile, selection);
    let mut clean_recovery = None;
    let mut clean_request = None;
    for (poison, operator) in rows {
        let poison = poison.map(|index| {
            let target = &inventory.granules[index];
            if operator == super::artifact_edit::ArtifactOperator::Truncate
                && matches!(target.family, "inline_page" | "extent_chunk")
            {
                inventory
                    .granules
                    .iter()
                    .enumerate()
                    .filter(|(_, candidate)| candidate.path == target.path)
                    .max_by_key(|(_, candidate)| candidate.offset())
                    .unwrap()
                    .0
            } else {
                index
            }
        });
        let family = poison.map_or("clean", |index| inventory.granules[index].family);
        let family = if matches!(
            operator,
            super::artifact_edit::ArtifactOperator::Remove
                | super::artifact_edit::ArtifactOperator::Duplicate
        ) {
            match family {
                "inline_page" => "segment_container",
                "extent_chunk" => "extent_container",
                "wal_frame" => "wal_container",
                "checkpoint_stream_header" => "checkpoint_container",
                family => family,
            }
        } else {
            family
        };
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
            records: manifest.records.clone(),
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
        if poison.is_none() {
            clean_request = Some(request.clone());
        }
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
            if matches!(
                operator,
                super::artifact_edit::ArtifactOperator::Remove
                    | super::artifact_edit::ArtifactOperator::Duplicate
            ) {
                let duplicate = (operator == super::artifact_edit::ArtifactOperator::Duplicate)
                    .then(|| super::artifact_presence::duplicate_path(target, &frozen));
                before_editor.require_presence_delta(&row, &target.path, duplicate.as_deref());
            } else {
                let mut allowed =
                    super::artifact_edit::enclosing_paths(&inventory, &frozen, index, operator);
                allowed.push(target.path.clone());
                let (before, after) = before_editor
                    .require_only_files_delta(&row, &allowed, &target.path)
                    .unwrap();
                super::artifact_edit::audit(&before, &after, target, operator);
                super::artifact_edit::audit_enclosures(&row, &frozen, &inventory, index, operator);
            }
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
        if poison.is_none()
            && profile == ProductionWorldProfile::Primary16KiB
            && (selection.is_none() || selection == Some("inline_page"))
        {
            disagreement::require_actual_disagreement(observer, reports, &request, &inventory);
        }
        super::artifact_process::create_marker(&request.report.with_extension("release"));
        wait(&mut runtime);
        if poison.is_none()
            || (matches!(
                operator,
                super::artifact_edit::ArtifactOperator::CoveredByte
                    | super::artifact_edit::ArtifactOperator::ScopeSubstitution
            ) && poison.is_some_and(|index| {
                matches!(
                    inventory.granules[index].family,
                    "bootstrap_catalog"
                        | "current_root_selector"
                        | "previous_root_selector"
                        | "root_manifest"
                        | "free_space_header"
                )
            }))
        {
            let unchanged = ProcessTreeSnapshot::observe_live_diagnostic(&row).unwrap();
            let mut ordinary = request.clone();
            ordinary.role = ArtifactProcessRole::OrdinaryOpen;
            let mut child = launch(
                &executable,
                reports,
                &format!("{label}-ordinary-open"),
                &ordinary,
            );
            wait(&mut child);
            unchanged.require_unchanged(&row).unwrap();
        }
        if operator == super::artifact_edit::ArtifactOperator::CoveredByte
            || (operator == super::artifact_edit::ArtifactOperator::ScopeSubstitution
                && poison.is_some_and(|index| {
                    matches!(
                        inventory.granules[index].family,
                        "bootstrap_catalog"
                            | "current_root_selector"
                            | "previous_root_selector"
                            | "root_manifest"
                    )
                }))
        {
            let target = poison.map(|index| &inventory.granules[index]);
            let observed = super::artifact_recovery::observe(
                &row,
                reports,
                &label,
                manifest.store_identity(),
                target,
                profile,
            );
            if let Some(target) = target {
                super::artifact_recovery::require_consumption(
                    clean_recovery.as_ref().unwrap(),
                    &observed,
                    target,
                    operator,
                );
            } else {
                clean_recovery = Some(observed);
            }
        }
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
                if matches!(
                    operator,
                    super::artifact_edit::ArtifactOperator::Remove
                        | super::artifact_edit::ArtifactOperator::Duplicate
                ) {
                    for member in inventory
                        .granules
                        .iter()
                        .filter(|member| member.path == target.path)
                    {
                        super::artifact_presence::require(
                            wire,
                            member,
                            &frozen,
                            operator,
                            wire["role"] == "runtime-integrity-observer",
                        );
                    }
                    continue;
                }
                let artifact = find(wire, target);
                super::artifact_expectation::require_common_scope(
                    artifact,
                    target,
                    &frozen,
                    Some(operator),
                    wire["role"] == "runtime-integrity-observer",
                );
                super::artifact_expectation::require(
                    artifact,
                    target,
                    operator,
                    &label,
                    wire["role"].as_str().unwrap(),
                );
                if operator == super::artifact_edit::ArtifactOperator::SelectiveAggregate {
                    let footer = inventory
                        .granules
                        .iter()
                        .find(|g| g.path == target.path && g.family == "checkpoint_footer")
                        .unwrap();
                    super::artifact_expectation::require_aggregate(wire, target, footer, &label);
                }
            }
        } else {
            assert_eq!(runtime_wire["completeness"], "complete", "{runtime_wire}");
            for target in &inventory.granules {
                for (wire, runtime) in [(&runtime_wire, true), (&offline.report, false)] {
                    super::artifact_expectation::require_common_scope(
                        find(wire, target),
                        target,
                        &frozen,
                        None,
                        runtime,
                    );
                }
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
        if operator == super::artifact_edit::ArtifactOperator::Duplicate {
            let target = &inventory.granules[poison.unwrap()];
            std::fs::remove_file(
                row.join(super::artifact_presence::duplicate_path(target, &frozen)),
            )
            .unwrap();
        }
        println!("C9 artifact courtroom {label} passed actual runtime/offline observations");
    }
    restore_bytes(&manifest, &frozen, baseline);
    manifest.require_unchanged(&frozen).unwrap();
    if profile == ProductionWorldProfile::Primary16KiB
        && (selection.is_none() || selection == Some("free_space_membership_block"))
    {
        // All read-only/poison rows are finished. This paired positive control
        // may now consume the temporary live root; the immutable bytes remain.
        let mut request = clean_request.unwrap();
        request.role = ArtifactProcessRole::AllocationControl;
        let mut child = launch(&executable, reports, "clean-allocation", &request);
        wait(&mut child);
        manifest.require_unchanged(&frozen).unwrap();
    }
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
