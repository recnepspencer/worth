pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {std::process::Command, std::process::Output};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {
    worth_store_offline_verifier::RecoveryObserverDecodeDenial,
    worth_store_offline_verifier::RecoveryObserverReport,
    worth_store_offline_verifier::RECOVERY_OBSERVER_REPORT_PROTOCOL,
    worth_store_offline_verifier::RECOVERY_OBSERVER_REPORT_VERSION,
};
pub(crate) use {
    worth_store_physical_format::store_namespace::ProposedStoreIdentity,
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord,
    worth_store_physical_format::store_namespace::StoreNamespaceVersion,
};
pub(crate) use {
    worth_store_physical_format::DurableRootSelector,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
    worth_store_physical_format::RootSelectorIdentity,
    worth_store_physical_format::RootSelectorRole,
};

pub(crate) fn refresh_digest(bytes: &mut [u8]) {
    let split = bytes.len() - 32;
    let digest: [u8; 32] = sha2::Sha256::digest(&bytes[..split]).into();
    bytes[split..].copy_from_slice(&digest);
}

pub(crate) fn rehashed_prefix(encoded: &[u8], payload_length: usize) -> Vec<u8> {
    let mut truncated = encoded[..payload_length].to_vec();
    let digest: [u8; 32] = sha2::Sha256::digest(&truncated).into();
    truncated.extend_from_slice(&digest);
    truncated
}

pub(crate) fn literal_v1_report() -> (Vec<u8>, Vec<usize>) {
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
    let digest: [u8; 32] = sha2::Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&digest);
    (bytes, optional_flags)
}

pub(crate) fn push_digest(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&[0; 32]);
}

pub(crate) fn push_raw_digest(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&[0; 32]);
}

pub(crate) fn push_optional_u64_flag(bytes: &mut Vec<u8>, offsets: &mut Vec<usize>) {
    offsets.push(bytes.len());
    bytes.push(0);
}

pub(crate) fn push_optional_array_flag(bytes: &mut Vec<u8>, offsets: &mut Vec<usize>) {
    offsets.push(bytes.len());
    bytes.push(0);
}

pub(crate) fn run_observer(root: &std::path::Path, output: &std::path::Path, maximum_bytes: u64) {
    let maximum_bytes = maximum_bytes.to_string();
    let process = run_bounded(
        std::process::Command::new(
            worth_store_testkit_executables::physical_store_offline_observer(),
        )
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

pub(crate) fn run_bounded(command: &mut Command) -> Output {
    let mut child = command.spawn().expect("launch shipped offline observer");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().expect("collect observer output"),
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(20));
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

pub(crate) fn selector_bytes(
    identity: u64,
    linked: u64,
    root_generation: u64,
    linked_generation: u64,
) -> Vec<u8> {
    let proposed =
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [7; 16],
        )
        .expect("store identity");
    let store = worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        proposed,
    )
    .published_identity();
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .expect("physical format");
    worth_store_physical_format::DurableRootSelector::new(
        store,
        format,
        worth_store_physical_format::RootSelectorIdentity::new(identity)
            .expect("selector identity"),
        worth_store_physical_format::RootSelectorRole::Current,
        root_generation,
        worth_store_physical_format::RootSelectorIdentity::new(linked),
        Some(linked_generation),
    )
    .expect("durable selector")
    .encode()
    .to_vec()
}
