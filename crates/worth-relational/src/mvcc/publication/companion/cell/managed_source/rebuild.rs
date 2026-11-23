//! A genuine selected Source loan retains basis custody, never an Execution request.
use super::*;
use crate::validation::candidate_source::ManagedIndexPublishedSourceRoot;
use crate::visibility::materialization::read_records::VisibilityProjectionView;
use worth_execution::ExecutionRequest;

pub struct ManagedIndexSourceRebuild {
    pub(crate) projection: VisibilityProjectionView,
    pub(crate) expected: Arc<ManagedIndexPublishedSourceRoot>,
    expected_placed: bool,
    image: ManagedSourceRootImage,
    selected: PositionedRelationalSnapshot,
    generation: u64,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ManagedIndexSourceRebuild {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        &self.expected.association.program
    }
    pub fn scope(&self) -> EntityId {
        self.expected.association.scope
    }
    pub(crate) fn finish(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        selected: PositionedRelationalSnapshot,
        generation: u64,
        payload: &Arc<dyn Any + Send + Sync>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<
        (
            ManagedIndexSourceObservation,
            crate::validation::candidate_source::ManagedIndexSourceCacheRetirement,
        ),
        Denial,
    > {
        if self.selected != selected || self.generation != generation {
            return Err(Denial::ForeignSource);
        }
        self.image.admit_selected(&selected, generation)?;
        let image = self
            .image
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
        let retirement = image
            .refill_source(execution, &self.expected, self.expected_placed, payload)
            .map_err(Denial::Publication)?;
        Ok((
            ManagedIndexSourceObservation {
                core: self.image.core.clone(),
                affinity: Arc::clone(&self.image.affinity),
                record: Arc::clone(&self.expected.association),
                payload: Arc::clone(payload),
                selected,
                _metadata_memory: memory,
            },
            retirement,
        ))
    }
}
impl ManagedSourceRootImage {
    pub(crate) fn prepare_rebuild(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        projection: VisibilityProjectionView,
        selected: PositionedRelationalSnapshot,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        generation: u64,
        admit_cold_scope: impl FnOnce(&VisibilityProjectionView) -> Result<(), Denial>,
    ) -> Result<ManagedIndexSourceRebuild, Denial> {
        self.admit_selected(&selected, generation)?;
        let image = self
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
        let (expected, expected_placed) = match image.retained(program, scope)? {
            Some(placed) => (placed, true),
            None => {
                admit_cold_scope(&projection)?;
                let revision = selected.commit_id().ok_or(Denial::RootUnavailable)?;
                let cold =
                    ManagedIndexBranchImage::plan_cold_source(execution, program, scope, revision)
                        .map_err(Denial::Publication)?;
                (cold, false)
            }
        };
        if expected.physical_availability() != ManagedIndexSourceAvailability::Available {
            return Err(Denial::RootUnavailable);
        }
        Ok(ManagedIndexSourceRebuild {
            projection,
            expected,
            expected_placed,
            image: self.clone(),
            selected,
            generation,
            _memory: memory,
        })
    }
}
