pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison::compare, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::observation::observe, crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, crate::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError};

pub struct CertifiedSupplyChainBaseline {
    pub world: ProductionSeededSupplyChainWorld,
    pub expected: ExpectedSupplyChainObservation,
    pub observed: ObservedSupplyChainState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BaselineAuditError {
    Oracle(SchemaError),
    Observation(ObservationError),
    Comparison(ComparisonMismatch),
}

pub fn audit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: ProductionSeededSupplyChainWorld,
) -> Result<CertifiedSupplyChainBaseline, BaselineAuditError> {
    let oracle = crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch::genesis(crate::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState::from_definition(world.program.definition()));
    oracle
        .state
        .validate_complete()
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::baseline_audit::BaselineAuditError::Oracle)?;
    let expected = crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&oracle);
    let observed = observe(execution, &world).map_err(crate::relational_certification::ordinary_context::world::supply_chain::baseline_audit::BaselineAuditError::Observation)?;
    compare(&expected, &observed)
        .map_err(|failure| crate::relational_certification::ordinary_context::world::supply_chain::baseline_audit::BaselineAuditError::Comparison(failure.mismatch))?;
    Ok(CertifiedSupplyChainBaseline {
        world,
        expected,
        observed,
    })
}
