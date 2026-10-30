use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedAttemptBinding {
    pub key: [u8; 32],
    pub lease_store: [u8; 16],
    pub policy: [u8; 32],
    pub issuance: u64,
    pub expiry: u64,
    pub material: [u8; 32],
    pub fingerprint: [u8; 32],
    pub store: [u8; 16],
    pub runtime: u64,
    pub lifecycle_generation: u64,
    pub operation: u64,
    pub group: [u8; 32],
    pub ordinal: u32,
    pub member_count: u32,
    pub membership: [u8; 32],
    pub member: [u8; 32],
    pub lsn_start: u64,
    pub lsn_end: u64,
    pub redo_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingField {
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
pub enum BindingInspectionDenial {
    InvalidFrame,
    Truncated,
    InvalidFieldLength(BindingField),
    DomainMismatch,
    FieldMismatch(BindingField),
    TrailingBytes,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InspectedAttemptBinding {
    pub value: ExpectedAttemptBinding,
    pub spans: Vec<(BindingField, Range<usize>)>,
}

impl InspectedAttemptBinding {
    pub fn span(&self, field: BindingField) -> Range<usize> {
        self.spans
            .iter()
            .find(|(candidate, _)| *candidate == field)
            .map(|(_, span)| span.clone())
            .expect("every binding field has one span")
    }
}

pub fn require_expected_fields(
    value: &ExpectedAttemptBinding,
    expected: &ExpectedAttemptBinding,
) -> Result<(), BindingInspectionDenial> {
    for (field, equal) in [
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Key,
            value.key == expected.key,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::LeaseStore,
            value.lease_store == expected.lease_store,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Policy,
            value.policy == expected.policy,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Issuance,
            value.issuance == expected.issuance,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Expiry,
            value.expiry == expected.expiry,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Material,
            value.material == expected.material,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Fingerprint,
            value.fingerprint == expected.fingerprint,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Store,
            value.store == expected.store,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Runtime,
            value.runtime == expected.runtime,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::LifecycleGeneration,
            value.lifecycle_generation == expected.lifecycle_generation,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Operation,
            value.operation == expected.operation,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Group,
            value.group == expected.group,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Ordinal,
            value.ordinal == expected.ordinal,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::MemberCount,
            value.member_count == expected.member_count,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Membership,
            value.membership == expected.membership,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::Member,
            value.member == expected.member,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::LsnStart,
            value.lsn_start == expected.lsn_start,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::LsnEnd,
            value.lsn_end == expected.lsn_end,
        ),
        (
            crate::durability_admission::independent_wal_oracle::BindingField::RedoDigest,
            value.redo_digest == expected.redo_digest,
        ),
    ] {
        if !equal {
            return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::FieldMismatch(field));
        }
    }
    Ok(())
}
