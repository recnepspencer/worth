use serde_json::Value;
use std::{path::Path, process::Command};

pub(super) fn compare(observer: &Path, runtime: &Path, offline: &Path, output: &Path) {
    let status = Command::new(observer)
        .args(["compare", "--runtime-observation"])
        .arg(runtime)
        .arg("--offline-observation")
        .arg(offline)
        .arg("--report")
        .arg(output)
        .status()
        .unwrap();
    assert!(status.success());
    let compared: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(compared["protocol"], "store.physical.integrity-comparison");
    let runtime_input: Value = serde_json::from_slice(&std::fs::read(runtime).unwrap()).unwrap();
    let offline_input: Value = serde_json::from_slice(&std::fs::read(offline).unwrap()).unwrap();
    assert_eq!(
        compared["runtime"], runtime_input,
        "comparison must retain the actual runtime report"
    );
    assert_eq!(
        compared["offline"], offline_input,
        "comparison must retain the actual offline report"
    );
    let rows = compared["comparisons"].as_array().unwrap();
    for actual in runtime_input["artifacts"].as_array().unwrap() {
        let counterpart = offline_input["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| {
                row["path"] == actual["path"]
                    && row["family"] == actual["family"]
                    && row["range"]["offset"] == actual["range"]["offset"]
            });
        let comparison = rows
            .iter()
            .find(|row| {
                row["path"] == actual["path"]
                    && row["family"] == actual["family"]
                    && row["offset"] == actual["range"]["offset"]
            })
            .unwrap();
        assert_eq!(
            comparison["agreement"],
            counterpart == Some(actual),
            "agreement must preserve all actual scope and diagnostic facts"
        );
        assert_eq!(
            comparison["different_fields"]
                .as_array()
                .unwrap()
                .is_empty(),
            counterpart == Some(actual)
        );
    }
    assert_eq!(
        compared["consumed"]["agreements"].as_u64().unwrap(),
        rows.iter().filter(|row| row["agreement"] == true).count() as u64
    );
    assert_eq!(
        compared["consumed"]["disagreements"].as_u64().unwrap(),
        rows.iter().filter(|row| row["agreement"] == false).count() as u64
    );
    // The offline inventory additionally names historical/unreachable paths; unmatched
    // requested scopes stay explicit disagreements, never edited out of either input.
}
