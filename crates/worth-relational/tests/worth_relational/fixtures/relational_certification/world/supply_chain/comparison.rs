pub(crate) use super::comparison_ancestry::compare_ancestry;
pub(crate) use super::comparison_entities::compare_entities;
pub(crate) use super::comparison_relations::compare_relations;
pub(crate) use super::comparison_schema::compare_schema;
pub(crate) use super::expected_observation::ExpectedSupplyChainObservation;

pub(crate) use super::comparison_state::{
    ComparisonFailure, ComparisonMismatch, ObservedSupplyChainState,
};

/// Compare a complete observation by semantic axis.  Each axis owns its
/// mismatch construction; this function is only the ordered table of contents.
pub(crate) fn compare(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    compare_schema(expected, observed)?;
    compare_ancestry(expected, observed)?;
    compare_entities(expected, observed)?;
    compare_relations(expected, observed)?;
    Ok(())
}
