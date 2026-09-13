use sha2::{Digest, Sha256};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use worth_store_offline_verifier::{
    RecoveryObserverDecodeDenial, RecoveryObserverReport, RECOVERY_OBSERVER_REPORT_PROTOCOL,
    RECOVERY_OBSERVER_REPORT_VERSION,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    DurableRootSelector, PhysicalRecordFormatDeclaration, RootSelectorIdentity, RootSelectorRole,
};

#[test]
fn shipped_observer_process_emits_the_version_one_bounded_report() {
    let root = tempfile::tempdir().expect("observer input root");
    let output_root = tempfile::tempdir().expect("observer output root");
    std::fs::write(root.path().join("selector"), b"observed").expect("observer input");
    let output = output_root.path().join("observer-report.bin");

    let process = run_bounded(
        Command::new(env!("CARGO_BIN_EXE_physical_store_offline_observer"))
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
        RecoveryObserverReport::decode(&std::fs::read(output).expect("observer report output"))
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
        Command::new(env!("CARGO_BIN_EXE_physical_store_offline_observer"))
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
    let first = RecoveryObserverReport::decode(
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
    let second = RecoveryObserverReport::decode(
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
        Command::new(env!("CARGO_BIN_EXE_physical_store_offline_observer"))
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
        RecoveryObserverReport::decode(&wrong_family),
        Err(RecoveryObserverDecodeDenial::WrongProtocolFamily)
    );

    let version_offset = 8 + RECOVERY_OBSERVER_REPORT_PROTOCOL.as_str().len();
    let mut retired = encoded.clone();
    retired[version_offset..version_offset + 4].copy_from_slice(&0_u32.to_le_bytes());
    refresh_digest(&mut retired);
    assert_eq!(
        RecoveryObserverReport::decode(&retired),
        Err(RecoveryObserverDecodeDenial::Malformed)
    );

    let mut future = encoded.clone();
    future[version_offset..version_offset + 4].copy_from_slice(&2_u32.to_le_bytes());
    refresh_digest(&mut future);
    assert!(matches!(
        RecoveryObserverReport::decode(&future),
        Err(RecoveryObserverDecodeDenial::UnsupportedVersion(_))
    ));

    let mut tampered = encoded.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert_eq!(
        RecoveryObserverReport::decode(&tampered),
        Err(RecoveryObserverDecodeDenial::DigestMismatch)
    );
    assert_eq!(
        RecoveryObserverReport::decode(&encoded[..31]),
        Err(RecoveryObserverDecodeDenial::Malformed)
    );

    let payload_end = encoded.len() - 32;
    let header_end = 8 + RECOVERY_OBSERVER_REPORT_PROTOCOL.as_str().len() + 4;
    for payload_length in header_end..payload_end {
        let truncated = rehashed_prefix(&encoded, payload_length);
        assert_eq!(
            RecoveryObserverReport::decode(&truncated),
            Err(RecoveryObserverDecodeDenial::Malformed),
            "field-boundary truncation at {payload_length}"
        );
    }

    let mut trailing = encoded[..payload_end].to_vec();
    trailing.push(0xaa);
    trailing.extend_from_slice(&Sha256::digest(&trailing));
    assert_eq!(
        RecoveryObserverReport::decode(&trailing),
        Err(RecoveryObserverDecodeDenial::Malformed)
    );

    let selectors_start = header_end + 8 + 8 + 32 + 40 + 40;
    for optional_offset in [selectors_start + 24, selectors_start + 25] {
        let mut malformed_optional = encoded.clone();
        malformed_optional[optional_offset] = 2;
        refresh_digest(&mut malformed_optional);
        assert_eq!(
            RecoveryObserverReport::decode(&malformed_optional),
            Err(RecoveryObserverDecodeDenial::Malformed),
            "invalid optional flag at {optional_offset}"
        );
    }
}

#[test]
fn observer_v1_literal_fixture_rejects_every_optional_flag_and_every_truncation() {
    let (encoded, optional_flags) = literal_v1_report();
    RecoveryObserverReport::decode(&encoded).expect("literal version-one report");
    let valid_payload_end = encoded.len() - 32;
    for length in 0..encoded.len() {
        if length == valid_payload_end {
            continue;
        }
        assert_eq!(
            RecoveryObserverReport::decode(&rehashed_prefix(&encoded, length)),
            Err(RecoveryObserverDecodeDenial::Malformed),
            "literal v1 truncation at {length}"
        );
    }
    for offset in optional_flags {
        let mut malformed = encoded.clone();
        malformed[offset] = 2;
        refresh_digest(&mut malformed);
        assert_eq!(
            RecoveryObserverReport::decode(&malformed),
            Err(RecoveryObserverDecodeDenial::Malformed),
            "literal v1 optional flag at {offset}"
        );
    }
}

fn refresh_digest(bytes: &mut [u8]) {
    let split = bytes.len() - 32;
    let digest: [u8; 32] = Sha256::digest(&bytes[..split]).into();
    bytes[split..].copy_from_slice(&digest);
}

fn rehashed_prefix(encoded: &[u8], payload_length: usize) -> Vec<u8> {
    let mut truncated = encoded[..payload_length].to_vec();
    let digest: [u8; 32] = Sha256::digest(&truncated).into();
    truncated.extend_from_slice(&digest);
    truncated
}

fn literal_v1_report() -> (Vec<u8>, Vec<usize>) {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &(RECOVERY_OBSERVER_REPORT_PROTOCOL.as_str().len() as u64).to_le_bytes(),
    );
    bytes.extend_from_slice(RECOVERY_OBSERVER_REPORT_PROTOCOL.as_str().as_bytes());
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&[0; 32]);
    push_digest(&mut bytes);
    push_digest(&mut bytes);
    for _ in 0..3 {
        bytes.extend_from_slice(&0_u64.to_le_bytes());
    }
    let mut optional_flags = Vec::new();
    push_optional_array_flag(&mut bytes, &mut optional_flags);
    push_optional_u64_flag(&mut bytes, &mut optional_flags);
    push_raw_digest(&mut bytes);
    for _ in 0..2 {
        bytes.extend_from_slice(&0_u64.to_le_bytes());
    }
    for _ in 0..4 {
        push_optional_u64_flag(&mut bytes, &mut optional_flags);
    }
    push_raw_digest(&mut bytes);
    for _ in 0..4 {
        bytes.extend_from_slice(&0_u64.to_le_bytes());
    }
    for _ in 0..2 {
        push_optional_u64_flag(&mut bytes, &mut optional_flags);
    }
    push_raw_digest(&mut bytes);
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    for _ in 0..2 {
        push_optional_u64_flag(&mut bytes, &mut optional_flags);
    }
    push_raw_digest(&mut bytes);
    for _ in 0..2 {
        bytes.extend_from_slice(&0_u64.to_le_bytes());
    }
    push_raw_digest(&mut bytes);
    for _ in 0..2 {
        bytes.extend_from_slice(&0_u64.to_le_bytes());
    }
    push_raw_digest(&mut bytes);
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&digest);
    (bytes, optional_flags)
}

fn push_digest(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&[0; 32]);
}

fn push_raw_digest(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&[0; 32]);
}

fn push_optional_u64_flag(bytes: &mut Vec<u8>, offsets: &mut Vec<usize>) {
    offsets.push(bytes.len());
    bytes.push(0);
}

fn push_optional_array_flag(bytes: &mut Vec<u8>, offsets: &mut Vec<usize>) {
    offsets.push(bytes.len());
    bytes.push(0);
}

fn run_observer(root: &std::path::Path, output: &std::path::Path, maximum_bytes: u64) {
    let maximum_bytes = maximum_bytes.to_string();
    let process = run_bounded(
        Command::new(env!("CARGO_BIN_EXE_physical_store_offline_observer"))
            .arg("c8-recovery-observe")
            .arg(root)
            .arg(output)
            .args(["2", "1", "1", maximum_bytes.as_str()]),
    );
    assert!(
        process.status.success(),
        "observer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr),
    );
}

fn run_bounded(command: &mut Command) -> Output {
    let mut child = command.spawn().expect("launch shipped offline observer");
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().expect("collect observer output"),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("shipped offline observer exceeded its bounded wait");
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("poll shipped offline observer: {error}");
            }
        }
    }
}

fn selector_bytes(
    identity: u64,
    linked: u64,
    root_generation: u64,
    linked_generation: u64,
) -> Vec<u8> {
    let proposed = ProposedStoreIdentity::from_nonzero_bytes([7; 16]).expect("store identity");
    let store = StoreNamespaceIdentityRecord::new(StoreNamespaceVersion::CURRENT, proposed)
        .published_identity();
    let format = PhysicalRecordFormatDeclaration::builder()
        .admit()
        .expect("physical format");
    DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(identity).expect("selector identity"),
        RootSelectorRole::Current,
        root_generation,
        RootSelectorIdentity::new(linked),
        Some(linked_generation),
    )
    .expect("durable selector")
    .encode()
    .to_vec()
}
