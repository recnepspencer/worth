use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The constitution has three runners, the hook, the terminal and CI, and all
/// three go through the one entrypoint. CI runs that entrypoint; it must not
/// grow a second boundary lane that invokes the tools directly.
#[test]
fn hook_terminal_and_ci_converge_on_one_entrypoint() {
    let root = root();
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    let settings = fs::read_to_string(root.join(".claude/settings.json")).unwrap();
    let hook =
        fs::read_to_string(root.join("scripts/check-constitution-post-tool-use.ps1")).unwrap();
    let entrypoint = fs::read_to_string(root.join("scripts/check-constitution.ps1")).unwrap();

    assert!(ci.contains("./scripts/check-constitution.ps1"));
    assert!(!ci.contains("Road 1 boundary enforcement"));
    assert!(!ci.contains("Generated crate contexts are fresh"));
    assert!(!ci.contains("tools/boundary-check/Cargo.toml"));
    assert!(!ci.contains("tools/agent-context/Cargo.toml"));
    assert!(settings.contains("scripts/check-constitution-post-tool-use.ps1"));
    assert!(settings.contains("scripts/prepare-constitution-hook.ps1"));
    assert!(settings.contains("Write|Edit|MultiEdit|apply_patch|Bash"));
    assert!(hook.contains("check-constitution.ps1"));
    assert!(hook.contains("WORTH_CONSTITUTION_PREBUILT"));
    assert!(!hook.contains("tools/boundary-check/Cargo.toml"));
    assert!(!hook.contains("tools/agent-context/Cargo.toml"));
    assert_eq!(
        entrypoint
            .matches("tools/boundary-check/Cargo.toml")
            .count(),
        2
    );
    assert_eq!(
        entrypoint.matches("tools/agent-context/Cargo.toml").count(),
        2
    );
}
