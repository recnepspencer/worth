use super::post_tool_use_fixture::*;
use serde_json::Value;
use std::fs;
use std::process::Command;
use std::time::Instant;

#[test]
fn root_manifest_edit_and_patch_match_the_hand_entrypoint() {
    prepare_prebuilt_tools();
    let command = configured_hook_command();
    let source = root().join("tools/boundary-check/tests/fixtures/root_owned_road1_package");
    let workspace = hostile_workspace();
    fs::copy(source.join("Cargo.toml"), workspace.join("Cargo.toml")).unwrap();
    let expected = invoke(&hand_entrypoint_command(), None, Some(&workspace));
    for payload in [
        r#"{"tool_name":"Edit","tool_input":{"file_path":"Cargo.toml"}}"#,
        r#"{"tool_name":"apply_patch","tool_input":{"patch":"*** Begin Patch\n*** Update File: Cargo.toml\n@@\n-old\n+new\n*** End Patch"}}"#,
    ] {
        let actual = invoke(&command, Some(payload), Some(&workspace));
        assert_eq!(expected.status.code(), actual.status.code());
        assert_eq!(expected.stdout, actual.stdout);
    }
    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn shell_mutation_routes_to_the_same_json_entrypoint() {
    prepare_prebuilt_tools();
    let workspace = hostile_workspace();
    let manifest =
        workspace.join("cad/workspaces/worth-contracts/crates/worth-schema-core/Cargo.toml");
    let illegal_dependency = "worth-query = { path = \"../../../../../vendor/WORTH-query\" }";
    let legal = fs::read_to_string(&manifest)
        .unwrap()
        .replace(illegal_dependency, "");
    fs::write(&manifest, legal).unwrap();
    let mutation = Command::new("powershell")
        .args(["-NoProfile", "-Command"])
        .arg("Add-Content -LiteralPath $env:WORTH_MUTATION_PATH -Value $env:WORTH_MUTATION_TEXT")
        .env("WORTH_MUTATION_PATH", &manifest)
        .env("WORTH_MUTATION_TEXT", illegal_dependency)
        .output()
        .unwrap();
    assert!(mutation.status.success());

    let direct_output = invoke(&hand_entrypoint_command(), None, Some(&workspace));
    let started = Instant::now();
    let hook_output = invoke(
        &configured_hook_command(),
        Some(r#"{"tool_name":"Bash","tool_input":{"command":"opaque shell mutation"}}"#),
        Some(&workspace),
    );
    let elapsed = started.elapsed();
    assert_eq!(direct_output.status.code(), hook_output.status.code());
    assert_eq!(direct_output.stdout, hook_output.stdout);
    assert!(elapsed <= HOOK_BUDGET, "shell hook took {elapsed:?}");
    let report: Value = serde_json::from_slice(&hook_output.stdout).unwrap();
    assert!(diagnostics(&report)
        .iter()
        .any(|item| item["code"] == "BC3001_DIRECT_QUERY_ENGINE"));
    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn configured_hook_matches_hand_entrypoint_on_hostile_query_edit_within_budget() {
    prepare_prebuilt_tools();
    let workspace = hostile_workspace();
    let direct_output = invoke(&hand_entrypoint_command(), None, Some(&workspace));
    let started = Instant::now();
    let hook_output = invoke(
        &configured_hook_command(),
        Some(
            r#"{"tool_name":"Edit","tool_input":{"file_path":"cad/workspaces/worth-contracts/crates/worth-schema-core/Cargo.toml"}}"#,
        ),
        Some(&workspace),
    );
    let elapsed = started.elapsed();
    assert_eq!(direct_output.status.code(), hook_output.status.code());
    assert_eq!(direct_output.stdout, hook_output.stdout);
    assert!(elapsed <= HOOK_BUDGET, "hook took {elapsed:?}");

    let report: Value = serde_json::from_slice(&hook_output.stdout).unwrap();
    let diagnostics = diagnostics(&report);
    let query = diagnostics
        .iter()
        .find(|item| item["code"] == "BC3001_DIRECT_QUERY_ENGINE")
        .unwrap();
    assert_eq!(query["legal_home"], "tools/boundary-check/config/road1.toml [rule_contracts.query_audience]: no Query audience is legal for `schema`; remove the Query dependency");
    assert!(diagnostics.iter().any(|item| {
        item["code"] == "BC3003_QUERY_AUDIENCE_FACADE_CONTRACT"
            && item["legal_home"] == "tools/boundary-check/config/road1.toml [rule_contracts.query_audience]; restore the configured workspaces/worth-query package or audience leaf"
    }));
    assert!(diagnostics.iter().any(|item| {
        item["code"] == "BC5002_SUBWORKSPACE_CONTRACT_VIOLATION"
            && item["legal_home"]
                .as_str()
                .unwrap()
                .contains("machine_authority.mirrored_docs")
    }));
    assert!(diagnostics.iter().any(|item| {
        item["code"] == "BC8001_SNAPSHOT_BASELINE"
            && item["legal_home"]
                .as_str()
                .unwrap()
                .starts_with("tools/boundary-check/snapshots/")
    }));
    fs::remove_dir_all(workspace).unwrap();
}
