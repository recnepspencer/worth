use std::fs;

use serde_json::Value;

use super::support::{golden_envelope, ReleaseWorld};

#[test]
fn external_signature_ceremony_reproduces_the_frozen_release_and_report() {
    let world = ReleaseWorld::new();
    let first_envelope = world.output_path("first.worth-query");
    let first_report = world.output_path("first.json");
    let first = world.run(&[], &first_envelope, &first_report);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(fs::read(&first_envelope).unwrap(), golden_envelope());

    let report: Value = serde_json::from_slice(&fs::read(&first_report).unwrap()).unwrap();
    assert_eq!(report["artifact_posture"], "untrusted-signed-envelope");
    assert_eq!(
        report["package_identity"],
        "b252098143f06caf6cc143c0af20bc10778339ae6d1e250ad7fb9b3bce14a9b8"
    );
    assert_eq!(report["source_reference"], "refs/tags/query-9.16.2");
    // The golden envelope is the archive crate's release golden, re-frozen
    // whenever a layer's exact protocol advances; these literals move with it.
    assert_eq!(report["envelope_protocol_version"], 1);
    assert_eq!(report["archive_protocol_version"], 2);
    assert_eq!(report["manifest_protocol_version"], 1);
    assert_eq!(report["record_protocol_version"], 6);

    let second_envelope = world.output_path("second.worth-query");
    let second_report = world.output_path("second.json");
    let second = world.run(&[], &second_envelope, &second_report);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        fs::read(first_envelope).unwrap(),
        fs::read(second_envelope).unwrap()
    );
    assert_eq!(
        fs::read(first_report).unwrap(),
        fs::read(second_report).unwrap()
    );
}
