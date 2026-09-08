use crate::support::clean_store;
use std::process::Command;

#[test]
fn operator_guide_observe_command_executes_verbatim_with_only_path_substitution() {
    let fixture = clean_store("documented-command");
    let guide = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../_docs/worth-store/physical-integrity-and-offline-verification.md"
    ));
    let line = guide
        .lines()
        .find(|line| line.starts_with("physical_store_integrity_observer observe "))
        .expect("documented executable command");
    let mut command = Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"));
    for token in line.split_whitespace().skip(1) {
        match token {
            "<closed-store>" => {
                command.arg(&fixture.store);
            }
            "<external-report.json>" => {
                command.arg(&fixture.report);
            }
            literal => {
                command.arg(literal);
            }
        }
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let wire: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&fixture.report).unwrap()).unwrap();
    assert_eq!(wire["version"], 1);
    assert_eq!(wire["role"], "offline-root-observer");
    assert_eq!(wire["completeness"], "complete");
}
