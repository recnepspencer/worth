use super::comparison::{compare, ComparisonMismatch, ObservedSupplyChainState};
use super::expected_observation::ExpectedSupplyChainObservation;
use super::observation::{observe, ObservationError};
use super::oracle::{OracleBranch, OracleState};
use super::production_world::ProductionSeededSupplyChainWorld;
use super::schema::SchemaError;

pub(crate) struct CertifiedSupplyChainBaseline {
    pub(crate) world: ProductionSeededSupplyChainWorld,
    pub(crate) expected: ExpectedSupplyChainObservation,
    pub(crate) observed: ObservedSupplyChainState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BaselineAuditError {
    Oracle(SchemaError),
    Observation(ObservationError),
    Comparison(ComparisonMismatch),
}

pub(crate) fn audit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: ProductionSeededSupplyChainWorld,
) -> Result<CertifiedSupplyChainBaseline, BaselineAuditError> {
    let oracle = OracleBranch::genesis(OracleState::from_definition(world.program.definition()));
    oracle
        .state
        .validate_complete()
        .map_err(BaselineAuditError::Oracle)?;
    let expected = ExpectedSupplyChainObservation::from_branch(&oracle);
    let observed = observe(execution, &world).map_err(BaselineAuditError::Observation)?;
    compare(&expected, &observed)
        .map_err(|failure| BaselineAuditError::Comparison(failure.mismatch))?;
    Ok(CertifiedSupplyChainBaseline {
        world,
        expected,
        observed,
    })
}
