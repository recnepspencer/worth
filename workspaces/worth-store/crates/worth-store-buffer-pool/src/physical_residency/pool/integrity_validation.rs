use super::*;
use worth_store_physical_integrity::PhysicalIntegrityValidationRecord;

impl PoolState {
    pub(super) fn advance_resident_generation(
        &mut self,
    ) -> Result<PhysicalResidentFrameGeneration, PhysicalResidencyDenial> {
        let generation = self.next_resident_generation;
        self.next_resident_generation = generation
            .checked_next()
            .ok_or(PhysicalResidencyDenial::AllocationFailed)?;
        Ok(generation)
    }

    pub(super) fn invalidate_all_integrity_validation(&mut self) {
        self.frames
            .values_mut()
            .for_each(FrameEntry::invalidate_integrity_validation);
    }
}

impl PoolInner {
    pub(crate) fn invalidate_integrity_validation_if(
        &self,
        key: PhysicalFrameKey,
        bytes: &Arc<Vec<u8>>,
        generation: PhysicalResidentFrameGeneration,
        expected: PhysicalIntegrityValidationRecord,
    ) {
        let mut state = self.lock();
        let Some(entry) = state.frames.get_mut(&key.coordinate) else {
            return;
        };
        if entry.resident_generation != Some(generation) {
            return;
        }
        let exact_record = entry
            .integrity_validation
            .and_then(|record| record.validation_for(generation))
            == Some(expected);
        if exact_record
            && matches!(&entry.state, FrameState::Resident(current) if Arc::ptr_eq(current, bytes))
        {
            entry.invalidate_integrity_validation();
        }
    }

    pub(crate) fn commit_integrity_validation(
        &self,
        key: PhysicalFrameKey,
        bytes: &Arc<Vec<u8>>,
        generation: PhysicalResidentFrameGeneration,
        validation: PhysicalIntegrityValidationRecord,
    ) -> Result<(), super::super::integrity_validation::CleanFrameIntegrityValidationDenial> {
        use super::super::integrity_validation::CleanFrameIntegrityValidationDenial as Denial;

        let mut state = self.lock();
        if !state.accepting {
            return Err(Denial::PoolClosed);
        }
        let Some(entry) = state.frames.get_mut(&key.coordinate) else {
            return Err(Denial::FrameNotResident);
        };
        if entry.dirty {
            return Err(Denial::FrameDirty);
        }
        if entry.resident_generation != Some(generation) {
            return Err(Denial::FrameGenerationChanged);
        }
        match &entry.state {
            FrameState::Resident(current) if Arc::ptr_eq(current, bytes) => {}
            FrameState::Resident(_) => return Err(Denial::FrameBytesChanged),
            _ => return Err(Denial::FrameNotResident),
        }
        entry.integrity_validation = Some(CleanFrameIntegrityValidationRecord::new(
            generation, validation,
        ));
        Ok(())
    }

    pub(crate) fn integrity_validation(
        &self,
        key: PhysicalFrameKey,
        bytes: &Arc<Vec<u8>>,
        generation: PhysicalResidentFrameGeneration,
    ) -> Option<PhysicalIntegrityValidationRecord> {
        let state = self.lock();
        let entry = state.frames.get(&key.coordinate)?;
        if entry.dirty || entry.resident_generation != Some(generation) {
            return None;
        }
        match &entry.state {
            FrameState::Resident(current) if Arc::ptr_eq(current, bytes) => entry
                .integrity_validation
                .and_then(|record| record.validation_for(generation)),
            _ => None,
        }
    }
}
