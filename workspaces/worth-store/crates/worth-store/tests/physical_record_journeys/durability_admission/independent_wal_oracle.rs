use std::ops::Range;

const FRAME_HEADER_BYTES: usize = 116;
const FRAME_FOOTER_BYTES: usize = 32;
const BINDING_DOMAIN: &[u8] = b"store.physical.mutation-attempt-binding.v1";

#[path = "independent_wal_oracle/canonical_redo.rs"]
mod canonical_redo;
#[path = "independent_wal_oracle/retirement_redo.rs"]
mod retirement_redo;
#[path = "independent_wal_oracle/rewrite_redo.rs"]
mod rewrite_redo;
#[path = "independent_wal_oracle/segment_inventory.rs"]
mod segment_inventory;
#[path = "independent_wal_oracle/target_claim.rs"]
mod target_claim;

pub(crate) use retirement_redo::{
    file_retirement_payloads, produced_retirement_payloads, IndependentRetiredKind,
    IndependentRetirementAction,
};
pub(crate) use rewrite_redo::produced_rewrite_payloads;
pub(super) use segment_inventory::inspect_wal_inventory;
pub(super) use target_claim::{independent_target_claim, IndependentRedoTargetClaim};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExpectedAttemptBinding {
    pub(super) key: [u8; 32],
    pub(super) lease_store: [u8; 16],
    pub(super) policy: [u8; 32],
    pub(super) issuance: u64,
    pub(super) expiry: u64,
    pub(super) material: [u8; 32],
    pub(super) fingerprint: [u8; 32],
    pub(super) store: [u8; 16],
    pub(super) runtime: u64,
    pub(super) lifecycle_generation: u64,
    pub(super) operation: u64,
    pub(super) group: [u8; 32],
    pub(super) ordinal: u32,
    pub(super) member_count: u32,
    pub(super) membership: [u8; 32],
    pub(super) member: [u8; 32],
    pub(super) lsn_start: u64,
    pub(super) lsn_end: u64,
    pub(super) redo_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BindingField {
    Domain,
    Key,
    LeaseStore,
    Policy,
    Issuance,
    Expiry,
    Material,
    Fingerprint,
    Store,
    Runtime,
    LifecycleGeneration,
    Operation,
    Group,
    Ordinal,
    MemberCount,
    Membership,
    Member,
    LsnStart,
    LsnEnd,
    RedoDigest,
    RedoPayload,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum BindingInspectionDenial {
    InvalidFrame,
    Truncated,
    InvalidFieldLength(BindingField),
    DomainMismatch,
    FieldMismatch(BindingField),
    TrailingBytes,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct InspectedAttemptBinding {
    pub(super) value: ExpectedAttemptBinding,
    pub(super) spans: Vec<(BindingField, Range<usize>)>,
}

impl InspectedAttemptBinding {
    pub(super) fn span(&self, field: BindingField) -> Range<usize> {
        self.spans
            .iter()
            .find(|(candidate, _)| *candidate == field)
            .map(|(_, span)| span.clone())
            .expect("every binding field has one span")
    }
}

pub(super) fn independent_frame_payload(bytes: &[u8]) -> Result<&[u8], BindingInspectionDenial> {
    if bytes.len() < FRAME_HEADER_BYTES + FRAME_FOOTER_BYTES
        || bytes.get(..8) != Some(b"WORTHWAL".as_slice())
        || read_u16_at(bytes, 8)? != 1
        || usize::from(read_u16_at(bytes, 10)?) != FRAME_HEADER_BYTES
    {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    let payload_bytes = usize::try_from(read_u64_at(bytes, 44)?)
        .map_err(|_| BindingInspectionDenial::InvalidFrame)?;
    let payload_end = FRAME_HEADER_BYTES
        .checked_add(payload_bytes)
        .ok_or(BindingInspectionDenial::InvalidFrame)?;
    if payload_end
        .checked_add(FRAME_FOOTER_BYTES)
        .filter(|end| *end == bytes.len())
        .is_none()
    {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    Ok(&bytes[FRAME_HEADER_BYTES..payload_end])
}

pub(super) fn independent_canonical_redo(
    records: &[&[u8]],
    lsn_start: u64,
    targets: &[Vec<IndependentRedoTargetClaim>],
    projection: &[u8],
) -> Vec<u8> {
    canonical_redo::independent_canonical_redo(records, lsn_start, targets, projection)
}

pub(super) fn independent_recovery_projection(
    canonical_redo: &[u8],
) -> Result<&[u8], BindingInspectionDenial> {
    canonical_redo::independent_recovery_projection(canonical_redo)
}

pub(super) fn split_member_payload(
    payload: &[u8],
) -> Result<(&[u8], &[u8]), BindingInspectionDenial> {
    let mut cursor = ByteCursor::new(payload);
    let binding = cursor.field(BindingField::Domain)?;
    let redo = cursor.field(BindingField::RedoDigest)?;
    cursor.finish()?;
    Ok((binding, redo))
}

pub(super) fn inspect_member_payload(
    payload: &[u8],
    expected: &ExpectedAttemptBinding,
    expected_redo: &[u8],
) -> Result<InspectedAttemptBinding, BindingInspectionDenial> {
    let (binding, redo) = split_member_payload(payload)?;
    if redo != expected_redo {
        return Err(BindingInspectionDenial::FieldMismatch(
            BindingField::RedoPayload,
        ));
    }
    inspect_attempt_binding(binding, expected)
}

pub(super) fn inspect_attempt_binding(
    bytes: &[u8],
    expected: &ExpectedAttemptBinding,
) -> Result<InspectedAttemptBinding, BindingInspectionDenial> {
    let mut cursor = ByteCursor::new(bytes);
    let domain = cursor.fixed_field(BindingField::Domain, BINDING_DOMAIN.len())?;
    if domain != BINDING_DOMAIN {
        return Err(BindingInspectionDenial::DomainMismatch);
    }
    let key = array(cursor.fixed_field(BindingField::Key, 32)?);
    let lease_store = array(cursor.fixed_field(BindingField::LeaseStore, 16)?);
    let policy = array(cursor.fixed_field(BindingField::Policy, 32)?);
    let issuance = cursor.u64(BindingField::Issuance)?;
    let expiry = cursor.u64(BindingField::Expiry)?;
    let material = array(cursor.fixed_field(BindingField::Material, 32)?);
    let fingerprint = array(cursor.fixed_field(BindingField::Fingerprint, 32)?);
    let store = array(cursor.fixed_field(BindingField::Store, 16)?);
    let runtime = cursor.u64(BindingField::Runtime)?;
    let lifecycle_generation = cursor.u64(BindingField::LifecycleGeneration)?;
    let operation = cursor.u64(BindingField::Operation)?;
    let group = array(cursor.fixed_field(BindingField::Group, 32)?);
    let ordinal = cursor.u32(BindingField::Ordinal)?;
    let member_count = cursor.u32(BindingField::MemberCount)?;
    let membership = array(cursor.fixed_field(BindingField::Membership, 32)?);
    let member = array(cursor.fixed_field(BindingField::Member, 32)?);
    let lsn_start = cursor.u64(BindingField::LsnStart)?;
    let lsn_end = cursor.u64(BindingField::LsnEnd)?;
    let redo_digest = array(cursor.fixed_field(BindingField::RedoDigest, 32)?);
    cursor.finish()?;
    let value = ExpectedAttemptBinding {
        key,
        lease_store,
        policy,
        issuance,
        expiry,
        material,
        fingerprint,
        store,
        runtime,
        lifecycle_generation,
        operation,
        group,
        ordinal,
        member_count,
        membership,
        member,
        lsn_start,
        lsn_end,
        redo_digest,
    };
    require_expected_fields(&value, expected)?;
    Ok(InspectedAttemptBinding {
        value,
        spans: cursor.spans,
    })
}

fn require_expected_fields(
    value: &ExpectedAttemptBinding,
    expected: &ExpectedAttemptBinding,
) -> Result<(), BindingInspectionDenial> {
    for (field, equal) in [
        (BindingField::Key, value.key == expected.key),
        (
            BindingField::LeaseStore,
            value.lease_store == expected.lease_store,
        ),
        (BindingField::Policy, value.policy == expected.policy),
        (BindingField::Issuance, value.issuance == expected.issuance),
        (BindingField::Expiry, value.expiry == expected.expiry),
        (BindingField::Material, value.material == expected.material),
        (
            BindingField::Fingerprint,
            value.fingerprint == expected.fingerprint,
        ),
        (BindingField::Store, value.store == expected.store),
        (BindingField::Runtime, value.runtime == expected.runtime),
        (
            BindingField::LifecycleGeneration,
            value.lifecycle_generation == expected.lifecycle_generation,
        ),
        (
            BindingField::Operation,
            value.operation == expected.operation,
        ),
        (BindingField::Group, value.group == expected.group),
        (BindingField::Ordinal, value.ordinal == expected.ordinal),
        (
            BindingField::MemberCount,
            value.member_count == expected.member_count,
        ),
        (
            BindingField::Membership,
            value.membership == expected.membership,
        ),
        (BindingField::Member, value.member == expected.member),
        (
            BindingField::LsnStart,
            value.lsn_start == expected.lsn_start,
        ),
        (BindingField::LsnEnd, value.lsn_end == expected.lsn_end),
        (
            BindingField::RedoDigest,
            value.redo_digest == expected.redo_digest,
        ),
    ] {
        if !equal {
            return Err(BindingInspectionDenial::FieldMismatch(field));
        }
    }
    Ok(())
}

struct ByteCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
    spans: Vec<(BindingField, Range<usize>)>,
}

impl<'a> ByteCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            spans: Vec::new(),
        }
    }

    fn field(&mut self, field: BindingField) -> Result<&'a [u8], BindingInspectionDenial> {
        let length = usize::try_from(self.read_u64()?)
            .map_err(|_| BindingInspectionDenial::InvalidFieldLength(field))?;
        self.take(field, length)
    }

    fn fixed_field(
        &mut self,
        field: BindingField,
        expected: usize,
    ) -> Result<&'a [u8], BindingInspectionDenial> {
        let length = usize::try_from(self.read_u64()?)
            .map_err(|_| BindingInspectionDenial::InvalidFieldLength(field))?;
        if length != expected {
            return Err(BindingInspectionDenial::InvalidFieldLength(field));
        }
        self.take(field, length)
    }

    fn u64(&mut self, field: BindingField) -> Result<u64, BindingInspectionDenial> {
        let bytes = self.take(field, 8)?;
        Ok(u64::from_le_bytes(bytes.try_into().expect("fixed u64")))
    }

    fn u32(&mut self, field: BindingField) -> Result<u32, BindingInspectionDenial> {
        let bytes = self.take(field, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().expect("fixed u32")))
    }

    fn read_u64(&mut self) -> Result<u64, BindingInspectionDenial> {
        let bytes = self
            .bytes
            .get(self.offset..self.offset + 8)
            .ok_or(BindingInspectionDenial::Truncated)?;
        self.offset += 8;
        Ok(u64::from_le_bytes(bytes.try_into().expect("fixed u64")))
    }

    fn take(
        &mut self,
        field: BindingField,
        length: usize,
    ) -> Result<&'a [u8], BindingInspectionDenial> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(BindingInspectionDenial::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(BindingInspectionDenial::Truncated)?;
        self.spans.push((field, self.offset..end));
        self.offset = end;
        Ok(value)
    }

    fn finish(&self) -> Result<(), BindingInspectionDenial> {
        (self.offset == self.bytes.len())
            .then_some(())
            .ok_or(BindingInspectionDenial::TrailingBytes)
    }
}

fn read_u16_at(bytes: &[u8], offset: usize) -> Result<u16, BindingInspectionDenial> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or(BindingInspectionDenial::Truncated)?;
    Ok(u16::from_le_bytes(value.try_into().expect("fixed u16")))
}

fn read_u64_at(bytes: &[u8], offset: usize) -> Result<u64, BindingInspectionDenial> {
    let value = bytes
        .get(offset..offset + 8)
        .ok_or(BindingInspectionDenial::Truncated)?;
    Ok(u64::from_le_bytes(value.try_into().expect("fixed u64")))
}

fn array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    bytes.try_into().expect("fixed field length was checked")
}
