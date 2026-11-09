use std::fs;

use super::support::{ReleaseWorld, MISMATCH_CASES};

#[test]
fn every_independent_release_expectation_is_enforced_before_output() {
    for (index, (field, value)) in MISMATCH_CASES.into_iter().enumerate() {
        let world = ReleaseWorld::new();
        let envelope = world.output_path(&format!("denied-{index}.worth-query"));
        let report = world.output_path(&format!("denied-{index}.json"));
        let result = world.run(&[(field, value)], &envelope, &report);
        assert!(!result.status.success(), "{field} unexpectedly passed");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("expectation mismatch"),
            "{field}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!envelope.exists());
        assert!(!report.exists());
    }
}

#[test]
fn malformed_payload_and_invalid_signature_leave_no_release_outputs() {
    let world = ReleaseWorld::new();
    let mut tampered = world.signing_payload();
    tampered[0] ^= 0xff;
    world.replace_signing_payload(&tampered);
    assert_denied_without_output(&world, "tampered");

    let world = ReleaseWorld::new();
    world.replace_signature(&[]);
    assert_denied_without_output(&world, "empty-signature");

    let world = ReleaseWorld::new();
    world.replace_signature(&vec![0xa5; 16 * 1_024 + 1]);
    assert_denied_without_output(&world, "oversized-signature");
}

#[test]
fn finalization_enforces_the_host_expected_signature_shape() {
    let world = ReleaseWorld::new();
    world.replace_signature(&[0xa5; 63]);
    let envelope = world.output_path("wrong-signature-shape.worth-query");
    let report = world.output_path("wrong-signature-shape.json");

    let result = world.run(&[], &envelope, &report);

    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("signature byte count"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!envelope.exists());
    assert!(!report.exists());
}

#[test]
fn matching_forged_claim_still_fails_fresh_query_identity_derivation() {
    const ENVELOPE_EXPECTED_IDENTITY_OFFSET: usize = 18;
    let world = ReleaseWorld::new();
    let mut payload = world.signing_payload();
    payload[ENVELOPE_EXPECTED_IDENTITY_OFFSET] = 0;
    world.replace_signing_payload(&payload);
    let envelope = world.output_path("forged-claim.worth-query");
    let report = world.output_path("forged-claim.json");
    let forged_identity = "005923afa113ac168ab43d923a50b5e0cdee3002ea50888f4bd1453fc084bffd";
    let result = world.run(
        &[("--expected-package-identity", forged_identity)],
        &envelope,
        &report,
    );
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("fresh Query readmission denied"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!envelope.exists());
    assert!(!report.exists());
}

#[test]
fn ceremony_refuses_to_replace_an_existing_release() {
    let world = ReleaseWorld::new();
    let envelope = world.output_path("existing.worth-query");
    let report = world.output_path("existing.json");
    fs::write(&envelope, b"operator-owned").unwrap();
    let result = world.run(&[], &envelope, &report);
    assert!(!result.status.success());
    assert_eq!(fs::read(&envelope).unwrap(), b"operator-owned");
    assert!(!report.exists());
}

fn assert_denied_without_output(world: &ReleaseWorld, name: &str) {
    let envelope = world.output_path(&format!("{name}.worth-query"));
    let report = world.output_path(&format!("{name}.json"));
    let result = world.run(&[], &envelope, &report);
    assert!(!result.status.success(), "{name} unexpectedly passed");
    assert!(!envelope.exists());
    assert!(!report.exists());
}
