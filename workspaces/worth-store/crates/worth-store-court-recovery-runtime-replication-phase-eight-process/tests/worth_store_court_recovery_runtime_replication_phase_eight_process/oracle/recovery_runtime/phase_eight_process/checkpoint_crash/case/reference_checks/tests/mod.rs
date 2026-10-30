#[allow(unused_imports)]
use crate::oracle::support::checkpoint_recovery::tests::*;
use sha2::Digest;

use crate::oracle::support::checkpoint_recovery::fate_from_files;

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
        crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::Indeterminate
    );
    let mut selected_files = files;
    selected_files.push((
        "families/checkpoint.current".to_owned(),
        checkpoint_current(1),
    ));
    let selected =
        crate::phase_eight_process::history::parent_oracle::selected_basis::select(&selected_files)
            .unwrap();
    assert_eq!(
        crate::history::parent_oracle::in_flight::classify(&selected, &[7; 32], payload).unwrap(),
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
    let mut key = sha2::Sha256::new();
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
    append_field(&mut binding, &sha2::Sha256::digest(&redo));

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
    wal[52..84].copy_from_slice(&sha2::Sha256::digest(declared_identity.as_bytes()));
    wal[84..116].copy_from_slice(&sha2::Sha256::digest(&member));
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
        crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::DurableEffect
    );
    let selected =
        crate::phase_eight_process::history::parent_oracle::selected_basis::select(&files).unwrap();
    assert_eq!(
        crate::history::parent_oracle::in_flight::classify(&selected, &idempotency, payload)
            .unwrap(),
        (true, true),
        "selected canonical WAL binds the persisted idempotency to the exact payload",
    );
    assert_eq!(
        crate::history::parent_oracle::in_flight::classify(&selected, &material, payload).unwrap(),
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
        crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::DurableEffect,
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
        crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::DurableEffect,
        "certified checkpoint schema retains the selected WAL frontier",
    );
    assert_eq!(
        fate_from_files(&files, &idempotency, b"foreign-payload"),
        crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::Indeterminate
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
        crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::Indeterminate,
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
        crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::Indeterminate
    );
}
