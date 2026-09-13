use worth_store_physical_integrity::PhysicalIntegrityValidationRecord;

/// Process-local identity of one byte image installed in a residency entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalResidentFrameGeneration(u64);

impl PhysicalResidentFrameGeneration {
    pub(crate) const FIRST: Self = Self(1);

    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(next) => Some(Self(next)),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CleanFrameIntegrityValidationRecord {
    generation: PhysicalResidentFrameGeneration,
    validation: PhysicalIntegrityValidationRecord,
}

impl CleanFrameIntegrityValidationRecord {
    pub(crate) const fn new(
        generation: PhysicalResidentFrameGeneration,
        validation: PhysicalIntegrityValidationRecord,
    ) -> Self {
        Self {
            generation,
            validation,
        }
    }

    pub(crate) const fn validation_for(
        self,
        generation: PhysicalResidentFrameGeneration,
    ) -> Option<PhysicalIntegrityValidationRecord> {
        if self.generation.get() == generation.get() {
            Some(self.validation)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanFrameIntegrityValidationDenial {
    PoolClosed,
    FrameNotResident,
    FrameGenerationChanged,
    FrameBytesChanged,
    FrameDirty,
}
