use std::fs;

use super::support::{golden_envelope, ReleaseWorld, MISMATCH_CASES};

#[test]
fn preflight_stages_only_the_exact_canonical_payload_after_fresh_readmission() {
    let world = ReleaseWorld::new();
    let staged = world.output_path("staged.signing-payload");

    let result = world.run_preflight(&[], &staged);

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(fs::read(staged).unwrap(), world.signing_payload());
}

#[test]
fn preflight_enforces_every_independent_expectation_before_staging() {
    for (index, (field, value)) in MISMATCH_CASES.into_iter().enumerate() {
        let world = ReleaseWorld::new();
        let staged = world.output_path(&format!("denied-{index}.signing-payload"));

        let result = world.run_preflight(&[(field, value)], &staged);

        assert!(!result.status.success(), "{field} unexpectedly passed");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("expectation mismatch"),
            "{field}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!staged.exists());
    }
}

#[test]
fn preflight_rejects_complete_envelopes_and_matching_forged_identity_claims() {
    let world = ReleaseWorld::new();
    world.replace_signing_payload(&golden_envelope());
    let staged = world.output_path("complete-envelope.signing-payload");
    let complete = world.run_preflight(&[], &staged);
    assert!(!complete.status.success());
    assert!(!staged.exists());

    const EXPECTED_IDENTITY_OFFSET: usize = 18;
    let world = ReleaseWorld::new();
    let mut forged = world.signing_payload();
    forged[EXPECTED_IDENTITY_OFFSET] = 0;
    world.replace_signing_payload(&forged);
    let staged = world.output_path("forged.signing-payload");
    let forged_identity = "005923afa113ac168ab43d923a50b5e0cdee3002ea50888f4bd1453fc084bffd";
    let result = world.run_preflight(&[("--expected-package-identity", forged_identity)], &staged);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("fresh Query readmission denied"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!staged.exists());
}

#[test]
fn preflight_refuses_to_replace_an_existing_staged_payload() {
    let world = ReleaseWorld::new();
    let staged = world.output_path("existing.signing-payload");
    fs::write(&staged, b"operator-owned").unwrap();

    let result = world.run_preflight(&[], &staged);

    assert!(!result.status.success());
    assert_eq!(fs::read(staged).unwrap(), b"operator-owned");
}

#[test]
fn preflight_denies_an_impossible_signature_shape_before_staging() {
    let world = ReleaseWorld::new();
    let staged = world.output_path("impossible-signature.signing-payload");

    let result = world.run_preflight_with_signature_bytes(&[], u32::MAX, &staged);

    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("expected signature capacity"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!staged.exists());
}
