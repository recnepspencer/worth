//! The graph-only prefix and suffix shared by stored and derived relation laws.
use super::super::wire_vocabulary::{decode_optional, write_optional};
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::foundational_value::{decode_bool, write_bool};
use crate::record::{
    decode_budget::RecordDecodeAttempt,
    sequence::{decode_sequence, write_sequence},
};
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};
use worth_query_declaration::facade::application_schema::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationEndpoints, ApplicationRelationMinimumReachabilityRecord,
};

pub(super) fn write_prefix(
    output: &mut dyn BinaryEncodingSink,
    endpoints: ApplicationRelationEndpoints,
    cardinality: ApplicationRelationCardinality,
) -> Result<(), Denial> {
    output.u16(match endpoints.cross_context_policy {
        ApplicationRelationCrossContextPolicy::AllowExplicit => 1,
        ApplicationRelationCrossContextPolicy::SchemaControlled => 2,
        ApplicationRelationCrossContextPolicy::Forbid => 3,
    })?;
    write_bool(output, endpoints.self_edges_allowed)?;
    for value in [
        cardinality.source_min,
        cardinality.source_max,
        cardinality.target_min,
        cardinality.target_max,
        cardinality.pair_min,
        cardinality.pair_max,
    ] {
        write_optional(output, value.as_ref(), |output, value| output.u64(*value))?;
    }
    Ok(())
}

pub(super) fn decode_prefix(
    input: &mut BinaryInput<'_>,
) -> Result<(ApplicationRelationEndpoints, ApplicationRelationCardinality), Denial> {
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
    Ok((
        ApplicationRelationEndpoints::new(self_edges_allowed, cross_context_policy),
        cardinality,
    ))
}

pub(super) fn write_minimum(
    output: &mut dyn BinaryEncodingSink,
    minimum: &ApplicationRelationMinimumReachabilityRecord,
) -> Result<(), Denial> {
    output.u32(minimum.minimum_reachable_targets)?;
    write_sequence(output, &minimum.target_kinds, |output, kind| {
        output.text(kind)
    })
}

pub(super) fn decode_minimum(
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
