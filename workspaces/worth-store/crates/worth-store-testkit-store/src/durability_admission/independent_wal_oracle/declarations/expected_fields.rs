use super::*;

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
