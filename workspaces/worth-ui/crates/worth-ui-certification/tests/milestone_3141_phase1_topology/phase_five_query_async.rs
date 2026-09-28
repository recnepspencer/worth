use super::{repository_document, workspace_source_inventory};

const QUERY_BINDING_ROOT: &str = "crates/worth-ui-query-binding/src/presentation_async";
const QUERY_BINDING_FILES: [&str; 7] = [
    "declaration.rs",
    "request_basis.rs",
    "runtime_bridge.rs",
    "semantic_invalidation.rs",
    "observation.rs",
    "retained_posture.rs",
    "terminal_projection.rs",
];

#[test]
fn phase_five_query_async_dependency_direction_is_explicit_and_enforced() {
    let gate_e_active = future_binding_file_count() > 0;
    let binding_manifest =
        repository_document("workspaces/worth-ui/crates/worth-ui-query-binding/Cargo.toml");
    for dependency in ["worth-query", "worth-runtime-bridge", "worth-signal"] {
        assert!(
            binding_manifest.contains(dependency),
            "the sole WUI Query audience edge omits {dependency}"
        );
    }

    for manifest in ["workspaces/worth-ui/crates/worth-ui-runtime/Cargo.toml"] {
        let document = repository_document(manifest);
        let governed = if gate_e_active {
            document.as_str()
        } else {
            document
                .split("[dev-dependencies]")
                .next()
                .unwrap_or(&document)
        };
        for forbidden in ["worth-query", "worth-signal"] {
            assert!(
                !governed.contains(forbidden),
                "{manifest} gives a governed target direct {forbidden} access"
            );
        }
    }

    let inventory = workspace_source_inventory();
    for root in ["crates/worth-ui-runtime/src"] {
        for source in inventory.rust_files_under(root) {
            let path = source.relative_path().to_string_lossy();
            if is_test_source(&path) {
                continue;
            }
            for forbidden in ["worth_query", "worth_signal"] {
                assert!(
                    !source.text().contains(forbidden),
                    "{} imports {forbidden} outside the Query audience edge",
                    source.relative_path().display()
                );
            }
        }
    }
    if !gate_e_active {
        let plan =
            repository_document("plans/worth-ui/milestone-3.14.1-phase-5-implementation-plan.md");
        assert!(plan.contains("direct test-only `worth-query-host` installation"));
        assert!(plan.contains("no direct"));
        assert!(plan.contains("runtime imports neither Signal nor Query"));
    }

    let native_manifest =
        repository_document("workspaces/worth-ui/crates/worth-ui-host-native/Cargo.toml");
    assert!(
        !native_manifest.contains("worth-query"),
        "host-native may own physical Signal but never Query"
    );
    for source in inventory.rust_files_under("crates/worth-ui-host-native/src") {
        assert!(
            !source.text().contains("worth_query"),
            "{} imports Query into the native physical owner",
            source.relative_path().display()
        );
    }
}

#[test]
fn phase_five_query_async_destination_owns_the_missing_substrate_extensions() {
    let specification = repository_document("plans/worth-ui/milestone-3.14.1-phase-5.md");
    let plan =
        repository_document("plans/worth-ui/milestone-3.14.1-phase-5-implementation-plan.md");
    for required in [
        "application/declaration/async_resource/request_identity.rs",
        "runtime/async_result_state.rs",
        "source/async_declaration/completion/",
        "effects-indeterminate completion class",
        "Query-owned `Unresolved` retained-result state",
    ] {
        assert!(
            specification.contains(required) || plan.contains(required),
            "Phase 5 leaves the required Query substrate owner implicit: {required}"
        );
    }
    assert!(plan.contains("current identity part"));
    assert!(plan.contains("string-only"));

    let current_identity = repository_document(
        "workspaces/worth-query/crates/worth-query/src/application/declaration/async_resource/request_identity.rs",
    );
    assert!(current_identity.contains("pub fn text("));
    let current_state = repository_document(
        "workspaces/worth-query/crates/worth-query/src/runtime/async_result_state.rs",
    );
    if !current_state.contains("Unresolved") {
        assert!(plan.contains("Add Query's retained `Unresolved` async result state"));
    }

    assert_complete_future_binding_or_absent();
}

#[test]
fn semantic_and_physical_signal_domains_have_no_shared_owner_or_ambient_runtime() {
    let query_binding =
        repository_document("workspaces/worth-ui/crates/worth-ui-query-binding/Cargo.toml");
    let host_native =
        repository_document("workspaces/worth-ui/crates/worth-ui-host-native/Cargo.toml");
    let runtime = repository_document("workspaces/worth-ui/crates/worth-ui-runtime/Cargo.toml");
    assert!(!query_binding.contains("worth-ui-host-native"));
    assert!(!host_native.contains("worth-query"));
    assert!(!host_native.contains("worth-runtime-bridge"));
    assert!(!runtime.contains("worth-signal"));

    let inventory = workspace_source_inventory();
    for source in inventory.rust_files_under("crates/worth-ui-query-binding/src/presentation_async")
    {
        for forbidden in [
            "UiNativePhysicalSignalOwner",
            "UiNativePhysicalSignalRequestToken",
            "physical_work_signal",
        ] {
            assert!(
                !source.text().contains(forbidden),
                "{} imports physical Signal vocabulary through {forbidden}",
                source.relative_path().display()
            );
        }
    }
    for source in
        inventory.rust_files_under("crates/worth-ui-host-native/src/native/physical_work_signal")
    {
        for forbidden in [
            "WorthQuery",
            "worth_query",
            "BridgeOwnedSignalRuntime",
            "presentation_async",
        ] {
            assert!(
                !source.text().contains(forbidden),
                "{} imports semantic Signal vocabulary through {forbidden}",
                source.relative_path().display()
            );
        }
    }
}

#[test]
fn terminal_projection_and_string_identity_cannot_enter_operational_text_flow() {
    let inventory = workspace_source_inventory();
    for root in [
        "crates/worth-ui-runtime/src/native_platform/text_presentation",
        "crates/worth-ui-host-native/src/native/text_atlas",
        "crates/worth-ui-host-native/src/native/presentation/text",
    ] {
        for source in inventory.rust_files_under(root) {
            let path = source.relative_path().to_string_lossy();
            if is_test_source(&path) {
                continue;
            }
            for forbidden in [
                "terminal_projection",
                "serde_json::Value",
                "WorthQueryAsyncRequestIdentityPart::text",
            ] {
                assert!(
                    !source.text().contains(forbidden),
                    "{} admits operational authority through {forbidden}",
                    source.relative_path().display()
                );
            }
        }
    }

    let fixture = repository_document(
        "workspaces/worth-ui/crates/worth-ui/tests/ui/facade/query_binding/fail/reporting_projection_cannot_enter_consumption.rs",
    );
    assert!(fixture.contains("reporting.into()"));
    let compile_manifest = repository_document(
        "workspaces/worth-ui/crates/worth-ui-certification/tests/fixtures/compile_contracts/Cargo.toml",
    );
    assert!(compile_manifest.contains("reporting_projection_cannot_enter_consumption.rs"));
}

fn assert_complete_future_binding_or_absent() {
    let paths = future_binding_paths();
    let present = future_binding_file_count();
    assert!(
        present == 0 || present == paths.len(),
        "presentation_async must appear as one complete named responsibility set"
    );
    if present == paths.len() {
        let inventory = workspace_source_inventory();
        let operational = inventory
            .source(format!("{QUERY_BINDING_ROOT}/runtime_bridge.rs"))
            .expect("complete presentation_async runtime bridge owner");
        assert!(!operational
            .text()
            .contains("WorthQueryAsyncRequestIdentityPart::text"));
        let terminal = inventory
            .source(format!("{QUERY_BINDING_ROOT}/terminal_projection.rs"))
            .expect("complete presentation_async terminal projection owner");
        assert!(!terminal.text().contains("pub fn prepare"));
        assert!(!terminal.text().contains("pub fn settle"));
    }
}

fn future_binding_file_count() -> usize {
    let inventory = workspace_source_inventory();
    future_binding_paths()
        .iter()
        .filter(|path| inventory.source(path).is_some())
        .count()
}

fn future_binding_paths() -> [String; QUERY_BINDING_FILES.len()] {
    QUERY_BINDING_FILES.map(|name| format!("{QUERY_BINDING_ROOT}/{name}"))
}

fn is_test_source(path: &str) -> bool {
    path.contains("/tests/")
        || path.contains("\\tests\\")
        || path.ends_with("_tests.rs")
        || path.ends_with("tests.rs")
        || path.ends_with("_test_support.rs")
}
