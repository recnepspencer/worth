use worth_store_physical_format::PhysicalGenerationOwner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionSourceIntegrityAdmissionDenial {
    EmptyInspection,
}

/// Physical-isolation-owned admission for source bytes that may be moved.
///
/// The integrity package supplies descriptive evidence. This owner decides
/// whether that evidence is sufficient to authorize a compaction transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionSourceIntegrityAdmission {
    locality_owner: Option<PhysicalGenerationOwner>,
    inspected_bytes: u64,
    movement_permitted: bool,
}

impl CompactionSourceIntegrityAdmission {
    pub(in crate::compaction_interlock) fn admit_intact_source(
        locality_owner: PhysicalGenerationOwner,
        inspected_bytes: u64,
    ) -> Result<Self, CompactionSourceIntegrityAdmissionDenial> {
        if inspected_bytes == 0 {
            return Err(CompactionSourceIntegrityAdmissionDenial::EmptyInspection);
        }
        Ok(Self {
            locality_owner: Some(locality_owner),
            inspected_bytes,
            movement_permitted: true,
        })
    }

    pub(in crate::compaction_interlock) const fn quarantined_source(
        locality_owner: PhysicalGenerationOwner,
    ) -> Self {
        Self {
            locality_owner: Some(locality_owner),
            inspected_bytes: 0,
            movement_permitted: false,
        }
    }

    #[cfg(any(test, feature = "certification-authority"))]
    pub fn for_certification_test(
        locality_owner: PhysicalGenerationOwner,
        inspected_bytes: u64,
    ) -> Result<Self, CompactionSourceIntegrityAdmissionDenial> {
        Self::admit_intact_source(locality_owner, inspected_bytes)
    }

    pub const fn permits_compaction_movement(self) -> bool {
        self.movement_permitted
    }

    pub const fn inspected_bytes(self) -> u64 {
        self.inspected_bytes
    }

    pub const fn locality_owner(self) -> Option<PhysicalGenerationOwner> {
        self.locality_owner
    }
}
