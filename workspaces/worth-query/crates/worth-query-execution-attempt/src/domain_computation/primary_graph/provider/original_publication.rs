//! Concrete original-owner inputs; memory data cannot qualify a Home source.
use worth_relational::facade::{
    durability::{ReadmittedRelationalCreatedEntity, RelationalAssociatedEffectRefusal},
    identity::EntityId,
    transactions::RelationalCommitSemanticResult,
};
use worth_runtime_world::facade::{WorldObservedHomePublication, WorldObservedMemoryPublication};

pub(super) enum OriginalPublication {
    Home(WorldObservedHomePublication),
    Memory(WorldObservedMemoryPublication),
}

impl OriginalPublication {
    pub(super) fn matches_affinity(
        &self,
        expected: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        match self {
            Self::Home(value) => value.matches_receiver(expected),
            Self::Memory(value) => value.matches_receiver(expected),
        }
    }
    pub(super) fn branch_name_bytes(&self) -> usize {
        match self {
            Self::Home(value) => {
                value.before().branch().name().len() + value.after().branch().name().len()
            }
            Self::Memory(value) => value.before().branch().len() + value.after().branch().len(),
        }
    }
    pub(super) fn world_commit(&self) -> u64 {
        match self {
            Self::Home(value) => value.after().selected_commit().ordinal(),
            Self::Memory(value) => value.after().selected_commit(),
        }
    }
    pub(super) fn recorded_components(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        port: &worth_relational::facade::branch::RelationalBranchBasisPort,
    ) -> Result<
        (
            worth_relational::facade::branch::RecordedRelationalComponentReference,
            worth_relational::facade::branch::RecordedRelationalComponentReference,
        ),
        worth_relational::facade::branch::RelationalRecordedComponentRefusal,
    > {
        match self {
            Self::Home(value) => port.export_recorded_effect_components(
                execution,
                value.native(),
                value.recorded_before_component(),
            ),
            Self::Memory(value) => Ok((
                port.export_memory_component(execution, value.before_component())?,
                port.export_memory_component(execution, value.after_component())?,
            )),
        }
    }
    pub(super) fn semantic_result(&self) -> &RelationalCommitSemanticResult {
        match self {
            Self::Home(value) => value.semantic_result(),
            Self::Memory(value) => value.native().semantic_result().semantic(),
        }
    }
    pub(super) fn owner_coordinate(&self) -> worth_foundational::facade::WorldOwnerCoordinate {
        match self {
            Self::Home(value) => worth_foundational::facade::WorldOwnerCoordinate::Home {
                owner: value.after().branch().owner(),
            },
            Self::Memory(value) => worth_foundational::facade::WorldOwnerCoordinate::MemoryOnly {
                observation: value.after().owner(),
            },
        }
    }

    pub(super) fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<Option<ReadmittedRelationalCreatedEntity<'_>>, RelationalAssociatedEffectRefusal>
    {
        match self {
            Self::Home(value) => value.created_entity(execution, entity),
            Self::Memory(value) => value.native().created_entity(execution, entity),
        }
    }
}
