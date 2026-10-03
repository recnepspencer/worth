use sha2::{Digest, Sha256};

use super::{fate_from_files, history};

#[test]
fn unselected_arena_payload_is_not_a_durable_effect() {
    let payload = b"staged-but-unselected";
    let files = vec![
        (
            "families/records/arenas/arena-0000000000000001.data".to_owned(),
            payload.to_vec(),
        ),
        (
            "families/records/roots/root-0000000000000002.manifest".to_owned(),
            payload.to_vec(),
        ),
    ];
    assert_eq!(
        fate_from_files(&files, &[7; 32], payload),
        history::InFlightMutationFate::Indeterminate
    );
    let mut selected_files = files;
    selected_files.push((
        "families/checkpoint.current".to_owned(),
        checkpoint_current(1),
    ));
    let selected = history::select_recovery_basis(&selected_files).unwrap();
    assert_eq!(
        history::classify_in_flight_artifacts(&selected, &[7; 32], payload).unwrap(),
        (false, false),
        "staged arena and candidate root are not selected fate evidence",
    );
}

#[test]
fn authenticated_canonical_redo_binding_proves_payload() {
    const KEY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";
    let payload = b"canonical-record-payload";
    let store = [1_u8; 16];
    let policy = [2_u8; 32];
    let material = [3_u8; 32];
    let group = [6_u8; 32];
    let member_identity = [8_u8; 32];
    let fingerprint = [5_u8; 32];
    let mut key = Sha256::new();
    field(&mut key, KEY_DOMAIN);
    key.update(store);
    key.update(policy);
    key.update(1_u64.to_le_bytes());
    key.update(2_u64.to_le_bytes());
    key.update(material);
    let idempotency: [u8; 32] = key.finalize().into();

    let mut redo = Vec::new();
    append_field(&mut redo, b"store.physical.wal.canonical-redo.v3");
    redo.extend_from_slice(&1_u64.to_le_bytes());
    redo.extend_from_slice(&0_u32.to_le_bytes());
    redo.extend_from_slice(&1_u64.to_le_bytes());
    redo.extend_from_slice(&1_u64.to_le_bytes());
    append_field(&mut redo, b"target");
    redo.extend_from_slice(&[4; 32]);
    append_field(&mut redo, payload);
    append_field(&mut redo, b"projection");

    let mut binding = Vec::new();
    append_field(&mut binding, b"store.physical.mutation-attempt-binding.v1");
    append_field(&mut binding, &idempotency);
    append_field(&mut binding, &store);
    append_field(&mut binding, &policy);
    binding.extend_from_slice(&1_u64.to_le_bytes());
    binding.extend_from_slice(&2_u64.to_le_bytes());
    append_field(&mut binding, &material);
    append_field(&mut binding, &fingerprint);
    append_field(&mut binding, &store);
    binding.extend_from_slice(&1_u64.to_le_bytes());
    binding.extend_from_slice(&1_u64.to_le_bytes());
    binding.extend_from_slice(&1_u64.to_le_bytes());
    append_field(&mut binding, &group);
    binding.extend_from_slice(&1_u32.to_le_bytes());
    binding.extend_from_slice(&1_u32.to_le_bytes());
    append_field(&mut binding, &[7; 32]);
    append_field(&mut binding, &member_identity);
    binding.extend_from_slice(&1_u64.to_le_bytes());
    binding.extend_from_slice(&2_u64.to_le_bytes());
    append_field(&mut binding, &Sha256::digest(&redo));

    let mut member = Vec::new();
    append_field(&mut member, &binding);
    append_field(&mut member, &redo);
    let mut wal = vec![0; 116];
    wal[..8].copy_from_slice(b"WORTHWAL");
    wal[8..10].copy_from_slice(&1_u16.to_le_bytes());
    wal[10..12].copy_from_slice(&116_u16.to_le_bytes());
    wal[12..20].copy_from_slice(&1_u64.to_le_bytes());
    wal[20..28].copy_from_slice(&1_u64.to_le_bytes());
    wal[28..36].copy_from_slice(&1_u64.to_le_bytes());
    wal[36..44].copy_from_slice(&2_u64.to_le_bytes());
    wal[44..52].copy_from_slice(&(member.len() as u64).to_le_bytes());
    let declared_identity = format!(
        "group-{}-member-{}-{}",
        hex(&group),
        hex(&member_identity),
        hex(&fingerprint)
    );
    wal[52..84].copy_from_slice(&Sha256::digest(declared_identity.as_bytes()));
    wal[84..116].copy_from_slice(&Sha256::digest(&member));
    wal.extend_from_slice(&member);
    reseal_footer(&mut wal);
    let path = "families/wal/segment-1-generation-1.wal".to_owned();
    let checkpoint = checkpoint_current(1);
    let files = vec![
        ("families/checkpoint.current".to_owned(), checkpoint.clone()),
        (path.clone(), wal.clone()),
    ];
    assert_eq!(
        fate_from_files(&files, &idempotency, payload),
        history::InFlightMutationFate::DurableEffect
    );
    let selected = history::select_recovery_basis(&files).unwrap();
    assert_eq!(
        history::classify_in_flight_artifacts(&selected, &idempotency, payload).unwrap(),
        (true, true),
        "selected canonical WAL binds the persisted idempotency to the exact payload",
    );
    assert_eq!(
        history::classify_in_flight_artifacts(&selected, &material, payload).unwrap(),
        (false, false),
        "mutation material is not the persisted idempotency key",
    );
    assert_eq!(
        fate_from_files(
            &[
                (
                    "families/checkpoint.current".to_owned(),
                    checkpoint_current_schema(1, 2),
                ),
                files[1].clone(),
            ],
            &idempotency,
            payload,
        ),
        history::InFlightMutationFate::DurableEffect,
        "maintenance checkpoint schema retains the selected WAL frontier",
    );
    assert_eq!(
        fate_from_files(
            &[
                (
                    "families/checkpoint.current".to_owned(),
                    checkpoint_current_schema(1, 3),
                ),
                files[1].clone(),
            ],
            &idempotency,
            payload,
        ),
        history::InFlightMutationFate::DurableEffect,
        "certified checkpoint schema retains the selected WAL frontier",
    );
    assert_eq!(
        fate_from_files(&files, &idempotency, b"foreign-payload"),
        history::InFlightMutationFate::Indeterminate
    );
    assert_eq!(
        fate_from_files(
            &[
                (
                    "families/checkpoint.current".to_owned(),
                    checkpoint_current(2)
                ),
                files[1].clone(),
            ],
            &idempotency,
            payload,
        ),
        history::InFlightMutationFate::Indeterminate,
        "authenticated pre-frontier WAL is not a selected durable effect"
    );
    wal[10..12].copy_from_slice(&0_u16.to_le_bytes());
    wal.truncate(wal.len() - 32);
    reseal_footer(&mut wal);
    assert_eq!(
        fate_from_files(
            &[
                ("families/checkpoint.current".to_owned(), checkpoint),
                (path, wal)
            ],
            &idempotency,
            payload
        ),
        history::InFlightMutationFate::Indeterminate
    );
}

fn checkpoint_current(frontier: u64) -> Vec<u8> {
    checkpoint_current_schema(frontier, 1)
}

fn checkpoint_current_schema(frontier: u64, schema: u8) -> Vec<u8> {
    let mut header = vec![0; 144];
    header[..16].copy_from_slice(&[9; 16]);
    header[16..24].copy_from_slice(&1_u64.to_le_bytes());
    header[32..40].copy_from_slice(&frontier.to_le_bytes());
    header[64] = 1;
    let certified = schema == 3;
    let mut footer = vec![0; if certified { 184 } else { 136 }];
    footer[..24].copy_from_slice(&header[..24]);
    footer[32..64].copy_from_slice(&Sha256::digest([]));
    footer[64..72].copy_from_slice(&164_u64.to_le_bytes());
    footer[80..88].copy_from_slice(&frontier.to_le_bytes());
    footer[104..136].copy_from_slice(&Sha256::digest([]));
    let mut compaction = vec![0; 16];
    compaction[8..16].copy_from_slice(&frontier.to_le_bytes());
    let mut bytes = record(schema, 1, &header);
    bytes.extend_from_slice(&record(schema, 3, &compaction));
    if certified {
        // One release-custody certificate record precedes the footer.
        let certificate = record(schema, 7, &[7; 24]);
        footer[136..144].copy_from_slice(&1_u64.to_le_bytes());
        footer[144..152].copy_from_slice(&(certificate.len() as u64).to_le_bytes());
        footer[152..184].copy_from_slice(&Sha256::digest(&certificate));
        bytes.extend_from_slice(&certificate);
    }
    bytes.extend_from_slice(&record(schema, 5, &footer));
    bytes
}

fn record(schema: u8, kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"WCP7REC\0");
    bytes.push(schema);
    bytes.push(kind);
    bytes.extend_from_slice(&[0; 2]);
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&crc32c(&bytes).to_le_bytes());
    bytes
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut value = !0_u32;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(value & 1);
            value = (value >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !value
}

fn reseal_footer(wal: &mut Vec<u8>) {
    let footer = Sha256::digest(&wal);
    wal.extend_from_slice(&footer);
}

fn append_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
}

fn field(target: &mut Sha256, field: &[u8]) {
    target.update((field.len() as u64).to_le_bytes());
    target.update(field);
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
