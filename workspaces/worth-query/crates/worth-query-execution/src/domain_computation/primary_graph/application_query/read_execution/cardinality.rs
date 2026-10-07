//! Cardinality admission for both typed and prepared reads.

use super::*;

pub(super) fn validate_cardinality_and_limit<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    cardinality: ApplicationQueryCardinality,
    count: usize,
    plan: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    validate_read_cardinality(cardinality, count, &read_plan::ReadPlan::of(plan))
}

pub(super) fn validate_read_cardinality(
    cardinality: ApplicationQueryCardinality,
    count: usize,
    plan: &read_plan::ReadPlan<'_>,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    if count > plan.maximum_result_count {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ResultLimitExceeded,
            plan.name,
        ));
    }
    let valid = match cardinality {
        ApplicationQueryCardinality::OptionalOne => count <= 1,
        ApplicationQueryCardinality::ExactlyOne => count == 1,
        ApplicationQueryCardinality::Many => true,
    };
    if valid {
        Ok(())
    } else {
        Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch,
            plan.name,
        ))
    }
}
