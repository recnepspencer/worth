#[allow(unused_imports)]
use crate::hostile_process::*;
use crate::support::checksum::refresh_crc32c;

#[test]
fn binary_hostile_walk_keeps_alias_cost_bounds_and_output_separation() {
    let fixture = clean_store("binary-hostile-walk");
    let baseline = observe(&fixture.store, 512);
    let selector = fixture.records.join("root-current.selector");
    for index in 0..16 {
        std::fs::hard_link(
            &selector,
            fixture
                .records
                .join(format!("root-current-{index:016x}.candidate")),
        )
        .unwrap();
    }
    for index in 0..80 {
        std::fs::write(
            fixture.records.join(format!("unknown-{index:04}")),
            [0x5a; 1024],
        )
        .unwrap();
    }
    let outside = fixture.store.parent().unwrap().join("outside");
    std::fs::write(&outside, [0x33; 8192]).unwrap();
    let link = fixture.records.join("escaping-link");
    file_link(&outside, &link);
    let mut unsupported = std::fs::read(fixture.records.join("root-previous.selector")).unwrap();
    unsupported[9] = 3;
    refresh_crc32c(&mut unsupported);
    std::fs::write(fixture.records.join("root-previous.selector"), unsupported).unwrap();
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
    let written = std::fs::read(&fixture.report).unwrap();
    let second = run(&fixture.store, 512, &fixture.report);
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).contains("DestinationAlreadyExists"));
    assert_eq!(std::fs::read(&fixture.report).unwrap(), written);
    assert_eq!(snapshot(&fixture.store), original);
    assert_eq!(std::fs::read(outside).unwrap(), [0x33; 8192]);
}

#[cfg(windows)]
#[test]
fn binary_refuses_mutation_contention_then_reads_the_quiescent_source() {
    let fixture = clean_store("binary-mutation-contention");
    let path = fixture.records.join("root-current.selector");
    let original = snapshot(&fixture.store);
    // A write-capable OS handle conflicts with the observer's read-only sharing.
    // This tests prevention of an unstable snapshot, not a fictitious mid-read edit.
    let writer = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
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
