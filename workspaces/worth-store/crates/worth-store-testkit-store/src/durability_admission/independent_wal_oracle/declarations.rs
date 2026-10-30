pub use super::attempt_binding::{
    require_expected_fields, BindingField, BindingInspectionDenial, ExpectedAttemptBinding,
    InspectedAttemptBinding,
};

pub use std::ops::Range;

pub const FRAME_HEADER_BYTES: usize = 116;
pub const FRAME_FOOTER_BYTES: usize = 32;
pub const BINDING_DOMAIN: &[u8] = b"store.physical.mutation-attempt-binding.v1";

pub use {crate::durability_admission::independent_wal_oracle::copy_intent::produced_copy_intents};
pub use {crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::file_retirement_payloads, crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction};
pub use {crate::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::produced_rewrite_payloads};
pub use {crate::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory};
pub use {crate::durability_admission::independent_wal_oracle::target_claim::independent_target_claim, crate::durability_admission::independent_wal_oracle::target_claim::IndependentRedoTargetClaim};

// Record journeys read the full census; blob journeys share only the core.
impl crate::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::IndependentWalInventory {
    pub fn segments(&self) -> &[(u64, u64)] {
        &self.segments
    }
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }
    pub const fn peak_segment_bytes(&self) -> u64 {
        self.peak_segment_bytes
    }
}

impl crate::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::IndependentWalSegment {
    pub const fn byte_count(self) -> u64 {
        self.byte_count
    }
}

pub fn independent_frame_payload(bytes: &[u8]) -> Result<&[u8], BindingInspectionDenial> {
    if bytes.len() < FRAME_HEADER_BYTES + FRAME_FOOTER_BYTES
        || bytes.get(..8) != Some(b"WORTHWAL".as_slice())
        || read_u16_at(bytes, 8)? != 1
        || usize::from(read_u16_at(bytes, 10)?) != FRAME_HEADER_BYTES
    {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    let payload_bytes = usize::try_from(read_u64_at(bytes, 44)?).map_err(|_| {
        crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame
    })?;
    let payload_end = FRAME_HEADER_BYTES.checked_add(payload_bytes).ok_or(
        crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame,
    )?;
    if payload_end
        .checked_add(FRAME_FOOTER_BYTES)
        .filter(|end| *end == bytes.len())
        .is_none()
    {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    Ok(&bytes[FRAME_HEADER_BYTES..payload_end])
}

pub fn independent_canonical_redo(
    records: &[&[u8]],
    lsn_start: u64,
    targets: &[Vec<IndependentRedoTargetClaim>],
    projection: &[u8],
) -> Vec<u8> {
    crate::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::independent_canonical_redo(records, lsn_start, targets, projection)
}

pub fn independent_recovery_projection(
    canonical_redo: &[u8],
) -> Result<&[u8], BindingInspectionDenial> {
    crate::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::independent_recovery_projection(canonical_redo)
}

pub fn split_member_payload(payload: &[u8]) -> Result<(&[u8], &[u8]), BindingInspectionDenial> {
    let mut cursor = crate::durability_admission::independent_wal_oracle::ByteCursor::new(payload);
    let binding =
        cursor.field(crate::durability_admission::independent_wal_oracle::BindingField::Domain)?;
    let redo = cursor
        .field(crate::durability_admission::independent_wal_oracle::BindingField::RedoDigest)?;
    cursor.finish()?;
    Ok((binding, redo))
}

pub fn inspect_member_payload(
    payload: &[u8],
    expected: &ExpectedAttemptBinding,
    expected_redo: &[u8],
) -> Result<InspectedAttemptBinding, BindingInspectionDenial> {
    let (binding, redo) = split_member_payload(payload)?;
    if redo != expected_redo {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::FieldMismatch(
            crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
        ));
    }
    inspect_attempt_binding(binding, expected)
}

pub fn inspect_attempt_binding(
    bytes: &[u8],
    expected: &ExpectedAttemptBinding,
) -> Result<InspectedAttemptBinding, BindingInspectionDenial> {
    let mut cursor = crate::durability_admission::independent_wal_oracle::ByteCursor::new(bytes);
    let domain = cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Domain,
        BINDING_DOMAIN.len(),
    )?;
    if domain != BINDING_DOMAIN {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::DomainMismatch);
    }
    let key = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Key,
        32,
    )?);
    let lease_store = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::LeaseStore,
        16,
    )?);
    let policy = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Policy,
        32,
    )?);
    let issuance =
        cursor.u64(crate::durability_admission::independent_wal_oracle::BindingField::Issuance)?;
    let expiry =
        cursor.u64(crate::durability_admission::independent_wal_oracle::BindingField::Expiry)?;
    let material = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Material,
        32,
    )?);
    let fingerprint = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Fingerprint,
        32,
    )?);
    let store = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Store,
        16,
    )?);
    let runtime =
        cursor.u64(crate::durability_admission::independent_wal_oracle::BindingField::Runtime)?;
    let lifecycle_generation = cursor.u64(
        crate::durability_admission::independent_wal_oracle::BindingField::LifecycleGeneration,
    )?;
    let operation =
        cursor.u64(crate::durability_admission::independent_wal_oracle::BindingField::Operation)?;
    let group = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Group,
        32,
    )?);
    let ordinal =
        cursor.u32(crate::durability_admission::independent_wal_oracle::BindingField::Ordinal)?;
    let member_count = cursor
        .u32(crate::durability_admission::independent_wal_oracle::BindingField::MemberCount)?;
    let membership = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Membership,
        32,
    )?);
    let member = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::Member,
        32,
    )?);
    let lsn_start =
        cursor.u64(crate::durability_admission::independent_wal_oracle::BindingField::LsnStart)?;
    let lsn_end =
        cursor.u64(crate::durability_admission::independent_wal_oracle::BindingField::LsnEnd)?;
    let redo_digest = array(cursor.fixed_field(
        crate::durability_admission::independent_wal_oracle::BindingField::RedoDigest,
        32,
    )?);
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

pub struct ByteCursor<'a> {
    pub bytes: &'a [u8],
    pub offset: usize,
    pub spans: Vec<(BindingField, Range<usize>)>,
}

impl<'a> ByteCursor<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            spans: Vec::new(),
        }
    }

    pub fn field(&mut self, field: BindingField) -> Result<&'a [u8], BindingInspectionDenial> {
        let length = usize::try_from(self.read_u64()?)
            .map_err(|_| crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFieldLength(field))?;
        self.take(field, length)
    }

    pub fn fixed_field(
        &mut self,
        field: BindingField,
        expected: usize,
    ) -> Result<&'a [u8], BindingInspectionDenial> {
        let length = usize::try_from(self.read_u64()?)
            .map_err(|_| crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFieldLength(field))?;
        if length != expected {
            return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFieldLength(field));
        }
        self.take(field, length)
    }

    pub fn u64(&mut self, field: BindingField) -> Result<u64, BindingInspectionDenial> {
        let bytes = self.take(field, 8)?;
        Ok(u64::from_le_bytes(bytes.try_into().expect("fixed u64")))
    }

    pub fn u32(&mut self, field: BindingField) -> Result<u32, BindingInspectionDenial> {
        let bytes = self.take(field, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().expect("fixed u32")))
    }

    pub fn read_u64(&mut self) -> Result<u64, BindingInspectionDenial> {
        let bytes = self.bytes.get(self.offset..self.offset + 8).ok_or(
            crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated,
        )?;
        self.offset += 8;
        Ok(u64::from_le_bytes(bytes.try_into().expect("fixed u64")))
    }

    pub fn take(
        &mut self,
        field: BindingField,
        length: usize,
    ) -> Result<&'a [u8], BindingInspectionDenial> {
        let end = self.offset.checked_add(length).ok_or(
            crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated,
        )?;
        let value = self.bytes.get(self.offset..end).ok_or(
            crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated,
        )?;
        self.spans.push((field, self.offset..end));
        self.offset = end;
        Ok(value)
    }

    pub fn finish(&self) -> Result<(), BindingInspectionDenial> {
        (self.offset == self.bytes.len())
            .then_some(())
            .ok_or(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::TrailingBytes)
    }
}

pub fn read_u16_at(bytes: &[u8], offset: usize) -> Result<u16, BindingInspectionDenial> {
    let value = bytes.get(offset..offset + 2).ok_or(
        crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated,
    )?;
    Ok(u16::from_le_bytes(value.try_into().expect("fixed u16")))
}

pub fn read_u64_at(bytes: &[u8], offset: usize) -> Result<u64, BindingInspectionDenial> {
    let value = bytes.get(offset..offset + 8).ok_or(
        crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated,
    )?;
    Ok(u64::from_le_bytes(value.try_into().expect("fixed u64")))
}

pub fn array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    bytes.try_into().expect("fixed field length was checked")
}

#[path = "declarations/expected_fields.rs"]
mod expected_fields;
pub use expected_fields::*;
