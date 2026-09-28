use std::path::{Path, PathBuf};

pub(super) const RETIRED_CRATE_DIRECTORIES: [&str; 3] = [
    "workspaces/worth-ui/crates/worth-ui-host-egui",
    "workspaces/worth-ui/crates/worth-ui-theme",
    "workspaces/worth-ui/crates/worth-ui-components",
];

pub(super) const RETIRED_PRODUCT_FILES: [&str; 3] = [
    "workspaces/worth-ui/apps/platform-pulse/src/native_frame.rs",
    "workspaces/worth-ui/apps/platform-pulse/src/product_process/host_migration_grant.rs",
    "workspaces/worth-ui/crates/worth-ui-runtime/src/facade/entry/legacy_egui_application_transition.rs",
];

pub(super) const POSITIVE_COMPILE_TWIN: &str =
    "tests/ui/host/fixed_certification_headless_host_is_lawful.rs";
pub(super) const NEGATIVE_COMPILE_TWIN: &str = "tests/ui/host/retired_egui_surface_is_absent.rs";
const NEGATIVE_COMPILE_TWIN_REPOSITORY_PATH: &str = "workspaces/worth-ui/crates/worth-ui-certification/tests/ui/host/retired_egui_surface_is_absent.rs";
const NEGATIVE_COMPILE_TWIN_SNAPSHOT_REPOSITORY_PATH: &str = "workspaces/worth-ui/crates/worth-ui-certification/tests/ui/host/retired_egui_surface_is_absent.stderr";

pub(super) fn is_ignored_tree(relative: &Path) -> bool {
    relative.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some(".git" | "target" | "node_modules")
        )
    })
}

pub(super) fn is_exact_negative_fixture(relative: &Path) -> bool {
    let normalized = normalize(relative);
    normalized == NEGATIVE_COMPILE_TWIN_REPOSITORY_PATH
        || normalized == NEGATIVE_COMPILE_TWIN_SNAPSHOT_REPOSITORY_PATH
}

pub(super) fn is_historical_record(relative: &Path) -> bool {
    let normalized = normalize(relative);
    normalized.starts_with("plans/worth-ui/milestone-")
        || normalized == "plans/worth-ui/milestone-3.14.1-glyph-region-rebaseline.json"
}

pub(super) fn is_detector_source(relative: &Path) -> bool {
    let normalized = normalize(relative);
    normalized.starts_with(
        "workspaces/worth-ui/crates/worth-ui-certification/src/topology/host_platform/",
    ) || normalized
        == "workspaces/worth-ui/crates/worth-ui-certification/src/topology/dependency_audit.rs"
        || normalized == "workspaces/worth-ui/crates/worth-ui-certification/tests/topology_audit.rs"
}

pub(super) fn is_current_native_vision(relative: &Path) -> bool {
    normalize(relative) == "plans/worth-ui/worth-ui-vision.md"
}

pub(super) fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(super) fn compile_fixture_manifest(root: &Path) -> PathBuf {
    root.join(
        "workspaces/worth-ui/crates/worth-ui-certification/tests/fixtures/compile_contracts/Cargo.toml",
    )
}

pub(super) fn compile_case_inventory(root: &Path) -> PathBuf {
    root.join(
        "workspaces/worth-ui/crates/worth-ui-certification/tests/suites/compile_contract_cases.csv",
    )
}
