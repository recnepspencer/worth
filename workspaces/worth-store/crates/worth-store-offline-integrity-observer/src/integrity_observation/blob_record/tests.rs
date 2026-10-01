use super::super::sha256::sha256;
use super::*;

fn independently_encoded_chunk() -> Vec<u8> {
    let data = vec![9_u8; 64 << 10];
    let mut content = Vec::new();
    content.extend_from_slice(&[1, 0, 0, 0]);
    content.extend_from_slice(&(64_u32 << 10).to_le_bytes());
    content.extend_from_slice(&(data.len() as u32).to_le_bytes());
    content.extend_from_slice(&data);
    let mut payload = Vec::new();
    payload.extend_from_slice(&[1; 16]);
    payload.extend_from_slice(&[2; 16]);
    payload.extend_from_slice(&0_u64.to_le_bytes());
    payload.extend_from_slice(&(64_u32 << 10).to_le_bytes());
    payload.extend_from_slice(&(data.len() as u32).to_le_bytes());
    payload.extend_from_slice(&sha256(&content));
    payload.extend_from_slice(&content);
    let mut frame = Vec::new();
    frame.extend_from_slice(MAGIC);
    frame.extend_from_slice(&[2, 1]);
    frame.extend_from_slice(&1_u16.to_le_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&[0; 32]);
    frame.extend_from_slice(&payload);
    rehash(&mut frame);
    frame
}

fn rehash(frame: &mut [u8]) {
    let mut hasher = Sha256::new();
    hasher.update(&frame[..16]);
    hasher.update(&frame[48..]);
    frame[16..48].copy_from_slice(&hasher.finish());
}

#[test]
fn independent_chunk_reader_requires_occurrence_even_with_valid_outer_hash() {
    let mut frame = independently_encoded_chunk();
    let mut counters = OfflineIntegrityObservationCounters::default();
    let fact = decode(&frame, Some([1; 16]), &mut counters).expect("independent valid chunk");
    assert!(matches!(fact, BlobFact::Chunk { ordinal: 0, .. }));
    assert_eq!(counters.checksum_calculations, 2);
    frame[10..12].copy_from_slice(&0_u16.to_le_bytes());
    rehash(&mut frame);
    assert!(matches!(
        decode(&frame, Some([1; 16]), &mut counters),
        Err(Outcome::Damaged(_))
    ));
}

#[test]
fn independently_valid_inner_frame_still_rejects_wrong_store_scope() {
    let frame = independently_encoded_chunk();
    let mut counters = OfflineIntegrityObservationCounters::default();
    assert!(matches!(
        decode(&frame, Some([3; 16]), &mut counters),
        Err(Outcome::Damaged(_))
    ));
}

#[test]
fn independent_frontier_control_frame_preserves_exact_claim_fields() {
    let mut payload = Vec::with_capacity(160);
    payload.extend_from_slice(&[1; 16]);
    payload.extend_from_slice(&[2; 16]);
    payload.extend_from_slice(&[3; 16]);
    payload.extend_from_slice(&4_u64.to_le_bytes());
    payload.extend_from_slice(&[5; 32]);
    payload.extend_from_slice(&2_u64.to_le_bytes());
    payload.extend_from_slice(&(128_u64 << 10).to_le_bytes());
    payload.extend_from_slice(&[6; 16]);
    payload.extend_from_slice(&7_u64.to_le_bytes());
    payload.extend_from_slice(&[8; 32]);
    assert_eq!(payload.len(), 160);
    let mut frame = Vec::with_capacity(208);
    frame.extend_from_slice(MAGIC);
    frame.extend_from_slice(&[5, 1]);
    frame.extend_from_slice(&0_u16.to_le_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&[0; 32]);
    frame.extend_from_slice(&payload);
    rehash(&mut frame);
    let mut counters = OfflineIntegrityObservationCounters::default();
    let fact = decode(&frame, Some([1; 16]), &mut counters).expect("independent frontier");
    assert!(matches!(
        fact,
        BlobFact::Frontier {
            next_chunk_ordinal: 2,
            durable_bytes: 131_072,
            declaration_digest,
            last_chunk_digest,
            ..
        } if declaration_digest == [5; 32] && last_chunk_digest == [8; 32]
    ));
    frame[10..12].copy_from_slice(&1_u16.to_le_bytes());
    rehash(&mut frame);
    assert!(matches!(
        decode(&frame, Some([1; 16]), &mut counters),
        Err(Outcome::Damaged(_))
    ));
}

#[test]
fn independent_drop_manifest_checks_sorted_ids_and_domain_digest() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[1; 16]); // store
    payload.extend_from_slice(&[2; 16]); // attempt
    payload.extend_from_slice(&[3; 16]); // session
    payload.extend_from_slice(&[4; 16]);
    payload.extend_from_slice(&1_u64.to_le_bytes()); // declaration
    payload.extend_from_slice(&[5; 32]);
    payload.extend_from_slice(&[6; 16]);
    payload.extend_from_slice(&2_u64.to_le_bytes()); // abandoned
    payload.extend_from_slice(&[7; 32]);
    payload.extend_from_slice(&2_u16.to_le_bytes());
    let mut records = Vec::new();
    records.extend_from_slice(&[8; 16]);
    records.extend_from_slice(&3_u64.to_le_bytes());
    records.extend_from_slice(&[8; 16]);
    records.extend_from_slice(&4_u64.to_le_bytes());
    let mut hash = Sha256::new();
    hash.update(b"store.physical.blob-drop-set-identities.v1");
    hash.update(&2_u16.to_le_bytes());
    hash.update(&records);
    payload.extend_from_slice(&hash.finish());
    payload.extend_from_slice(&records);
    assert_eq!(payload.len(), 194 + 48);
    let mut frame = Vec::new();
    frame.extend_from_slice(MAGIC);
    frame.extend_from_slice(&[7, 1]);
    frame.extend_from_slice(&0_u16.to_le_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&[0; 32]);
    frame.extend_from_slice(&payload);
    rehash(&mut frame);
    let mut counters = OfflineIntegrityObservationCounters::default();
    assert!(matches!(
        decode(&frame, Some([1; 16]), &mut counters),
        Ok(BlobFact::DropSetManifest { dropped, .. }) if dropped.len() == 2
    ));
    // A valid outer frame cannot make an unsorted manifest authoritative.
    let first_record = frame[48 + 194..48 + 194 + 24].to_vec();
    frame[48 + 194 + 24..48 + 194 + 48].copy_from_slice(&first_record);
    let mut resealed_inner = Sha256::new();
    resealed_inner.update(b"store.physical.blob-drop-set-identities.v1");
    resealed_inner.update(&2_u16.to_le_bytes());
    resealed_inner.update(&frame[48 + 194..48 + 194 + 48]);
    frame[48 + 162..48 + 194].copy_from_slice(&resealed_inner.finish());
    rehash(&mut frame);
    assert!(matches!(
        decode(&frame, Some([1; 16]), &mut counters),
        Err(Outcome::Damaged(_))
    ));
}

fn control_frame(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(HEADER + payload.len());
    frame.extend_from_slice(MAGIC);
    frame.extend_from_slice(&[kind, 1]);
    frame.extend_from_slice(&0_u16.to_le_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&[0; 32]);
    frame.extend_from_slice(payload);
    rehash(&mut frame);
    frame
}

fn independent_v2_manifest_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[1; 16]);
    payload.extend_from_slice(&[2; 16]);
    payload.extend_from_slice(&[3; 16]);
    payload.extend_from_slice(&[4; 16]);
    payload.extend_from_slice(&1_u64.to_le_bytes());
    payload.extend_from_slice(&[5; 32]);
    payload.extend_from_slice(&[6; 16]);
    payload.extend_from_slice(&2_u64.to_le_bytes());
    payload.extend_from_slice(&[7; 32]);
    payload.extend_from_slice(&1_u16.to_le_bytes());
    let mut dropped = [8; 24];
    dropped[16..24].copy_from_slice(&3_u64.to_le_bytes());
    let mut hash = Sha256::new();
    hash.update(b"store.physical.blob-drop-set-identities.v1");
    hash.update(&1_u16.to_le_bytes());
    hash.update(&dropped);
    payload.extend_from_slice(&hash.finish());
    payload.extend_from_slice(&dropped);
    payload.extend_from_slice(&3_u64.to_le_bytes());
    payload.push(1);
    payload
}

#[test]
fn independent_v2_manifest_parser_preserves_initial_slot_without_promoting_it() {
    let mut frame = control_frame(9, &independent_v2_manifest_payload());
    let mut counters = OfflineIntegrityObservationCounters::default();
    assert!(matches!(
        decode(&frame, Some([1; 16]), &mut counters),
        Ok(BlobFact::DropSetManifest {
            never_reserved_slot_generation: Some(3),
            ..
        })
    ));
    let slot = frame.len() - 9;
    frame[slot..slot + 8].copy_from_slice(&0_u64.to_le_bytes());
    rehash(&mut frame);
    assert!(matches!(
        decode(&frame, Some([1; 16]), &mut counters),
        Err(Outcome::Damaged(_))
    ));
}

#[test]
fn independent_original_drop_reserved_parser_requires_exact_bound_fields() {
    let mut payload = Vec::with_capacity(216);
    payload.extend_from_slice(&[1; 16]);
    payload.extend_from_slice(&[2; 16]);
    payload.extend_from_slice(&[9; 16]);
    payload.extend_from_slice(&4_u64.to_le_bytes());
    payload.extend_from_slice(&[10; 32]);
    payload.extend_from_slice(&[11; 32]);
    payload.extend_from_slice(&3_u64.to_le_bytes());
    payload.extend_from_slice(&4_u64.to_le_bytes());
    payload.extend_from_slice(&[12; 32]);
    payload.extend_from_slice(&[13; 32]);
    payload.extend_from_slice(&3_u64.to_le_bytes());
    payload.extend_from_slice(&7_u64.to_le_bytes());
    assert_eq!(payload.len(), 216);
    let mut frame = control_frame(10, &payload);
    let mut counters = OfflineIntegrityObservationCounters::default();
    assert!(matches!(
        decode(&frame, Some([1; 16]), &mut counters),
        Ok(BlobFact::OriginalDropReserved {
            manifest_selected_generation: 3,
            reserved_selected_generation: 4,
            lease_expiry_generation: 7,
            ..
        })
    ));
    frame[48 + 128..48 + 136].copy_from_slice(&3_u64.to_le_bytes());
    rehash(&mut frame);
    assert!(matches!(
        decode(&frame, Some([1; 16]), &mut counters),
        Err(Outcome::Damaged(_))
    ));
}
