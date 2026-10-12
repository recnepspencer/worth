mod post_tool_use_fixture;

use post_tool_use_fixture::*;
use serde_json::Value;
use std::fs;
use std::process::Command;
use std::time::{Instant, SystemTime};

#[test]
fn removed_or_retargeted_hook_authority_is_rejected() {
    let direct = hand_entrypoint_command();
    for settings in [
        r#"{"hooks":{"PostToolUse":[]}}"#,
        r#"{"hooks":{"PostToolUse":[{"matcher":"Write|Edit|MultiEdit|apply_patch|Bash","hooks":[{"type":"command","command":"powershell -NoProfile -File scripts/redirect.ps1"}]}]}}"#,
        r#"{"hooks":{"SessionStart":[],"PostToolUse":[{"matcher":"Write|Edit|MultiEdit|apply_patch|Bash","hooks":[{"type":"command","command":"powershell -NoProfile -File scripts/check-constitution-post-tool-use.ps1"}]}]}}"#,
        r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"powershell -NoProfile -File scripts/redirect.ps1"}]}],"PostToolUse":[{"matcher":"Write|Edit|MultiEdit|apply_patch|Bash","hooks":[{"type":"command","command":"powershell -NoProfile -File scripts/check-constitution-post-tool-use.ps1"}]}]}}"#,
    ] {
        let workspace = hostile_workspace();
        fs::write(workspace.join(".claude/settings.json"), settings).unwrap();
        let output = invoke(&direct, None, Some(&workspace));
        assert!(!output.status.success());
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(diagnostics(&report).iter().any(|item| {
            item["code"] == "BC5004_HOOK_AUTHORITY_VIOLATION"
                && item["subject"] == ".claude/settings.json"
                && item["legal_home"]
                    == ".claude/settings.json [hooks.SessionStart, hooks.PostToolUse]; restore the canonical prepare and check commands"
        }));
        fs::remove_dir_all(workspace).unwrap();
    }
}

#[test]
fn session_start_prepares_an_isolated_target_for_the_budgeted_hook() {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let target = std::env::temp_dir().join(format!("worth-phase7-tools-{nonce}"));
    let prepare = configured_command("SessionStart");
    let prepared = worth_harness::tooling::process_deadline::output(
        Command::new(&prepare[0])
            .args(&prepare[1..])
            .current_dir(root())
            .env("WORTH_CONSTITUTION_TOOL_TARGET", &target),
    )
    .unwrap();
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );

    let workspace = hostile_workspace();
    let hook = configured_hook_command();
    let started = Instant::now();
    let mut child = Command::new(&hook[0]);
    child
        .args(&hook[1..])
        .current_dir(root())
        .env("WORTH_CONSTITUTION_ROOT", &workspace)
        .env("WORTH_CONSTITUTION_TOOL_TARGET", &target);
    let output = output_with_payload(
        &mut child,
        Some(r#"{"tool_name":"Edit","tool_input":{"file_path":"Cargo.toml"}}"#),
    );
    assert!(started.elapsed() <= HOOK_BUDGET);
    assert!(!output.stdout.is_empty());
    fs::remove_dir_all(workspace).unwrap();
    fs::remove_dir_all(target).unwrap();
}
