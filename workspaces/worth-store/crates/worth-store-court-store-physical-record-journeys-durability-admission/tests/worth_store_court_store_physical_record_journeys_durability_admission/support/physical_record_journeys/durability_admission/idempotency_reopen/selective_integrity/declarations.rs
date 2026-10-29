pub(crate) use {std::fs};

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::PhysicalBindingCompactionReopenFailure, worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure, worth_store::physical_runtime::PhysicalIdempotencyReopenFailure, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalSignalConstructionFailure, worth_store::physical_runtime::RecordBootstrapFailure};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
pub(crate) use {crate::durability_admission::idempotency_reopen::support::checkpoint_records, crate::durability_admission::idempotency_reopen::support::inspect_checkpoint_reopen, crate::durability_admission::idempotency_reopen::support::prepare, crate::durability_admission::idempotency_reopen::support::reseal_record_crc, crate::durability_admission::idempotency_reopen::support::success_checkpoint};

pub(crate) fn binding_fingerprint_range(record: &[u8]) -> std::ops::Range<usize> {
    let mut offset = 16;
    skip_length_prefixed_field(record, &mut offset); // compaction record domain
    offset += 1; // binding state
    for _ in 0..3 {
        skip_length_prefixed_field(record, &mut offset); // key, store, policy
    }
    offset += 16; // issuance and expiry generations
    skip_length_prefixed_field(record, &mut offset); // caller material
    length_prefixed_field(record, &mut offset)
}

pub(crate) fn skip_length_prefixed_field(record: &[u8], offset: &mut usize) {
    let _ = length_prefixed_field(record, offset);
}

pub(crate) fn length_prefixed_field(record: &[u8], offset: &mut usize) -> std::ops::Range<usize> {
    let length = u64::from_le_bytes(record[*offset..*offset + 8].try_into().unwrap()) as usize;
    let start = *offset + 8;
    let end = start + length;
    assert!(end <= record.len() - 4);
    *offset = end;
    start..end
}

pub(crate) fn inject_dirty_basis(root: &std::path::Path) {
    const HEADER_BYTES: usize = 164;
    const DIRTY_BYTES: usize = 68;
    const FOOTER_BYTES: usize = 204;
    let mut dirty = vec![0_u8; DIRTY_BYTES];
    dirty[..8].copy_from_slice(b"WCP7REC\0");
    dirty[8] = 1;
    dirty[9] = 2;
    dirty[12..16].copy_from_slice(&48_u32.to_le_bytes());
    dirty[16] = 1;
    dirty[48..52].copy_from_slice(&1_u32.to_le_bytes());
    dirty[56..64].copy_from_slice(&1_u64.to_le_bytes());
    reseal_record_crc(&mut dirty);

    let path = root.join("families/checkpoint.current");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.splice(HEADER_BYTES..HEADER_BYTES, dirty.iter().copied());
    let footer_offset = bytes.len() - FOOTER_BYTES;
    bytes[footer_offset + 40..footer_offset + 48].copy_from_slice(&1_u64.to_le_bytes());
    bytes[footer_offset + 48..footer_offset + 80]
        .copy_from_slice(&<[u8; 32]>::from(sha2::Sha256::digest(&dirty)));
    bytes[footer_offset + 80..footer_offset + 88]
        .copy_from_slice(&((HEADER_BYTES + DIRTY_BYTES) as u64).to_le_bytes());
    reseal_record_crc(&mut bytes[footer_offset..]);
    std::fs::write(path, bytes).unwrap();
}

pub(crate) fn create_checkpoint_with_binding(root: &std::path::Path, seed: u8) {
    let serving = serving_from_initialization(root);
    add_binding_and_checkpoint(&serving, seed);
    serving.close();
}

pub(crate) fn create_checkpoint_with_unsealed_binding(root: &std::path::Path, seed: u8) {
    let serving = serving_from_initialization(root);
    add_binding(&serving, seed);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(
            [seed.wrapping_add(1); 32],
        ))
        .unwrap();
    let prepared = prepare(&submission, placement, key, b"selective-unsealed-record");
    let checkpoint = success_checkpoint(&serving, seed.wrapping_add(2));
    assert_eq!(checkpoint.binding_compaction().binding_count(), 2);
    drop(prepared);
    serving.close();
}

pub(crate) fn add_binding_and_checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    seed: u8,
) {
    add_binding(serving, seed);
    let checkpoint = success_checkpoint(serving, seed.wrapping_add(1));
    assert!(checkpoint.binding_compaction().binding_count() > 0);
}

pub(crate) fn add_binding(serving: &worth_store::physical_runtime::ServingPhysicalRuntime, seed: u8) {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([seed; 32]))
        .unwrap();
    let prepared = prepare(&submission, placement, key, b"selective-checkpoint-record");
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
}

pub(crate) fn record_location(bytes: &[u8], kind: u8) -> (usize, usize) {
    let target = checkpoint_records(bytes)
        .into_iter()
        .find(|record| record[9] == kind)
        .unwrap_or_else(|| panic!("checkpoint must contain record kind {kind}"));
    let offset = target.as_ptr() as usize - bytes.as_ptr() as usize;
    (offset, target.len())
}

pub(crate) fn unsealed_binding_location(bytes: &[u8]) -> (usize, usize) {
    let target = checkpoint_records(bytes)
        .into_iter()
        .filter(|record| record[9] == 4)
        .find(|record| {
            let mut offset = 16;
            skip_length_prefixed_field(record, &mut offset);
            record[offset] == 1
        })
        .expect("checkpoint must contain the intentionally unresolved binding");
    let offset = target.as_ptr() as usize - bytes.as_ptr() as usize;
    (offset, target.len())
}