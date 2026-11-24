//! A genuine selected Source loan retains basis custody, never an Execution request.
use super::*;
use crate::validation::candidate_source::ManagedIndexPublishedSourceRoot;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use worth_execution::ExecutionRequest;

pub struct ManagedIndexSourceRebuild {
    pub(crate) projection: VisibilityProjectionView,
    pub(crate) expected: Arc<ManagedIndexPublishedSourceRoot>,
    image: ManagedSourceRootImage,
    selected: PositionedRelationalSnapshot,
    generation: u64,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ManagedIndexSourceRebuild {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        &(*self.expected.owner_association()).program
    }
    pub fn scope(&self) -> EntityId {
        (*self.expected.owner_association()).scope
    }
    /// Each operation reborrows this caller's original request.
    pub fn rows<'a>(
        &'a self,
        execution: ExecutionRequest<'a, 'a>,
    ) -> crate::validation::candidate_source::ManagedIndexSourceRows<'a> {
        crate::validation::candidate_source::selected_rows(
            &self.projection,
            &self.expected,
            execution,
        )
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
        self.image.refill_selected(
            execution,
            selected,
            generation,
            &self.expected,
            payload,
            memory,
        )
    }

    pub(crate) fn prepare(
        image: &ManagedSourceRootImage,
        projection: VisibilityProjectionView,
        selected: PositionedRelationalSnapshot,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        generation: u64,
    ) -> Result<ManagedIndexSourceRebuild, Denial> {
        image.admit_selected(&selected, generation)?;
        let expected = image
            .selected_source_image()?
            .retained(program, scope)?
            .ok_or(Denial::RootUnavailable)?;
        if expected.physical_availability() != ManagedIndexSourceAvailability::Available {
            return Err(Denial::RootUnavailable);
        }
        Ok(ManagedIndexSourceRebuild {
            projection,
            expected,
            image: image.clone(),
            selected,
            generation,
            _memory: memory,
        })
    }
}
