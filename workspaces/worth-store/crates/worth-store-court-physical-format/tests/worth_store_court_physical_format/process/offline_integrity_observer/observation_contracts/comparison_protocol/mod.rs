#[allow(unused_imports)]
use crate::comparison_protocol::*;

#[test]
fn real_compare_binary_preserves_inputs_and_rejects_alias_output_and_repair_flags() {
    let fixture = clean_store("comparison-binary");
    let (runtime, offline) = reports();
    let directory = fixture.store.parent().unwrap();
    let runtime_path = directory.join("runtime.json");
    let offline_path = directory.join("offline.json");
    let output_path = directory.join("comparison.json");
    std::fs::write(&runtime_path, runtime.to_string()).unwrap();
    std::fs::write(&offline_path, offline.to_string()).unwrap();
    let guide = include_str!(concat!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../worth-store-offline-integrity-observer"
        ),
        "/../../../../plans/worth-store/physical-integrity-and-offline-verification.md"
    ));
    let line = guide
        .lines()
        .find(|line| line.starts_with("physical_store_integrity_observer compare "))
        .expect("documented comparison command");
    let invoke = |output: &std::path::Path, extra: &[&str]| {
        let mut command = std::process::Command::new(
            worth_store_testkit_executables::physical_store_integrity_observer(),
        );
        // Execute the actual guide syntax with path substitution, not a mirrored CLI.
        for token in line.split_whitespace().skip(1) {
            match token {
                "<runtime-v1.json>" => command.arg(&runtime_path),
                "<external-report.json>" => command.arg(&offline_path),
                "<external-comparison.json>" => command.arg(output),
                literal => command.arg(literal),
            };
        }
        command.args(extra).output().unwrap()
    };
    let result = invoke(&output_path, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result: Value = serde_json::from_slice(&std::fs::read(&output_path).unwrap()).unwrap();
    assert_eq!(result["runtime"], runtime);
    assert!(!invoke(&runtime_path, &[]).status.success());
    let alias = directory.join("alias.json");
    std::fs::hard_link(&runtime_path, &alias).unwrap();
    assert!(!invoke(&alias, &[]).status.success());
    assert!(
        !invoke(&directory.join("repair.json"), &["--repair", "yes"])
            .status
            .success()
    );
    assert_eq!(
        std::fs::read_to_string(&runtime_path).unwrap(),
        runtime.to_string()
    );
    assert_eq!(
        std::fs::read_to_string(&offline_path).unwrap(),
        offline.to_string()
    );
}
