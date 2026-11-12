//! Version-scanned Native transport waits for the same World DATA before eligibility.
use crate::{home_context::RelationalHomeOpeningInput, open::ApplicationHomeOpenRefusal as Denial};
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest,
    WorkCeilingDenial,
};
use worth_relational::facade::durability::{
    RelationalDurableReadBounds, RelationalWorldSourceCatalog,
};
use worth_store_relational::{PreparedFreshRelationalHomeSource, RelationalHomeSourcePreparation};

pub(in crate::open) enum PreparedNativeOpeningInput {
    Fresh(PreparedFreshRelationalHomeSource),
    Existing(Box<OriginalNativeOpening>),
}
pub(in crate::open) struct OriginalNativeOpening {
    preparation: RelationalHomeSourcePreparation,
    initial: Option<ExecutionImmutableBytes>,
    canonical: ExecutionArray<ExecutionImmutableBytes>,
    transitions: ExecutionArray<ExecutionImmutableBytes>,
    bounds: RelationalDurableReadBounds,
    _memory: Option<ExecutionMemoryReservation>,
}
impl PreparedNativeOpeningInput {
    pub(super) fn existing(
        preparation: RelationalHomeSourcePreparation,
        initial: Option<ExecutionImmutableBytes>,
        canonical: ExecutionArray<ExecutionImmutableBytes>,
        transitions: ExecutionArray<ExecutionImmutableBytes>,
        bounds: RelationalDurableReadBounds,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorkCeilingDenial> {
        let memory = request.reserve_retained_memory(size_of::<OriginalNativeOpening>() as u64)?;
        Ok(Self::Existing(Box::new(OriginalNativeOpening {
            preparation,
            initial,
            canonical,
            transitions,
            bounds,
            _memory: memory,
        })))
    }
    pub(in crate::open) fn finish(
        self,
        world: Option<Arc<RelationalWorldSourceCatalog>>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalHomeOpeningInput, Denial> {
        match self {
            Self::Fresh(prepared) => {
                if world.is_some() {
                    return Err(Denial::Relational(worth_store_relational::RelationalSectionReadmissionDenial::MalformedSection));
                }
                Ok(RelationalHomeOpeningInput::Fresh(prepared))
            }
            Self::Existing(original) => {
                let OriginalNativeOpening {
                    preparation,
                    initial,
                    canonical,
                    transitions,
                    bounds,
                    _memory,
                } = *original;
                preparation
                    .finish(initial, canonical, transitions, bounds, world, request)
                    .map(RelationalHomeOpeningInput::Existing)
                    .map_err(Denial::Relational)
            }
        }
    }
}
