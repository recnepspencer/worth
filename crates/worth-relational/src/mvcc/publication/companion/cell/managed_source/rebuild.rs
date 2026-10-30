//! A genuine selected Source loan retains basis custody, never an Execution request.
use super::*;
use crate::validation::candidate_source::ManagedIndexPublishedSourceRoot;
use crate::visibility::materialization::read_records::VisibilityProjectionView;
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
        let core = self.image.admit_selected(&selected, generation)?;
        let image = self
            .image
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
        let retirement = image
            .refill_source(execution, &self.expected, payload)
            .map_err(Denial::Publication)?;
        Ok((
            ManagedIndexSourceObservation {
                core,
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
    fn admit_selected(
        &self,
        selected: &PositionedRelationalSnapshot,
        generation: u64,
    ) -> Result<Arc<CompanionBranchCellCore>, Denial> {
        let core = self.core.upgrade().ok_or(Denial::RootUnavailable)?;
        if core.registration_generation != generation {
            return Err(Denial::Publication(
                CompanionPreflightStop::RegistrationChanged,
            ));
        }
        if core.runtime_instance_id != selected.runtime_instance_id()
            || core.branch_id != *selected.branch_id()
            || self.image.root_id != selected.root_id()
            || self.image.commit_id != selected.commit_id()
            || self.image.position() != selected.position()
        {
            return Err(Denial::ForeignSource);
        }
        Ok(core)
    }
    pub(crate) fn prepare_rebuild(
        &self,
        projection: VisibilityProjectionView,
        selected: PositionedRelationalSnapshot,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        generation: u64,
    ) -> Result<ManagedIndexSourceRebuild, Denial> {
        self.admit_selected(&selected, generation)?;
        let image = self
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
        let expected = image
            .retained(program, scope)?
            .ok_or(Denial::RootUnavailable)?;
        if expected.physical_availability() != ManagedIndexSourceAvailability::Available {
            return Err(Denial::RootUnavailable);
        }
        Ok(ManagedIndexSourceRebuild {
            projection,
            expected,
            image: self.clone(),
            selected,
            generation,
            _memory: memory,
        })
    }
}
