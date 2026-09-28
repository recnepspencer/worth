use super::{ArenaEvacuationSelection, PhysicalRecordSubmission, RecordPublicationDirector};
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationPreparationOutcome, PhysicalMutationRequest,
    RecordAppendDenial,
};

/// A producer poll performs at most one bounded route scan or prepares one
/// extent rewrite. Empty does not assert that protected roots permit deletion.
pub enum PhysicalArenaEvacuationPreparationOutcome {
    Idle,
    ScanPending,
    EmptyAwaitingRetirement {
        arena: u64,
        root_generation: u64,
    },
    RetirementWaiting {
        arena: u64,
        cause: crate::physical_runtime::PhysicalRetirementDenial,
    },
    Rewrite(PhysicalMutationPreparationOutcome),
    Copying(super::extent_copy::PhysicalExtentCopyProgress),
}

impl PhysicalRecordSubmission {
    pub fn prepare_arena_evacuation(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> Result<PhysicalArenaEvacuationPreparationOutcome, RecordAppendDenial> {
        let director = self
            .director
            .upgrade()
            .ok_or(RecordAppendDenial::PhysicalPressure)?;
        director.prepare_arena_evacuation(placement, request)
    }
}

impl RecordPublicationDirector {
    fn prepare_arena_evacuation(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> Result<PhysicalArenaEvacuationPreparationOutcome, RecordAppendDenial> {
        use PhysicalArenaEvacuationPreparationOutcome as Outcome;
        if !placement.admits(self.format) {
            return Err(RecordAppendDenial::PlacementFormatMismatch);
        }
        let selected = self
            .select_arena_evacuation(placement)
            .map_err(|error| match error {
                crate::physical_runtime::record_serving::RecordAppendError::Denied(denial) => {
                    denial
                }
                _ => RecordAppendDenial::PublishedLayoutDamaged,
            })?;
        let source = match selected {
            ArenaEvacuationSelection::Idle => {
                self.mutation.cancel_compaction_background();
                return Ok(Outcome::Idle);
            }
            ArenaEvacuationSelection::Continue => return Ok(Outcome::ScanPending),
            ArenaEvacuationSelection::ReleasePending { arena } => {
                return Ok(Outcome::RetirementWaiting {
                    arena: arena.get(),
                    cause: crate::physical_runtime::PhysicalRetirementDenial::Retained,
                })
            }
            ArenaEvacuationSelection::Empty { source_root, arena } => {
                self.mutation.cancel_compaction_background();
                if let Err(cause) = self.prepare_empty_arena_retirement(source_root, arena) {
                    return Ok(Outcome::RetirementWaiting {
                        arena: arena.get(),
                        cause,
                    });
                }
                return Ok(Outcome::EmptyAwaitingRetirement {
                    arena: arena.get(),
                    root_generation: source_root,
                });
            }
            ArenaEvacuationSelection::Extent { source } => source,
        };
        self.begin_extent_copy(placement, request, source)
            .map(|result| match result {
                Ok(progress) => Outcome::Copying(progress),
                Err(outcome) => Outcome::Rewrite(outcome),
            })
            .map_err(|error| match error {
                crate::physical_runtime::record_serving::RecordAppendError::Denied(denial) => {
                    denial
                }
                _ => RecordAppendDenial::PublishedLayoutDamaged,
            })
    }
}
