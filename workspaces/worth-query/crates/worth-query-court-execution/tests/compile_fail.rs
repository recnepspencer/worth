//! The real consumer fixture must name its member's unmet schema requirements.
use std::process::Command;

#[test]
fn foreign_schema_member_reports_the_schema_requirement() {
    let here = std::env::current_dir().unwrap();
    let root = here
        .ancestors()
        .find(|path| path.join("AGENTS.md").is_file())
        .unwrap();
    let manifest=root.join("workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml");
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(manifest)
        .args([
            "--keep-going",
            "--message-format=json",
            "-p",
            "worth-query-topology-entry",
            "-p",
            "worth-query-parameter-entry",
            "-p",
            "worth-query-consumer-root",
            "-p",
            "worth-query-hostile-foreign-schema",
        ])
        .current_dir(root)
        .output()
        .expect("run the consumer-entry compiler court");
    let text = String::from_utf8(output.stdout).unwrap();
    let diagnostics: Vec<serde_json::Value> = text
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let hostile = diagnostics
        .iter()
        .find(|event| {
            event["reason"] == "compiler-message"
                && event["package_id"]
                    .as_str()
                    .is_some_and(|package| package.contains("hostile_foreign_schema"))
                && event["message"]["level"] == "error"
        })
        .unwrap_or_else(|| {
            panic!(
                "hostile fixture did not reach its intended refusal: {}\n{text}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
    let rendered = hostile["message"]["rendered"].as_str().unwrap();
    for expected in [
        "TopologyContribution",
        "cannot be declared in schema `ConsumerSchema`",
        "schema requirements",
    ] {
        assert!(
            rendered.contains(expected),
            "expected diagnostic {expected:?}: {rendered}"
        );
    }
    for expected in [
        "worth_query_topology_entry",
        "worth_query_parameter_entry",
        "worth_query_consumer_root",
    ] {
        assert!(
            diagnostics
                .iter()
                .any(|event| event["reason"] == "compiler-artifact"
                    && event["target"]["name"]
                        .as_str()
                        .is_some_and(|name| name.replace('-', "_") == expected)),
            "positive consumer counterpart did not compile: {text}"
        );
    }
    assert!(
        !output.status.success(),
        "hostile schema unexpectedly compiled"
    );
}
