#![allow(dead_code)]

use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, SystemTime};

pub const HOOK_BUDGET: Duration = Duration::from_secs(15);

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn configured_command(event: &str) -> Vec<String> {
    let settings: Value =
        serde_json::from_str(&fs::read_to_string(root().join(".claude/settings.json")).unwrap())
            .unwrap();
    settings["hooks"][event][0]["hooks"][0]["command"]
        .as_str()
        .unwrap()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

pub fn configured_hook_command() -> Vec<String> {
    configured_command("PostToolUse")
}

pub fn hand_entrypoint_command() -> Vec<String> {
    vec![
        "powershell".into(),
        "-NoProfile".into(),
        "-File".into(),
        "scripts/check-constitution.ps1".into(),
        "--format".into(),
        "json".into(),
    ]
}

pub fn invoke(command: &[String], payload: Option<&str>, governed_root: Option<&Path>) -> Output {
    let mut child = Command::new(&command[0]);
    child
        .args(&command[1..])
        .current_dir(root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(root) = governed_root {
        child.env("WORTH_CONSTITUTION_ROOT", root);
    }
    let mut child = child.spawn().expect("start configured PostToolUse hook");
    if let Some(payload) = payload {
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(payload.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

pub fn prepare_prebuilt_tools() {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-File",
            "scripts/prepare-constitution-hook.ps1",
        ])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = if entry.file_name() == "Cargo.fixture.toml" {
            destination.join("Cargo.toml")
        } else {
            destination.join(entry.file_name())
        };
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

pub fn hostile_workspace() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("worth-phase7-hook-{nonce}"));
    copy_tree(
        &root().join("tools/boundary-check/tests/fixtures/schema_query_import"),
        &path,
    );
    fs::create_dir_all(path.join(".claude")).unwrap();
    fs::create_dir(path.join(".git")).unwrap();
    fs::copy(
        root().join(".claude/settings.json"),
        path.join(".claude/settings.json"),
    )
    .unwrap();
    path
}

pub fn diagnostics(value: &Value) -> Vec<&Value> {
    let mut found = Vec::new();
    fn visit<'a>(value: &'a Value, found: &mut Vec<&'a Value>) {
        if value.get("code").is_some() && value.get("legal_home").is_some() {
            found.push(value);
        }
        match value {
            Value::Array(values) => values.iter().for_each(|value| visit(value, found)),
            Value::Object(values) => values.values().for_each(|value| visit(value, found)),
            _ => {}
        }
    }
    visit(value, &mut found);
    found
}
