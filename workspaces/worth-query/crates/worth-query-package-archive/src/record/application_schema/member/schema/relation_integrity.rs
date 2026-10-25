//! The complete owned relation-integrity record grammar.
use super::super::super::super::foundational_value::{decode_bool, write_bool};
use super::super::super::wire_vocabulary::{decode_optional, write_optional};
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
    record::{
        decode_budget::RecordDecodeAttempt,
        sequence::{decode_sequence, write_sequence},
    },
};
use worth_query_declaration::facade::application_schema::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints,
    ApplicationRelationIntegrityRecord, ApplicationRelationMinimumReachabilityRecord,
    ApplicationRelationUniquenessScope,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    record: &ApplicationRelationIntegrityRecord,
) -> Result<(), Denial> {
    output.u16(match record.endpoints.cross_context_policy {
        ApplicationRelationCrossContextPolicy::AllowExplicit => 1,
        ApplicationRelationCrossContextPolicy::SchemaControlled => 2,
        ApplicationRelationCrossContextPolicy::Forbid => 3,
    })?;
    write_bool(output, record.endpoints.self_edges_allowed)?;
    for value in [
        record.cardinality.source_min,
        record.cardinality.source_max,
        record.cardinality.target_min,
        record.cardinality.target_max,
        record.cardinality.pair_min,
        record.cardinality.pair_max,
    ] {
        write_optional(output, value.as_ref(), |output, value| output.u64(*value))?;
    }
    output.u16(match record.deletion {
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit => 1,
        ApplicationRelationDeletionPolicy::CascadeDeleteRelations => 2,
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations => 3,
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit => 4,
        ApplicationRelationDeletionPolicy::RequireRelationRetirement => 5,
    })?;
    write_bool(output, record.acyclic)?;
    write_bool(output, record.symmetric)?;
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
        |output, minimum| {
            output.u32(minimum.minimum_reachable_targets)?;
            write_sequence(output, &minimum.target_kinds, |output, kind| {
                output.text(kind)
            })
        },
    )
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationRelationIntegrityRecord, Denial> {
    let cross_context_policy = match input.u16()? {
        1 => ApplicationRelationCrossContextPolicy::AllowExplicit,
        2 => ApplicationRelationCrossContextPolicy::SchemaControlled,
        3 => ApplicationRelationCrossContextPolicy::Forbid,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let self_edges_allowed = decode_bool(input)?;
    let mut next_bound = || decode_optional(input, |input| input.u64());
    let cardinality = ApplicationRelationCardinality::new(
        next_bound()?,
        next_bound()?,
        next_bound()?,
        next_bound()?,
        next_bound()?,
        next_bound()?,
    );
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
    let unique = decode_optional(input, decode_unique_scope)?;
    let same_partition_endpoints = decode_bool(input)?;
    let reaches_at_least = decode_optional(input, |input| decode_minimum(input, budget))?;
    Ok(ApplicationRelationIntegrityRecord {
        endpoints: ApplicationRelationEndpoints::new(self_edges_allowed, cross_context_policy),
        cardinality,
        deletion,
        acyclic,
        symmetric,
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

fn decode_minimum(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationRelationMinimumReachabilityRecord, Denial> {
    budget.require_nesting_depth(2)?;
    let minimum_reachable_targets = input.u32()?;
    let target_kinds = decode_sequence(input, budget, 4, |input, _| Ok(input.text()?.to_owned()))?;
    Ok(ApplicationRelationMinimumReachabilityRecord {
        minimum_reachable_targets,
        target_kinds,
    })
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
            decode_minimum(&mut BinaryInput::new(&bytes), &mut budget)
                .unwrap_err()
                .kind(),
            Kind::NestedEntryBudgetExceeded
        );
    }
}
