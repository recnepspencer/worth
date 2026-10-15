use std::collections::BTreeSet;

use super::delta_batches::*;
use crate::relational_certification::world::supply_chain::{
    observe_supply_chain_observation, BranchLabel, DeltaId, DeltaPrecondition, EntityKey,
    EntityRecord, ObservationError, ObservedSupplyChainState, RelationKey, SchemaVersion,
    SupplyChainSemanticHandles,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SupplyChainProductionDeltaLoweringError {
    UnknownBranch(BranchId),
    BranchBasis(worth_relational::facade::branch::RelationalBranchBasisDenial),
    Observation(ObservationError),
    MissingEntity(EntityKey),
    MissingRelation(RelationKey),
    WrongBranch {
        expected: BranchLabel,
        observed: BranchLabel,
    },
    WrongSchema {
        expected: SchemaVersion,
        observed: SchemaVersion,
    },
    DuplicateDelta(DeltaId),
    UnexpectedEntity(EntityKey),
    ArithmeticOverflow(EntityKey),
}

/// Lowers semantic intent from the selected branch's actual public owner
/// observation. Expected oracle state never enters this path.
pub(crate) fn lower_supply_chain_production_delta(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    program: &crate::relational_certification::world::supply_chain::CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    branch_id: &BranchId,
    previously_applied: &BTreeSet<DeltaId>,
    delta: DeltaId,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let identity = runtime
        .branch_identity(branch_id)
        .map_err(|_| SupplyChainProductionDeltaLoweringError::UnknownBranch(branch_id.clone()))?;
    let (_, basis) = runtime
        .observe_branch(&identity)
        .map_err(SupplyChainProductionDeltaLoweringError::BranchBasis)?;
    let observation = basis.observation();
    let branch_handles = handles.for_observation(&observation);
    let observed = observe_supply_chain_observation(
        execution,
        program,
        &branch_handles,
        runtime,
        &observation,
    )
    .map_err(SupplyChainProductionDeltaLoweringError::Observation)?;
    validate_preconditions(&observed, previously_applied, delta)?;

    match delta {
        DeltaId::StormRerouteAurora => lower_storm(handles, &observed),
        DeltaId::MaintainAtlasBerth => lower_maintenance(handles, &observed),
        DeltaId::HoldMedicalCargo => lower_medical_hold(handles),
        DeltaId::ExpandSouthpointCapacity => lower_southpoint_expansion(handles, &observed),
        DeltaId::CompetingAuroraArrival => lower_competing_arrival(handles, &observed),
        DeltaId::RetireAtlasWhileInspectingAurora => lower_inspection_retirement(handles),
        DeltaId::RewireAuroraPortCall => lower_rewire(handles, &observed),
        DeltaId::AdoptHazardClassificationV2 => lower_hazard_v2(handles),
    }
}

fn validate_preconditions(
    observed: &ObservedSupplyChainState,
    previously_applied: &BTreeSet<DeltaId>,
    delta: DeltaId,
) -> Result<(), SupplyChainProductionDeltaLoweringError> {
    for precondition in delta.contract().preconditions {
        match precondition {
            DeltaPrecondition::EntityPresent(key) if !observed.entities.contains_key(&key) => {
                return Err(SupplyChainProductionDeltaLoweringError::MissingEntity(key));
            }
            DeltaPrecondition::RelationPresent(key) if !observed.relations.contains_key(&key) => {
                return Err(SupplyChainProductionDeltaLoweringError::MissingRelation(
                    key,
                ));
            }
            DeltaPrecondition::Schema(expected) if observed.schema != expected => {
                return Err(SupplyChainProductionDeltaLoweringError::WrongSchema {
                    expected,
                    observed: observed.schema,
                });
            }
            DeltaPrecondition::Branch(expected) if observed.branch != expected => {
                return Err(SupplyChainProductionDeltaLoweringError::WrongBranch {
                    expected,
                    observed: observed.branch,
                });
            }
            DeltaPrecondition::DeltaNotAccepted(expected)
                if previously_applied.contains(&expected) =>
            {
                return Err(SupplyChainProductionDeltaLoweringError::DuplicateDelta(
                    expected,
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn observed_entity(
    observed: &ObservedSupplyChainState,
    key: EntityKey,
) -> Result<&EntityRecord, SupplyChainProductionDeltaLoweringError> {
    observed
        .entities
        .get(&key)
        .ok_or(SupplyChainProductionDeltaLoweringError::MissingEntity(key))
}

pub(super) fn observed_relation_source(
    observed: &ObservedSupplyChainState,
    key: RelationKey,
) -> Result<EntityKey, SupplyChainProductionDeltaLoweringError> {
    observed.relations.get(&key).map(|edge| edge.source).ok_or(
        SupplyChainProductionDeltaLoweringError::MissingRelation(key),
    )
}
