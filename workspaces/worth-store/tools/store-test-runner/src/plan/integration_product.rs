use std::{collections::BTreeSet, path::Path};

use super::offline_observer_build::offline_observer_build;
use super::TestExecutionUnit;

#[derive(Clone, Copy)]
struct IntegrationTarget {
    package: &'static str,
    target: &'static str,
    feature: Option<&'static str>,
}

pub(super) fn scenario(
    shard: Option<(usize, usize)>,
    workspace_root: &Path,
) -> Vec<TestExecutionUnit> {
    integration_units("scenario", SCENARIO_TARGETS, shard, workspace_root)
}

pub(super) fn ui(shard: Option<(usize, usize)>, workspace_root: &Path) -> Vec<TestExecutionUnit> {
    integration_units("ui", UI_TARGETS, shard, workspace_root)
}

pub(super) fn formal(
    shard: Option<(usize, usize)>,
    workspace_root: &Path,
) -> Vec<TestExecutionUnit> {
    integration_units("formal", FORMAL_TARGETS, shard, workspace_root)
}

fn integration_units(
    lane: &str,
    selections: &[IntegrationTarget],
    shard: Option<(usize, usize)>,
    workspace_root: &Path,
) -> Vec<TestExecutionUnit> {
    let tests = TestExecutionUnit::cargo(
        format!("{lane}::nextest"),
        workspace_root,
        integration_arguments(selections, shard),
    );
    if selections.iter().any(|selection| {
        selection.package == "worth-store" && selection.target == "physical_record_journeys"
    }) {
        vec![offline_observer_build(workspace_root), tests]
    } else {
        vec![tests]
    }
}

fn integration_arguments(
    selections: &[IntegrationTarget],
    shard: Option<(usize, usize)>,
) -> Vec<String> {
    let target_names = selections
        .iter()
        .map(|selection| selection.target)
        .collect::<BTreeSet<_>>();
    let features = selections
        .iter()
        .filter_map(|selection| selection.feature)
        .collect::<BTreeSet<_>>();
    let filter = selections
        .iter()
        .map(|selection| format!("binary_id(={}::{})", selection.package, selection.target))
        .collect::<Vec<_>>()
        .join(" + ");
    let mut arguments = vec![
        "nextest".into(),
        "run".into(),
        "--workspace".into(),
        "--no-fail-fast".into(),
        "--no-tests=fail".into(),
        "--filterset".into(),
        filter,
    ];
    for target in target_names {
        arguments.extend(["--test".into(), target.into()]);
    }
    for feature in features {
        arguments.extend(["--features".into(), feature.into()]);
    }
    if let Some((index, count)) = shard {
        arguments.extend([
            "--partition".into(),
            format!("hash:{}/{}", index + 1, count),
        ]);
    }
    arguments
}

const CERTIFICATION_FEATURE: Option<&str> = Some("worth-store/certification-test-authority");

const SCENARIO_TARGETS: &[IntegrationTarget] = &[
    target(
        "worth-store",
        "physical_media_journeys",
        CERTIFICATION_FEATURE,
    ),
    target(
        "worth-store",
        "physical_record_journeys",
        CERTIFICATION_FEATURE,
    ),
    target(
        "worth-store",
        "public_facade_downstream",
        CERTIFICATION_FEATURE,
    ),
    target("worth-store", "runtime_authority_pressure_journey", None),
    target("worth-store", "sealed_runtime_lifecycle_journey", None),
    target("worth-store-buffer-pool", "clean_authority", None),
    target("worth-store-physical-format", "manifest_access", None),
    target(
        "worth-store-physical-format",
        "physical_record_access",
        None,
    ),
    target(
        "worth-store-offline-verifier",
        "phase_eight_observer_cli",
        None,
    ),
    target("worth-store-physical-isolation", "layout_scenarios", None),
    target("worth-store-layout-indexes", "layout_scenarios", None),
    target("worth-store-branch-deltas", "layout_scenarios", None),
    target("worth-store-snapshots", "layout_scenarios", None),
    target("worth-store-recovery-runtime", "authority_compile", None),
    target(
        "worth-store-recovery-runtime",
        "phase_five_cancellation",
        None,
    ),
    target("worth-store-recovery-runtime", "phase_five_faults", None),
    target(
        "worth-store-recovery-runtime",
        "phase_five_generation_axes",
        None,
    ),
    target(
        "worth-store-recovery-runtime",
        "phase_five_prefix_completion",
        None,
    ),
    target("worth-store-recovery-runtime", "phase_five_staging", None),
    target("worth-store-recovery-runtime", "phase_four_planning", None),
    target("worth-store-recovery-runtime", "phase_four_process", None),
    target("worth-store-recovery-runtime", "phase_seven_cleanup", None),
    target("worth-store-recovery-runtime", "phase_seven_faults", None),
    target("worth-store-recovery-runtime", "phase_three_denials", None),
    target(
        "worth-store-recovery-runtime",
        "phase_three_discovery",
        None,
    ),
    target(
        "worth-store-recovery-runtime",
        "phase_three_manifest_denials",
        None,
    ),
    target(
        "worth-store-recovery-runtime",
        "phase_three_media_denials",
        None,
    ),
    target(
        "worth-store-recovery-runtime",
        "phase_three_root_anchor",
        None,
    ),
    target("worth-store-recovery-runtime", "production_entry", None),
    target(
        "worth-store-certification",
        "aspect_native_terminal_projection",
        None,
    ),
    target(
        "worth-store-certification",
        "aspect_native_terminal_projection_hostile_readmission",
        None,
    ),
    target("worth-store-certification", "blob_chunks", None),
    target("worth-store-certification", "io_scheduling", None),
    target("worth-store-certification", "layout_access", None),
    target("worth-store-certification", "operational_security", None),
    target("worth-store-certification", "physical_isolation", None),
    target(
        "worth-store-certification",
        "scheduler_queue_execution",
        None,
    ),
    target("store-test-runner", "cli", None),
];

const UI_TARGETS: &[IntegrationTarget] = &[
    target("worth-store", "physical_adapter_authority_ui", None),
    target(
        "worth-store",
        "physical_media_authority_ui",
        CERTIFICATION_FEATURE,
    ),
    target(
        "worth-store",
        "physical_runtime_authority_ui",
        CERTIFICATION_FEATURE,
    ),
    target(
        "worth-store-test-support",
        "harness_authority_compile_fail",
        None,
    ),
    target("worth-store-layout-indexes", "layout_compile_fail", None),
    target(
        "worth-store-physical-certification",
        "backend_qualification_public_boundary_compile_fail",
        None,
    ),
    target(
        "worth-store-physical-certification",
        "blob_harness_public_boundary_compile_fail",
        None,
    ),
    target(
        "worth-store-certification",
        "aspect_native_authority_ui",
        None,
    ),
    target(
        "worth-store-certification",
        "authority_projection_readmission_ui",
        None,
    ),
    target(
        "worth-store-certification",
        "s4_5_driver_contract_boundary_compile_fail",
        None,
    ),
    target(
        "worth-store-certification",
        "s5_1_security_scope_admission_compile_fail",
        None,
    ),
    target(
        "worth-store-certification",
        "s5_physical_isolation_entry_compile_fail",
        None,
    ),
    target(
        "worth-store-certification",
        "s5_tier_movement_future_chunk_compile_fail",
        None,
    ),
    target(
        "worth-store-certification",
        "s6_secure_io_authority_compile_fail",
        None,
    ),
    target(
        "worth-store-certification",
        "terminal_projection_quarantine_ui",
        None,
    ),
];

const FORMAL_TARGETS: &[IntegrationTarget] = &[
    target("worth-store-formal-models", "backend_assumptions", None),
    target(
        "worth-store-formal-models",
        "compaction_visibility_owner_execution",
        None,
    ),
];

const fn target(
    package: &'static str,
    target: &'static str,
    feature: Option<&'static str>,
) -> IntegrationTarget {
    IntegrationTarget {
        package,
        target,
        feature,
    }
}
