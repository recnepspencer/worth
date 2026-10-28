//! The complete owned relation-integrity record grammar.
use super::super::super::super::foundational_value::{decode_bool, write_bool};
use super::super::super::wire_vocabulary::{decode_optional, write_optional};
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
    record::decode_budget::RecordDecodeAttempt,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationRelationDeletionPolicy, ApplicationRelationIntegrityRecord,
    ApplicationRelationUniquenessScope,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    record: &ApplicationRelationIntegrityRecord,
) -> Result<(), Denial> {
    super::super::relation_graph::write_prefix(output, record.endpoints, record.cardinality)?;
    output.u16(match record.deletion {
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit => 1,
        ApplicationRelationDeletionPolicy::CascadeDeleteRelations => 2,
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations => 3,
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit => 4,
        ApplicationRelationDeletionPolicy::RequireRelationRetirement => 5,
    })?;
    write_bool(output, record.acyclic)?;
    write_bool(output, record.symmetric)?;
    write_bool(output, record.orbit)?;
    write_optional(output, record.unique.as_ref(), |output, scope| {
        output.u16(match scope {
            ApplicationRelationUniquenessScope::DirectedSemanticEdge => 1,
            ApplicationRelationUniquenessScope::NormalizedSymmetricEdge => 2,
        })
    })?;
    write_bool(output, record.same_partition_endpoints)?;
    write_optional(
        output,
        record.reaches_at_least.as_ref(),
        super::super::relation_graph::write_minimum,
    )
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationRelationIntegrityRecord, Denial> {
    let (endpoints, cardinality) = super::super::relation_graph::decode_prefix(input)?;
    let deletion = match input.u16()? {
        1 => ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
        2 => ApplicationRelationDeletionPolicy::CascadeDeleteRelations,
        3 => ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations,
        4 => ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
        5 => ApplicationRelationDeletionPolicy::RequireRelationRetirement,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let acyclic = decode_bool(input)?;
    let symmetric = decode_bool(input)?;
    let orbit = decode_bool(input)?;
    let unique = decode_optional(input, decode_unique_scope)?;
    let same_partition_endpoints = decode_bool(input)?;
    let reaches_at_least = decode_optional(input, |input| {
        super::super::relation_graph::decode_minimum(input, budget)
    })?;
    Ok(ApplicationRelationIntegrityRecord {
        endpoints,
        cardinality,
        deletion,
        acyclic,
        symmetric,
        orbit,
        unique,
        same_partition_endpoints,
        reaches_at_least,
    })
}

fn decode_unique_scope(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationRelationUniquenessScope, Denial> {
    match input.u16()? {
        1 => Ok(ApplicationRelationUniquenessScope::DirectedSemanticEdge),
        2 => Ok(ApplicationRelationUniquenessScope::NormalizedSymmetricEdge),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uniqueness_scope_has_only_its_two_current_discriminants() {
        for (tag, expected) in [
            (
                1_u16,
                ApplicationRelationUniquenessScope::DirectedSemanticEdge,
            ),
            (
                2,
                ApplicationRelationUniquenessScope::NormalizedSymmetricEdge,
            ),
        ] {
            assert_eq!(
                decode_unique_scope(&mut BinaryInput::new(&tag.to_be_bytes())).unwrap(),
                expected
            );
        }
        assert_eq!(
            decode_unique_scope(&mut BinaryInput::new(&9_u16.to_be_bytes()))
                .unwrap_err()
                .kind(),
            Kind::UnsupportedRecordVariant
        );
    }
    #[test]
    fn oversized_kind_count_refuses_before_allocating_or_reading_missing_names() {
        let bytes = [2_u32.to_be_bytes(), u32::MAX.to_be_bytes()].concat();
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            crate::limits::WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(
            super::super::super::relation_graph::decode_minimum(
                &mut BinaryInput::new(&bytes),
                &mut budget
            )
            .unwrap_err()
            .kind(),
            Kind::NestedEntryBudgetExceeded
        );
    }
}
