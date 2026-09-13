use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_physical_format::PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN;

use super::super::{artifacts, schedule};
use super::{select_recovery_basis, ExpectedCanonicalRecord};

const CHECKPOINT_MAGIC: &[u8] = b"WCP7REC\0";
const CHECKPOINT_PREFIX_BYTES: usize = 16;
const CHECKPOINT_CRC_BYTES: usize = 4;
const BINDING_RECORD_KIND: u8 = 4;
const TERMINAL_STATE: u8 = 3;
const PROVEN_NO_EFFECT_CLASS: u8 = 1;
const COMPLETED_CLASS: u8 = 2;
const KEY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";
const ATTEMPT_DOMAIN: &[u8] = b"store.physical.mutation-attempt-binding.v1";

pub(crate) struct SubmittedOperationBindings {
    pub(crate) durable: BTreeMap<[u8; 32], ExpectedCanonicalRecord>,
    pub(crate) no_effect: [u8; 32],
    pub(crate) in_flight: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct LeaseBasis {
    store: [u8; 16],
    policy: [u8; 32],
    issuance: u64,
    expiry: u64,
}

struct BindingBasis {
    identity: [u8; 32],
    material: [u8; 32],
    lease: LeaseBasis,
}

struct CompletedBinding {
    basis: BindingBasis,
    record: ([u8; 16], u64),
    redo_digest: [u8; 32],
}

pub(crate) fn bind(
    root: &Path,
    seed: u64,
    payloads: &[Vec<u8>],
    no_effect_material: [u8; 32],
    in_flight_material: [u8; 32],
) -> Result<SubmittedOperationBindings, String> {
    let canonical = root
        .canonicalize()
        .map_err(|error| format!("canonicalize submitted-operation root: {error}"))?;
    let mut files = Vec::new();
    artifacts::collect_files(&canonical, &canonical, &mut files)?;
    let selected = select_recovery_basis(&files)?;
    let checkpoint = selected
        .iter()
        .find(|(_, bytes)| bytes.starts_with(CHECKPOINT_MAGIC))
        .ok_or_else(|| "submitted-operation oracle found no selected checkpoint".to_owned())?;
    let terminal_records = binding_records(&checkpoint.1, &checkpoint.0)?;

    let expected_payloads = payloads
        .iter()
        .enumerate()
        .map(|(ordinal, payload)| {
            (
                schedule::mutation_material(seed, ordinal as u64),
                payload.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut durable = BTreeMap::new();
    let mut seen_materials = BTreeSet::new();
    let mut seen_records = BTreeSet::new();
    let mut lease_profile = None;
    let mut no_effect = None;

    for record in terminal_records {
        let mut cursor = Cursor::new(record);
        cursor.require_field(PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN)?;
        if cursor.byte()? != TERMINAL_STATE {
            continue;
        }
        let basis = parse_basis(&mut cursor)?;
        require_common_lease_profile(&mut lease_profile, basis.lease)?;
        match cursor.byte()? {
            PROVEN_NO_EFFECT_CLASS => {
                cursor.byte()?;
                cursor.require_end()?;
                if basis.material != no_effect_material
                    || no_effect.replace((basis.identity, basis.lease)).is_some()
                {
                    return Err(
                        "persisted no-effect binding is missing, duplicated, or foreign".into(),
                    );
                }
            }
            COMPLETED_CLASS => {
                let completed = parse_completed(basis, &mut cursor)?;
                let payload = expected_payloads
                    .get(&completed.basis.material)
                    .ok_or_else(|| {
                        "persisted completed binding has foreign caller material".to_owned()
                    })?;
                if !seen_materials.insert(completed.basis.material)
                    || !seen_records.insert(completed.record)
                {
                    return Err("persisted completed bindings are duplicate or ambiguous".into());
                }
                durable.insert(
                    completed.basis.identity,
                    ExpectedCanonicalRecord {
                        allocation_epoch: completed.record.0,
                        ordinal: completed.record.1,
                        payload: payload.clone(),
                        redo_digest: completed.redo_digest,
                    },
                );
            }
            _ => {
                return Err("submitted-operation oracle found an unexpected terminal class".into())
            }
        }
    }

    if seen_materials.len() != expected_payloads.len() || durable.len() != expected_payloads.len() {
        return Err(format!(
            "persisted completed binding coverage is {}, expected {}",
            durable.len(),
            expected_payloads.len()
        ));
    }
    let (no_effect, no_effect_lease) =
        no_effect.ok_or_else(|| "persisted no-effect terminal is absent".to_owned())?;
    let in_flight_lease = LeaseBasis {
        store: no_effect_lease.store,
        policy: no_effect_lease.policy,
        issuance: no_effect_lease
            .issuance
            .checked_add(1)
            .ok_or_else(|| "in-flight lease issuance overflowed".to_owned())?,
        expiry: no_effect_lease
            .expiry
            .checked_add(1)
            .ok_or_else(|| "in-flight lease expiry overflowed".to_owned())?,
    };
    Ok(SubmittedOperationBindings {
        durable,
        no_effect,
        in_flight: key_identity(in_flight_lease, in_flight_material),
    })
}

fn parse_completed(
    basis: BindingBasis,
    cursor: &mut Cursor<'_>,
) -> Result<CompletedBinding, String> {
    let attempt = parse_attempt(cursor.field()?)?;
    if attempt.identity != basis.identity
        || attempt.material != basis.material
        || attempt.lease != basis.lease
    {
        return Err("completed terminal and attempt binding disagree".into());
    }
    cursor.u32()?;
    cursor.u64()?;
    if cursor.u32()? != 1 {
        return Err("submitted operation completed with a non-unit record set".into());
    }
    let allocation_epoch = cursor.array_field::<16>()?;
    let ordinal = cursor.u64()?;
    if allocation_epoch == [0; 16] || ordinal == 0 {
        return Err("submitted operation has an invalid persisted record identity".into());
    }
    for _ in 0..13 {
        cursor.u64()?;
    }
    cursor.require_end()?;
    Ok(CompletedBinding {
        basis,
        record: (allocation_epoch, ordinal),
        redo_digest: attempt.redo_digest,
    })
}

struct AttemptBinding {
    identity: [u8; 32],
    material: [u8; 32],
    lease: LeaseBasis,
    redo_digest: [u8; 32],
}

fn parse_attempt(bytes: &[u8]) -> Result<AttemptBinding, String> {
    let mut cursor = Cursor::new(bytes);
    cursor.require_field(ATTEMPT_DOMAIN)?;
    let basis = parse_basis(&mut cursor)?;
    cursor.field()?;
    cursor.u32()?;
    cursor.u32()?;
    cursor.field()?;
    cursor.field()?;
    cursor.u64()?;
    cursor.u64()?;
    let redo_digest = cursor.array_field::<32>()?;
    cursor.require_end()?;
    Ok(AttemptBinding {
        identity: basis.identity,
        material: basis.material,
        lease: basis.lease,
        redo_digest,
    })
}

fn parse_basis(cursor: &mut Cursor<'_>) -> Result<BindingBasis, String> {
    let identity = cursor.array_field::<32>()?;
    let lease = LeaseBasis {
        store: cursor.array_field::<16>()?,
        policy: cursor.array_field::<32>()?,
        issuance: cursor.u64()?,
        expiry: cursor.u64()?,
    };
    let material = cursor.array_field::<32>()?;
    cursor.array_field::<32>()?;
    if cursor.array_field::<16>()? != lease.store
        || cursor.u64()? == 0
        || cursor.u64()? == 0
        || cursor.u64()? == 0
        || lease.store == [0; 16]
        || lease.policy == [0; 32]
        || lease.issuance >= lease.expiry
        || material == [0; 32]
        || identity != key_identity(lease, material)
    {
        return Err("persisted operation binding basis is non-canonical".into());
    }
    Ok(BindingBasis {
        identity,
        material,
        lease,
    })
}

fn require_common_lease_profile(
    current: &mut Option<([u8; 16], [u8; 32], u64)>,
    found: LeaseBasis,
) -> Result<(), String> {
    let retention = found
        .expiry
        .checked_sub(found.issuance)
        .ok_or_else(|| "persisted operation lease expires before issuance".to_owned())?;
    let found = (found.store, found.policy, retention);
    match current {
        Some(current) if *current != found => {
            Err("submitted operations use conflicting lease profiles".into())
        }
        Some(_) => Ok(()),
        None => {
            *current = Some(found);
            Ok(())
        }
    }
}

fn key_identity(lease: LeaseBasis, material: [u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update((KEY_DOMAIN.len() as u64).to_le_bytes());
    digest.update(KEY_DOMAIN);
    digest.update(lease.store);
    digest.update(lease.policy);
    digest.update(lease.issuance.to_le_bytes());
    digest.update(lease.expiry.to_le_bytes());
    digest.update(material);
    digest.finalize().into()
}

fn binding_records<'bytes>(bytes: &'bytes [u8], path: &str) -> Result<Vec<&'bytes [u8]>, String> {
    let mut offset = 0;
    let mut records = Vec::new();
    while offset < bytes.len() {
        let prefix = bytes
            .get(offset..offset + CHECKPOINT_PREFIX_BYTES)
            .ok_or_else(|| {
                format!("submitted-operation oracle found a truncated prefix: {path}")
            })?;
        if &prefix[..8] != CHECKPOINT_MAGIC || prefix[8] != 1 {
            return Err(format!(
                "submitted-operation oracle found an invalid record: {path}"
            ));
        }
        let payload_bytes = usize::try_from(u32::from_le_bytes(prefix[12..16].try_into().unwrap()))
            .map_err(|_| "checkpoint payload length overflowed".to_owned())?;
        let total = CHECKPOINT_PREFIX_BYTES
            .checked_add(payload_bytes)
            .and_then(|value| value.checked_add(CHECKPOINT_CRC_BYTES))
            .ok_or_else(|| "checkpoint record length overflowed".to_owned())?;
        let record = bytes.get(offset..offset + total).ok_or_else(|| {
            format!("submitted-operation oracle found a truncated record: {path}")
        })?;
        let checksum_offset = CHECKPOINT_PREFIX_BYTES + payload_bytes;
        let expected = u32::from_le_bytes(record[checksum_offset..].try_into().unwrap());
        if crc32c(&record[..checksum_offset]) != expected {
            return Err(format!(
                "submitted-operation oracle rejected a checksum: {path}"
            ));
        }
        if prefix[9] == BINDING_RECORD_KIND {
            records.push(&record[CHECKPOINT_PREFIX_BYTES..checksum_offset]);
        }
        offset += total;
    }
    Ok(records)
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

struct Cursor<'bytes> {
    remaining: &'bytes [u8],
}

impl<'bytes> Cursor<'bytes> {
    const fn new(bytes: &'bytes [u8]) -> Self {
        Self { remaining: bytes }
    }

    fn require_field(&mut self, expected: &[u8]) -> Result<(), String> {
        (self.field()? == expected)
            .then_some(())
            .ok_or_else(|| "persisted operation binding has the wrong domain".to_owned())
    }

    fn field(&mut self) -> Result<&'bytes [u8], String> {
        let length = usize::try_from(self.u64()?)
            .map_err(|_| "persisted operation field length overflowed".to_owned())?;
        self.take(length)
    }

    fn array_field<const N: usize>(&mut self) -> Result<[u8; N], String> {
        self.field()?
            .try_into()
            .map_err(|_| "persisted operation field width mismatch".to_owned())
    }

    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn take(&mut self, length: usize) -> Result<&'bytes [u8], String> {
        let (value, remaining) = self
            .remaining
            .split_at_checked(length)
            .ok_or_else(|| "persisted operation binding is truncated".to_owned())?;
        self.remaining = remaining;
        Ok(value)
    }

    fn require_end(&self) -> Result<(), String> {
        self.remaining
            .is_empty()
            .then_some(())
            .ok_or_else(|| "persisted operation binding has trailing bytes".to_owned())
    }
}
