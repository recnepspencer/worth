use super::post_tool_use_fixture::*;
use std::fs;

#[test]
fn non_governed_and_non_edit_payloads_skip_the_constitution() {
    let command = configured_hook_command();
    for payload in [
        r#"{"tool_name":"Edit","tool_input":{"file_path":"README.md"}}"#,
        r#"{"tool_name":"Read","tool_input":{"file_path":"tools/boundary-check/src/main.rs"}}"#,
    ] {
        let output = invoke(&command, Some(payload), None);
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn every_governed_root_routes_and_adjacent_paths_skip() {
    prepare_prebuilt_tools();
    let command = configured_hook_command();
    let workspace = hostile_workspace();
    for path in [
        "workspaces/worth-contracts/crates/worth-schema-core/src/lib.rs",
        "workspaces/worth-query/crates/worth-query/src/lib.rs",
        "workspaces/worth-query/crates/worth-query-certification/Cargo.toml",
        "tools/new-constitutional-tool/src/lib.rs",
        "crates/worth-proof/src/lib.rs",
        ".claude/settings.json",
        "scripts/prepare-constitution-hook.ps1",
        "Cargo.toml",
    ] {
        let payload = format!(r#"{{"tool_name":"Edit","tool_input":{{"file_path":"{path}"}}}}"#);
        let output = invoke(&command, Some(&payload), Some(&workspace));
        assert!(!output.status.success(), "governed path skipped: {path}");
        assert!(
            !output.stdout.is_empty(),
            "governed path emitted no report: {path}"
        );
    }
    for path in [
        "cad/workspaces/example/src/lib.rs",
        "cad/docs/road.md",
        "workspaces/worth-ui/crates/worth-ui/src/lib.rs",
        "examples/unrelated/Cargo.toml",
        "tooling/readme.md",
        "README.md",
    ] {
        let payload = format!(r#"{{"tool_name":"Edit","tool_input":{{"file_path":"{path}"}}}}"#);
        let output = invoke(&command, Some(&payload), Some(&workspace));
        assert!(output.status.success(), "adjacent path routed: {path}");
        assert!(
            output.stdout.is_empty(),
            "adjacent path emitted a report: {path}"
        );
    }
    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn patch_text_targets_route_governed_edits_and_skip_proven_non_governed_edits() {
    prepare_prebuilt_tools();
    let command = configured_hook_command();
    let workspace = hostile_workspace();
    let governed = r#"{"tool_name":"apply_patch","tool_input":{"patch":"*** Begin Patch\n*** Update File: tools/boundary-check/src/main.rs\n@@\n-old\n+new\n*** End Patch"}}"#;
    let output = invoke(&command, Some(governed), Some(&workspace));
    assert!(!output.status.success());
    assert!(!output.stdout.is_empty());

    let non_governed = r#"{"tool_name":"apply_patch","tool_input":{"patch":"*** Begin Patch\n*** Update File: README.md\n@@\n-old\n+new\n*** End Patch"}}"#;
    let output = invoke(&command, Some(non_governed), Some(&workspace));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    fs::remove_dir_all(workspace).unwrap();
}
