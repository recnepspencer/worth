//! Retained Source identity owns no live cell, newer image or execution request.
use super::*;

pub(in crate::mvcc::publication::companion) struct ManagedSourceCellAffinity {
    runtime: u64,
    branch: BranchId,
    generation: u64,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    // Weak<Core> still owns its Arc allocation after Core value destruction.
    // Share the existing fork quote through the last retained affinity owner.
    _fork_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl CompanionBranchCellCore {
    pub(in crate::mvcc::publication::companion) fn retain_source_affinity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<ManagedSourceCellAffinity>, CompanionPreflightStop> {
        if let Some(affinity) = self.source_affinity.get() {
            return Ok(Arc::clone(affinity));
        }
        let bytes = size_of::<ManagedSourceCellAffinity>()
            .checked_add(2 * size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(self.branch_id.0.len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?;
        let affinity = Arc::new(ManagedSourceCellAffinity {
            runtime: self.runtime_instance_id,
            branch: self.branch_id.clone(),
            generation: self.registration_generation,
            _memory: memory,
            _fork_memory: self._fork_memory.clone(),
        });
        // All competitors derive the same immutable metadata from this exact
        // cell. Only the installed Arc is shared as its retained identity.
        let _ = self.source_affinity.set(affinity);
        Ok(Arc::clone(
            self.source_affinity.get().expect("installed cell affinity"),
        ))
    }
}
impl ManagedSourceCellAffinity {
    pub(super) fn admit(
        &self,
        selected: &PositionedRelationalSnapshot,
        generation: u64,
    ) -> Result<(), Denial> {
        if self.generation != generation {
            return Err(Denial::Publication(
                CompanionPreflightStop::RegistrationChanged,
            ));
        }
        if self.runtime != selected.runtime_instance_id() || self.branch != *selected.branch_id() {
            return Err(Denial::ForeignSource);
        }
        Ok(())
    }
}

impl ManagedSourceRootImage {
    pub(super) fn admit_selected(
        &self,
        selected: &PositionedRelationalSnapshot,
        generation: u64,
    ) -> Result<(), Denial> {
        self.affinity.admit(selected, generation)?;
        if self.image.root_id != selected.root_id()
            || self.image.commit_id != selected.commit_id()
            || self.image.position() != selected.position()
        {
            return Err(Denial::ForeignSource);
        }
        Ok(())
    }
}
