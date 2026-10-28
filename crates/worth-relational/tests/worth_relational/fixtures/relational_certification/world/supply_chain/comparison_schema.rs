pub(crate) use super::comparison_state::{
    ComparisonFailure, ComparisonMismatch, ObservedSupplyChainState,
};
pub(crate) use super::expected_observation::ExpectedSupplyChainObservation;

pub(crate) fn compare_schema(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    if expected.schema.version != observed.schema {
        return Err(ComparisonFailure {
            mismatch: ComparisonMismatch::SchemaMeaning {
                expected: expected.schema.version,
                observed: observed.schema,
            },
        });
    }
    Ok(())
}
