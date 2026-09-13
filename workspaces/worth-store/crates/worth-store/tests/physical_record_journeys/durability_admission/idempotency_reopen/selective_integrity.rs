use std::fs;

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalBindingCompactionReopenFailure, PhysicalDurabilityStateReopenFailure,
    PhysicalIdempotencyReopenFailure, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalRecordOpen, PhysicalSignalConstructionFailure, RecordBootstrapFailure,
};

use super::super::super::{configuration, durability, media, serving_from_initialization};
use super::support::{
    checkpoint_records, inspect_checkpoint_reopen, prepare, reseal_record_crc, success_checkpoint,
};

#[test]
fn checksum_valid_binding_mutation_with_stale_footer_aggregate_blocks_ordinary_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    create_checkpoint_with_unsealed_binding(&root, 211);
    let path = root.join("families/checkpoint.current");
    let mut bytes = fs::read(&path).unwrap();
    let (offset, length) = unsealed_binding_location(&bytes);
    let fingerprint = binding_fingerprint_range(&bytes[offset..offset + length]);
    bytes[offset + fingerprint.start] ^= 0x20;
    reseal_record_crc(&mut bytes[offset..offset + length]);
    fs::write(&path, bytes).unwrap();

    let media_owner = media(&root);
    let durability = durability(&media_owner);
    let (format, _, access) = configuration();
    let outcome = media_owner
        .open_record_store(PhysicalRecordOpen::new(format, access, durability))
        .into_raw();
    let TransitionOutcome::Failed(inspection) = outcome else {
        panic!("stale binding selective aggregate must reject ordinary reopen")
    };
    let cause = inspection.cause();
    assert!(
        matches!(
            cause,
            RecordBootstrapFailure::SignalConstruction(
                PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    PhysicalDurabilityStateReopenFailure::Idempotency(
                        PhysicalIdempotencyReopenFailure::Checkpoint(
                            PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch
                        )
                    )
                )
            )
        ),
        "unexpected reopen cause: {cause:?}"
    );
}

fn binding_fingerprint_range(record: &[u8]) -> std::ops::Range<usize> {
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

fn skip_length_prefixed_field(record: &[u8], offset: &mut usize) {
    let _ = length_prefixed_field(record, offset);
}

fn length_prefixed_field(record: &[u8], offset: &mut usize) -> std::ops::Range<usize> {
    let length = u64::from_le_bytes(record[*offset..*offset + 8].try_into().unwrap()) as usize;
    let start = *offset + 8;
    let end = start + length;
    assert!(end <= record.len() - 4);
    *offset = end;
    start..end
}

#[test]
fn damaged_dirty_basis_remains_selectively_skipped_during_ordinary_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    create_checkpoint_with_binding(&root, 212);
    inject_dirty_basis(&root);
    let expected = inspect_checkpoint_reopen(&root);
    assert!(expected.dirty_bytes > 0);
    let path = root.join("families/checkpoint.current");
    let mut bytes = fs::read(&path).unwrap();
    let (offset, _) = record_location(&bytes, 2);
    bytes[offset + 16] ^= 0x80;
    fs::write(&path, bytes).unwrap();

    let reopened = super::super::super::serving_from_open(&root);
    let observation = reopened
        .durability_observation()
        .reopen()
        .expect("dirty-basis bytes are outside ordinary binding-compaction meaning");
    assert_eq!(
        observation.checkpoint_artifact_bytes(),
        expected.artifact_bytes
    );
    assert_eq!(observation.checkpoint_bytes_read(), expected.bytes_read);
    assert_eq!(observation.dirty_body_bytes_skipped(), expected.dirty_bytes);
    assert_eq!(observation.binding_records_read(), expected.binding_records);
    assert_eq!(
        observation.checkpoint_integrity_admissions(),
        expected.binding_records + 3
    );
    reopened.close();
}

fn inject_dirty_basis(root: &std::path::Path) {
    const HEADER_BYTES: usize = 164;
    const DIRTY_BYTES: usize = 68;
    const FOOTER_BYTES: usize = 156;
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
    let mut bytes = fs::read(&path).unwrap();
    bytes.splice(HEADER_BYTES..HEADER_BYTES, dirty.iter().copied());
    let footer_offset = bytes.len() - FOOTER_BYTES;
    bytes[footer_offset + 40..footer_offset + 48].copy_from_slice(&1_u64.to_le_bytes());
    bytes[footer_offset + 48..footer_offset + 80]
        .copy_from_slice(&<[u8; 32]>::from(Sha256::digest(&dirty)));
    bytes[footer_offset + 80..footer_offset + 88]
        .copy_from_slice(&((HEADER_BYTES + DIRTY_BYTES) as u64).to_le_bytes());
    reseal_record_crc(&mut bytes[footer_offset..]);
    fs::write(path, bytes).unwrap();
}

fn create_checkpoint_with_binding(root: &std::path::Path, seed: u8) {
    let serving = serving_from_initialization(root);
    add_binding_and_checkpoint(&serving, seed);
    serving.close();
}

fn create_checkpoint_with_unsealed_binding(root: &std::path::Path, seed: u8) {
    let serving = serving_from_initialization(root);
    add_binding(&serving, seed);
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
            [seed.wrapping_add(1); 32],
        ))
        .unwrap();
    let prepared = prepare(&submission, placement, key, b"selective-unsealed-record");
    let checkpoint = success_checkpoint(&serving, seed.wrapping_add(2));
    assert_eq!(checkpoint.binding_compaction().binding_count(), 2);
    drop(prepared);
    serving.close();
}

fn add_binding_and_checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    seed: u8,
) {
    add_binding(serving, seed);
    let checkpoint = success_checkpoint(&serving, seed.wrapping_add(1));
    assert!(checkpoint.binding_compaction().binding_count() > 0);
}

fn add_binding(serving: &worth_store::physical_runtime::ServingPhysicalRuntime, seed: u8) {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([seed; 32]))
        .unwrap();
    let prepared = prepare(&submission, placement, key, b"selective-checkpoint-record");
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
}

fn record_location(bytes: &[u8], kind: u8) -> (usize, usize) {
    let target = checkpoint_records(bytes)
        .into_iter()
        .find(|record| record[9] == kind)
        .unwrap_or_else(|| panic!("checkpoint must contain record kind {kind}"));
    let offset = target.as_ptr() as usize - bytes.as_ptr() as usize;
    (offset, target.len())
}

fn unsealed_binding_location(bytes: &[u8]) -> (usize, usize) {
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
