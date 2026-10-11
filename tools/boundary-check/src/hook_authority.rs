use crate::diagnostics::{Diagnostic, DiagnosticCode};
use serde_json::Value;
use std::path::Path;

const SETTINGS_PATH: &str = ".claude/settings.json";
const MUTATING_TOOLS: &str = "Write|Edit|MultiEdit|apply_patch|Bash";
const PREPARE_COMMAND: &str = "powershell -NoProfile -File scripts/prepare-constitution-hook.ps1";
const CANONICAL_COMMAND: &str =
    "powershell -NoProfile -File scripts/check-constitution-post-tool-use.ps1";

pub(crate) fn validate_hook_authority(root: &Path) -> Vec<Diagnostic> {
    // Hook registration is checkout authority. Production-binary fixtures are
    // complete governed roots, but they are not authoring checkouts.
    if !root.join(".git").exists() {
        return Vec::new();
    }
    let settings = match crate::source_documents::read_to_string(root.join(SETTINGS_PATH)) {
        Ok(settings) => settings,
        Err(error) => return vec![violation(format!("read hook authority failed: {error}"))],
    };
    let settings: Value = match serde_json::from_str(&settings) {
        Ok(settings) => settings,
        Err(error) => return vec![violation(format!("parse hook authority failed: {error}"))],
    };
    let session_start = settings["hooks"]["SessionStart"]
        .as_array()
        .is_some_and(|registrations| {
            registrations.len() == 1
                && registrations[0]["hooks"].as_array().is_some_and(|hooks| {
                    hooks.len() == 1
                        && hooks[0]["type"] == "command"
                        && hooks[0]["command"] == PREPARE_COMMAND
                })
        });
    let post_tool_use = settings["hooks"]["PostToolUse"]
        .as_array()
        .is_some_and(|registrations| {
            registrations.len() == 1
                && registrations[0]["matcher"] == MUTATING_TOOLS
                && registrations[0]["hooks"].as_array().is_some_and(|hooks| {
                    hooks.len() == 1
                        && hooks[0]["type"] == "command"
                        && hooks[0]["command"] == CANONICAL_COMMAND
                })
        });
    if session_start && post_tool_use {
        Vec::new()
    } else {
        vec![violation(
            "SessionStart must prepare the constitution tools and PostToolUse must route every supported mutating tool through the canonical constitution adapter"
                .to_owned(),
        )]
    }
}

fn violation(message: String) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc5004HookAuthorityViolation,
        SETTINGS_PATH,
        message,
    )
}
