use std::sync::Arc;

use crate::commit_strategies::data::{
    CommitStrategyId, StrategyPacketContract, StrategyReadContract, StrategyReadCostClass,
    StrategyReadLocalityClass, StrategyReadScopeClass, StrategyTraversalBasis,
};

use super::error::StrategyExecutionError;

pub fn validate_supported_read_contract(
    strategy_id: CommitStrategyId,
    read_contract: &StrategyReadContract,
) -> Result<(), StrategyExecutionError> {
    admit_execution_read_scope(strategy_id, read_contract.scope_class)?;
    admit_execution_read_locality(strategy_id, read_contract.locality_class)?;
    admit_execution_traversal_basis(strategy_id, read_contract.traversal_basis)?;
    admit_execution_packet_contract(strategy_id, read_contract.packet_contract)?;
    admit_execution_cost_class(strategy_id, read_contract.cost_class)?;
    Ok(())
}

fn admit_execution_read_scope(
    strategy_id: CommitStrategyId,
    scope_class: StrategyReadScopeClass,
) -> Result<(), StrategyExecutionError> {
    match scope_class {
        StrategyReadScopeClass::ExplicitTargetsOnly
        | StrategyReadScopeClass::PartitionBoundedScan => Ok(()),
        StrategyReadScopeClass::KindBoundedScan => deny_read_contract(
            strategy_id,
            "KindBoundedScan is not execution-admissible until bounded cross-partition accounting is implemented",
        ),
        StrategyReadScopeClass::BoundedNeighborhood => deny_read_contract(
            strategy_id,
            "BoundedNeighborhood is not execution-admissible until bounded traversal accounting is implemented",
        ),
    }
}

fn admit_execution_read_locality(
    strategy_id: CommitStrategyId,
    locality_class: StrategyReadLocalityClass,
) -> Result<(), StrategyExecutionError> {
    match locality_class {
        StrategyReadLocalityClass::SinglePartition
        | StrategyReadLocalityClass::PartitionBounded => Ok(()),
        StrategyReadLocalityClass::CrossPartitionBounded => deny_read_contract(
            strategy_id,
            "CrossPartitionBounded is not execution-admissible until explicit partition-bound enforcement exists",
        ),
    }
}

fn admit_execution_traversal_basis(
    strategy_id: CommitStrategyId,
    traversal_basis: StrategyTraversalBasis,
) -> Result<(), StrategyExecutionError> {
    match traversal_basis {
        StrategyTraversalBasis::NoTraversal => Ok(()),
        _ => deny_read_contract(
            strategy_id,
            "Traversal-enabled strategy execution is not admissible until traversal accounting is implemented",
        ),
    }
}

fn admit_execution_packet_contract(
    strategy_id: CommitStrategyId,
    packet_contract: StrategyPacketContract,
) -> Result<(), StrategyExecutionError> {
    match packet_contract {
        StrategyPacketContract::ProjectionOnly => Ok(()),
        _ => deny_read_contract(
            strategy_id,
            "PlannedPacketOnly is not execution-admissible until packet-planned strategy reads are implemented",
        ),
    }
}

fn admit_execution_cost_class(
    strategy_id: CommitStrategyId,
    cost_class: StrategyReadCostClass,
) -> Result<(), StrategyExecutionError> {
    match cost_class {
        StrategyReadCostClass::ORequestedSurface => Ok(()),
        _ => deny_read_contract(
            strategy_id,
            "Only ORequestedSurface strategy execution is admissible until cost enforcement is implemented",
        ),
    }
}

fn deny_read_contract<T>(
    strategy_id: CommitStrategyId,
    detail: &'static str,
) -> Result<T, StrategyExecutionError> {
    Err(StrategyExecutionError::UnsupportedReadContract {
        strategy_id,
        detail: Arc::from(detail),
    })
}
