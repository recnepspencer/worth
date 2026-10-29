#[allow(unused_imports)] use crate::phase_eight_observer_cli::*;
use {sha2::Digest, sha2::Sha256};
use {std::process::Command, std::process::Output};
use {std::time::Duration, std::time::Instant};

use {worth_store_offline_verifier::RecoveryObserverDecodeDenial, worth_store_offline_verifier::RecoveryObserverReport, worth_store_offline_verifier::RECOVERY_OBSERVER_REPORT_PROTOCOL, worth_store_offline_verifier::RECOVERY_OBSERVER_REPORT_VERSION};
use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
use {worth_store_physical_format::DurableRootSelector, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::RootSelectorIdentity, worth_store_physical_format::RootSelectorRole};

#[test]
fn shipped_observer_process_emits_the_version_one_bounded_report() {
    let root = tempfile::tempdir().expect("observer input root");
    let output_root = tempfile::tempdir().expect("observer output root");
    std::fs::write(root.path().join("selector"), b"observed").expect("observer input");
    let output = output_root.path().join("observer-report.bin");

    let process = run_bounded(
        std::process::Command::new(worth_store_testkit_executables::physical_store_offline_observer())
            .arg("c8-recovery-observe")
            .arg(root.path())
            .arg(&output)
            .args(["2", "1", "1", "8"]),
    );
    assert!(
        process.status.success(),
        "observer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr),
    );

    let report =
        worth_store_offline_verifier::RecoveryObserverReport::decode(&std::fs::read(output).expect("observer report output"))
            .expect("version-one observer report");
    assert_eq!(RECOVERY_OBSERVER_REPORT_VERSION.get(), 1);
    assert_eq!(report.artifact_count(), 1);
    assert_eq!(report.bytes_read(), 8);
    assert_ne!(report.artifact_set_digest(), [0; 32]);
    assert_eq!(report.artifact_identity_count(), 1);
    assert_eq!(report.durable_selector_count(), 0);
    assert_eq!(report.page_lsn_count(), 0);
    assert_eq!(report.manifest_count(), 0);
    assert_eq!(report.residue_artifact_count(), 1);
    assert_eq!(report.residue_bytes(), 8);

    let forbidden = root.path().join("observer-report-inside-root.bin");
    let before = std::fs::read(root.path().join("selector")).expect("selector before rejection");
    let rejected = run_bounded(
        std::process::Command::new(worth_store_testkit_executables::physical_store_offline_observer())
            .arg("c8-recovery-observe")
            .arg(root.path())
            .arg(&forbidden)
            .args(["2", "1", "1", "8"]),
    );
    assert!(!rejected.status.success());
    assert!(!forbidden.exists());
    assert_eq!(std::fs::read(root.path().join("selector")).unwrap(), before);
}

#[test]
fn observer_reports_content_based_selector_links_and_mutation() {
    let root = tempfile::tempdir().expect("observer input root");
    let output_root = tempfile::tempdir().expect("observer output root");
    let artifact = root.path().join("arbitrary-content-name");
    std::fs::write(&artifact, selector_bytes(19, 17, 8, 7)).expect("selector artifact");
    let first_output = output_root.path().join("first-report.bin");
    run_observer(root.path(), &first_output, 107);
    let first = worth_store_offline_verifier::RecoveryObserverReport::decode(
        &std::fs::read(&first_output).expect("first observer report output"),
    )
    .expect("first observer report");
    assert_eq!(first.durable_selector_count(), 1);
    assert_eq!(first.linked_selector_count(), 1);
    assert_eq!(first.unpaired_selector_link_count(), 1);
    assert_eq!(first.generation_link_count(), 1);
    assert_eq!(first.page_lsn_count(), 1);
    assert_eq!(first.residue_bytes(), 0);

    std::fs::write(&artifact, selector_bytes(20, 99, 9, 10)).expect("mutated selector artifact");
    let second_output = output_root.path().join("second-report.bin");
    run_observer(root.path(), &second_output, 107);
    let second = worth_store_offline_verifier::RecoveryObserverReport::decode(
        &std::fs::read(&second_output).expect("second observer report output"),
    )
    .expect("second observer report");
    assert_eq!(second.durable_selector_count(), 1);
    assert_eq!(second.unpaired_selector_link_count(), 1);
    assert_ne!(
        first.durable_selector_digest(),
        second.durable_selector_digest()
    );
    assert_ne!(
        first.generation_link_digest(),
        second.generation_link_digest()
    );
}

#[test]
fn shipped_observer_report_rejects_invalid_future_malformed_and_tampered_wire_inputs() {
    let root = tempfile::tempdir().expect("observer input root");
    let output_root = tempfile::tempdir().expect("observer output root");
    std::fs::write(root.path().join("selector"), b"observed").expect("observer input");
    let output = output_root.path().join("observer-report.bin");
    let process = run_bounded(
        std::process::Command::new(worth_store_testkit_executables::physical_store_offline_observer())
            .arg("c8-recovery-observe")
            .arg(root.path())
            .arg(&output)
            .args(["2", "1", "1", "8"]),
    );
    assert!(process.status.success());
    let encoded = std::fs::read(output).expect("observer report output");

    let mut wrong_family = encoded.clone();
    wrong_family[8] = b'x';
    refresh_digest(&mut wrong_family);
    assert_eq!(
        worth_store_offline_verifier::RecoveryObserverReport::decode(&wrong_family),
        Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::WrongProtocolFamily)
    );

    let version_offset = 8 + RECOVERY_OBSERVER_REPORT_PROTOCOL.as_str().len();
    let mut retired = encoded.clone();
    retired[version_offset..version_offset + 4].copy_from_slice(&0_u32.to_le_bytes());
    refresh_digest(&mut retired);
    assert_eq!(
        worth_store_offline_verifier::RecoveryObserverReport::decode(&retired),
        Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::Malformed)
    );

    let mut future = encoded.clone();
    future[version_offset..version_offset + 4].copy_from_slice(&2_u32.to_le_bytes());
    refresh_digest(&mut future);
    assert!(matches!(
        worth_store_offline_verifier::RecoveryObserverReport::decode(&future),
        Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::UnsupportedVersion(_))
    ));

    let mut tampered = encoded.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert_eq!(
        worth_store_offline_verifier::RecoveryObserverReport::decode(&tampered),
        Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::DigestMismatch)
    );
    assert_eq!(
        worth_store_offline_verifier::RecoveryObserverReport::decode(&encoded[..31]),
        Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::Malformed)
    );

    let payload_end = encoded.len() - 32;
    let header_end = 8 + RECOVERY_OBSERVER_REPORT_PROTOCOL.as_str().len() + 4;
    for payload_length in header_end..payload_end {
        let truncated = rehashed_prefix(&encoded, payload_length);
        assert_eq!(
            worth_store_offline_verifier::RecoveryObserverReport::decode(&truncated),
            Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::Malformed),
            "field-boundary truncation at {payload_length}"
        );
    }

    let mut trailing = encoded[..payload_end].to_vec();
    trailing.push(0xaa);
    trailing.extend_from_slice(&sha2::Sha256::digest(&trailing));
    assert_eq!(
        worth_store_offline_verifier::RecoveryObserverReport::decode(&trailing),
        Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::Malformed)
    );

    let selectors_start = header_end + 8 + 8 + 32 + 40 + 40;
    for optional_offset in [selectors_start + 24, selectors_start + 25] {
        let mut malformed_optional = encoded.clone();
        malformed_optional[optional_offset] = 2;
        refresh_digest(&mut malformed_optional);
        assert_eq!(
            worth_store_offline_verifier::RecoveryObserverReport::decode(&malformed_optional),
            Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::Malformed),
            "invalid optional flag at {optional_offset}"
        );
    }
}

#[test]
fn observer_v1_literal_fixture_rejects_every_optional_flag_and_every_truncation() {
    let (encoded, optional_flags) = literal_v1_report();
    worth_store_offline_verifier::RecoveryObserverReport::decode(&encoded).expect("literal version-one report");
    let valid_payload_end = encoded.len() - 32;
    for length in 0..encoded.len() {
        if length == valid_payload_end {
            continue;
        }
        assert_eq!(
            worth_store_offline_verifier::RecoveryObserverReport::decode(&rehashed_prefix(&encoded, length)),
            Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::Malformed),
            "literal v1 truncation at {length}"
        );
    }
    for offset in optional_flags {
        let mut malformed = encoded.clone();
        malformed[offset] = 2;
        refresh_digest(&mut malformed);
        assert_eq!(
            worth_store_offline_verifier::RecoveryObserverReport::decode(&malformed),
            Err(worth_store_offline_verifier::RecoveryObserverDecodeDenial::Malformed),
            "literal v1 optional flag at {offset}"
        );
    }
}
