pub use {std::collections::BTreeSet};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_competing_arrival, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_hazard_v2, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_inspection_retirement, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_maintenance, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_medical_hold, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_rewire, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_southpoint_expansion, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_storm};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::observation::observe_observation as observe_supply_chain_observation, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, crate::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles};
pub use {worth_relational::facade::history::BranchId};
pub use {worth_relational::facade::runtime::RelationalRuntime};
pub use {worth_relational::facade::transactions::WorkerIntentBatch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainProductionDeltaLoweringError {
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
pub fn lower_supply_chain_production_delta(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    program: &crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    branch_id: &BranchId,
    previously_applied: &BTreeSet<DeltaId>,
    delta: DeltaId,
) -> Result<WorkerIntentBatch, SupplyChainProductionDeltaLoweringError> {
    let identity = runtime
        .branch_identity(branch_id)
        .map_err(|_| crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::UnknownBranch(branch_id.clone()))?;
    let (_, basis) = runtime
        .observe_branch(&identity)
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::BranchBasis)?;
    let observation = basis.observation();
    let branch_handles = handles.for_observation(&observation);
    let observed = observe_supply_chain_observation(
        execution,
        program,
        &branch_handles,
        runtime,
        &observation,
    )
    .map_err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::Observation)?;
    validate_preconditions(&observed, previously_applied, delta)?;

    match delta {
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora => lower_storm(handles, &observed),
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth => lower_maintenance(handles, &observed),
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo => lower_medical_hold(handles),
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity => lower_southpoint_expansion(handles, &observed),
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival => lower_competing_arrival(handles, &observed),
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora => lower_inspection_retirement(handles),
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall => lower_rewire(handles, &observed),
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 => lower_hazard_v2(handles),
    }
}

pub fn validate_preconditions(
    observed: &ObservedSupplyChainState,
    previously_applied: &BTreeSet<DeltaId>,
    delta: DeltaId,
) -> Result<(), SupplyChainProductionDeltaLoweringError> {
    for precondition in delta.contract().preconditions {
        match precondition {
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(key) if !observed.entities.contains_key(&key) => {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::MissingEntity(key));
            }
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::RelationPresent(key) if !observed.relations.contains_key(&key) => {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::MissingRelation(
                    key,
                ));
            }
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(expected) if observed.schema != expected => {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::WrongSchema {
                    expected,
                    observed: observed.schema,
                });
            }
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(expected) if observed.branch != expected => {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::WrongBranch {
                    expected,
                    observed: observed.branch,
                });
            }
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(expected)
                if previously_applied.contains(&expected) =>
            {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::DuplicateDelta(
                    expected,
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn observed_entity(
    observed: &ObservedSupplyChainState,
    key: EntityKey,
) -> Result<&EntityRecord, SupplyChainProductionDeltaLoweringError> {
    observed
        .entities
        .get(&key)
        .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::MissingEntity(key))
}

pub fn observed_relation_source(
    observed: &ObservedSupplyChainState,
    key: RelationKey,
) -> Result<EntityKey, SupplyChainProductionDeltaLoweringError> {
    observed.relations.get(&key).map(|edge| edge.source).ok_or(
        crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::MissingRelation(key),
    )
}