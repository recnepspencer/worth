use crate::support::{clean_store, refresh_crc32c};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[test]
fn binary_hostile_walk_keeps_alias_cost_bounds_and_output_separation() {
    let fixture = clean_store("binary-hostile-walk");
    let baseline = observe(&fixture.store, 512);
    let selector = fixture.records.join("root-current.selector");
    for index in 0..16 {
        fs::hard_link(
            &selector,
            fixture
                .records
                .join(format!("root-current-{index:016x}.candidate")),
        )
        .unwrap();
    }
    for index in 0..80 {
        fs::write(
            fixture.records.join(format!("unknown-{index:04}")),
            [0x5a; 1024],
        )
        .unwrap();
    }
    let outside = fixture.store.parent().unwrap().join("outside");
    fs::write(&outside, [0x33; 8192]).unwrap();
    let link = fixture.records.join("escaping-link");
    file_link(&outside, &link);
    let mut unsupported = fs::read(fixture.records.join("root-previous.selector")).unwrap();
    unsupported[9] = 3;
    refresh_crc32c(&mut unsupported);
    fs::write(fixture.records.join("root-previous.selector"), unsupported).unwrap();
    let original = snapshot(&fixture.store);
    let report = observe(&fixture.store, 512);
    assert_eq!(report["consumed"]["bytes"], baseline["consumed"]["bytes"]);
    assert_eq!(report["consumed"]["duplicate_identities"], 16);
    assert_eq!(report["consumed"]["symlinks_refused"], 1);
    assert_eq!(report["consumed"]["unsupported_versions"], 1);
    // The unsupported previous selector removes one checksum/decoder entry;
    // sixteen aliases must not put any of them back.
    for counter in [
        "checksum_calculations",
        "durable_frame_decoders",
        "selector_decoders",
    ] {
        assert_eq!(
            report["consumed"][counter].as_u64().unwrap() + 1,
            baseline["consumed"][counter].as_u64().unwrap(),
            "{counter}"
        );
    }
    let rows = report["artifacts"].as_array().unwrap();
    let aliases: Vec<_> = rows
        .iter()
        .filter(|row| row["path"].as_str().unwrap().ends_with(".candidate"))
        .collect();
    assert_eq!(aliases.len(), 16);
    for alias in aliases {
        assert_eq!(
            alias["outcome"],
            json!({"posture":"unknown","reason":"physical_alias_not_reinspected"})
        );
        assert_eq!(
            alias["duplicates"],
            json!([{"kind":"physical_alias","first_path":"families/records/root-current.selector"}])
        );
    }
    assert_eq!(
        rows.iter()
            .filter(|row| row["path"].as_str().unwrap().contains("unknown-"))
            .count(),
        80
    );
    assert_eq!(
        rows.iter()
            .find(|row| row["path"] == "families/records/escaping-link")
            .unwrap()["outcome"],
        json!({"posture":"indeterminate","reason":"symlink_refused"})
    );

    let bounded = observe(&fixture.store, 16);
    assert!(bounded["consumed"]["entries"].as_u64().unwrap() <= 16);
    assert!(bounded["consumed"]["bytes"].as_u64().unwrap() <= 1_048_576);
    assert!(bounded["consumed"]["exhausted_bounds"].as_u64().unwrap() > 0);
    assert_eq!(bounded["completeness"], "bound_exhausted");
    assert!(bounded["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["outcome"]
            == json!({"posture":"indeterminate","reason":"entry_bound_exceeded"})));

    let inside = fixture.store.join("observer-report.json");
    let denied = run(&fixture.store, 512, &inside);
    assert!(!denied.status.success());
    assert!(!inside.exists());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("DestinationInsideStoreRoot"));
    let external = run(&fixture.store, 512, &fixture.report);
    assert!(
        external.status.success(),
        "{}",
        String::from_utf8_lossy(&external.stderr)
    );
    let written = fs::read(&fixture.report).unwrap();
    let second = run(&fixture.store, 512, &fixture.report);
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).contains("DestinationAlreadyExists"));
    assert_eq!(fs::read(&fixture.report).unwrap(), written);
    assert_eq!(snapshot(&fixture.store), original);
    assert_eq!(fs::read(outside).unwrap(), [0x33; 8192]);
}

#[cfg(windows)]
#[test]
fn binary_refuses_mutation_contention_then_reads_the_quiescent_source() {
    let fixture = clean_store("binary-mutation-contention");
    let path = fixture.records.join("root-current.selector");
    let original = snapshot(&fixture.store);
    // A write-capable OS handle conflicts with the observer's read-only sharing.
    // This tests prevention of an unstable snapshot, not a fictitious mid-read edit.
    let writer = fs::OpenOptions::new().write(true).open(&path).unwrap();
    let contended = observe(&fixture.store, 512);
    let row = contended["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == "families/records/root-current.selector")
        .unwrap();
    assert_eq!(
        row["outcome"],
        json!({"posture":"indeterminate","reason":"io_failure"})
    );
    drop(writer);
    let quiescent = observe(&fixture.store, 512);
    let row = quiescent["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == "families/records/root-current.selector")
        .unwrap();
    assert_eq!(row["outcome"], json!({"posture":"intact"}));
    assert_eq!(snapshot(&fixture.store), original);
}

fn observe(root: &Path, entries: u64) -> Value {
    let output = run(root, entries, Path::new("-"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn run(root: &Path, entries: u64, report: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"))
        .args(["observe", "--store-root"])
        .arg(root)
        .arg("--report")
        .arg(report)
        .arg("--max-entries")
        .arg(entries.to_string())
        .args([
            "--max-bytes",
            "1048576",
            "--max-open-files",
            "8",
            "--max-depth",
            "12",
            "--max-symlinks",
            "4",
            "--max-elapsed-ms",
            "10000",
            "--max-report-bytes",
            "262144",
        ])
        .output()
        .unwrap()
}

#[cfg(windows)]
fn file_link(source: &Path, target: &Path) {
    std::os::windows::fs::symlink_file(source, target).unwrap();
}
#[cfg(unix)]
fn file_link(source: &Path, target: &Path) {
    std::os::unix::fs::symlink(source, target).unwrap();
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, (u8, Vec<u8>)> {
    let mut result = BTreeMap::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            let contents = if kind.is_symlink() {
                (
                    2,
                    fs::read_link(&path)
                        .unwrap()
                        .to_string_lossy()
                        .as_bytes()
                        .to_vec(),
                )
            } else if kind.is_dir() {
                pending.push(path.clone());
                (1, Vec::new())
            } else {
                (0, fs::read(&path).unwrap())
            };
            result.insert(path.strip_prefix(root).unwrap().to_owned(), contents);
        }
    }
    result
}
