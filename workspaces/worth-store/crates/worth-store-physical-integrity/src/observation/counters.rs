use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;

use crate::localization::PhysicalDamageCause;
use crate::validation::PhysicalIntegrityRejection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityRejectionClass {
    Damaged(PhysicalDamageCause),
    Unsupported,
    Unknown,
    Indeterminate,
}

const REJECTION_CLASS_COUNT: usize = 20;

impl PhysicalIntegrityRejectionClass {
    const fn index(self) -> usize {
        match self {
            Self::Damaged(PhysicalDamageCause::WrongMagic) => 0,
            Self::Damaged(PhysicalDamageCause::FamilyMismatch) => 1,
            Self::Damaged(PhysicalDamageCause::FramingLengthMismatch) => 2,
            Self::Damaged(PhysicalDamageCause::ChecksumMismatch) => 3,
            Self::Damaged(PhysicalDamageCause::FormatMismatch) => 4,
            Self::Damaged(PhysicalDamageCause::StoreIdentityMismatch) => 5,
            Self::Damaged(PhysicalDamageCause::ArtifactIdentityMismatch) => 6,
            Self::Damaged(PhysicalDamageCause::PhysicalGenerationMismatch) => 7,
            Self::Damaged(PhysicalDamageCause::SelectorRoleMismatch) => 8,
            Self::Damaged(PhysicalDamageCause::RecordKindMismatch) => 9,
            Self::Damaged(PhysicalDamageCause::ChildReferenceMismatch) => 10,
            Self::Damaged(PhysicalDamageCause::SequenceMismatch) => 11,
            Self::Damaged(PhysicalDamageCause::AggregateMismatch) => 12,
            Self::Damaged(PhysicalDamageCause::MalformedStructure) => 13,
            Self::Damaged(PhysicalDamageCause::Truncated) => 14,
            Self::Damaged(PhysicalDamageCause::MissingArtifact) => 15,
            Self::Damaged(PhysicalDamageCause::DuplicateArtifact) => 16,
            Self::Unsupported => 17,
            Self::Unknown => 18,
            Self::Indeterminate => 19,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityObservationCounters {
    family: PhysicalIntegrityArtifactFamily,
    inspected_frames: u64,
    inspected_bytes: u64,
    intact_frames: u64,
    rejected_frames: u64,
    rejections_by_class: [u64; REJECTION_CLASS_COUNT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityCounterDenial {
    Overflow,
}

impl PhysicalIntegrityObservationCounters {
    pub const fn empty(family: PhysicalIntegrityArtifactFamily) -> Self {
        Self {
            family,
            inspected_frames: 0,
            inspected_bytes: 0,
            intact_frames: 0,
            rejected_frames: 0,
            rejections_by_class: [0; REJECTION_CLASS_COUNT],
        }
    }

    pub(crate) fn record_intact(
        &mut self,
        byte_count: u64,
    ) -> Result<(), PhysicalIntegrityCounterDenial> {
        let inspected_frames = checked_increment(self.inspected_frames)?;
        let inspected_bytes = self
            .inspected_bytes
            .checked_add(byte_count)
            .ok_or(PhysicalIntegrityCounterDenial::Overflow)?;
        let intact_frames = checked_increment(self.intact_frames)?;
        self.inspected_frames = inspected_frames;
        self.inspected_bytes = inspected_bytes;
        self.intact_frames = intact_frames;
        Ok(())
    }

    pub(crate) fn record_rejected(
        &mut self,
        byte_count: u64,
        rejection_class: PhysicalIntegrityRejectionClass,
    ) -> Result<(), PhysicalIntegrityCounterDenial> {
        let inspected_frames = checked_increment(self.inspected_frames)?;
        let inspected_bytes = self
            .inspected_bytes
            .checked_add(byte_count)
            .ok_or(PhysicalIntegrityCounterDenial::Overflow)?;
        let rejected_frames = checked_increment(self.rejected_frames)?;
        let rejection_index = rejection_class.index();
        let class_count = checked_increment(self.rejections_by_class[rejection_index])?;
        self.inspected_frames = inspected_frames;
        self.inspected_bytes = inspected_bytes;
        self.rejected_frames = rejected_frames;
        self.rejections_by_class[rejection_index] = class_count;
        Ok(())
    }

    pub(crate) fn one_intact(family: PhysicalIntegrityArtifactFamily, byte_count: u64) -> Self {
        let mut counters = Self::empty(family);
        counters
            .record_intact(byte_count)
            .expect("one bounded validation cannot overflow counters");
        counters
    }

    pub(crate) fn one_rejected(
        family: PhysicalIntegrityArtifactFamily,
        byte_count: u64,
        rejection: PhysicalIntegrityRejection,
    ) -> Self {
        let mut counters = Self::empty(family);
        counters
            .record_rejected(byte_count, rejection_class(rejection))
            .expect("one bounded validation cannot overflow counters");
        counters
    }

    pub const fn family(self) -> PhysicalIntegrityArtifactFamily {
        self.family
    }

    pub const fn inspected_frames(self) -> u64 {
        self.inspected_frames
    }

    pub const fn inspected_bytes(self) -> u64 {
        self.inspected_bytes
    }

    pub const fn intact_frames(self) -> u64 {
        self.intact_frames
    }

    pub const fn rejected_frames(self) -> u64 {
        self.rejected_frames
    }

    pub const fn rejected_for(self, rejection_class: PhysicalIntegrityRejectionClass) -> u64 {
        self.rejections_by_class[rejection_class.index()]
    }
}

const fn rejection_class(rejection: PhysicalIntegrityRejection) -> PhysicalIntegrityRejectionClass {
    match rejection {
        PhysicalIntegrityRejection::Damaged(localization) => {
            PhysicalIntegrityRejectionClass::Damaged(localization.cause())
        }
        PhysicalIntegrityRejection::Unsupported(_) => PhysicalIntegrityRejectionClass::Unsupported,
        PhysicalIntegrityRejection::Unknown(_) => PhysicalIntegrityRejectionClass::Unknown,
        PhysicalIntegrityRejection::Indeterminate(_) => {
            PhysicalIntegrityRejectionClass::Indeterminate
        }
    }
}

fn checked_increment(value: u64) -> Result<u64, PhysicalIntegrityCounterDenial> {
    value
        .checked_add(1)
        .ok_or(PhysicalIntegrityCounterDenial::Overflow)
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;

    use super::{PhysicalIntegrityObservationCounters, PhysicalIntegrityRejectionClass};
    use crate::localization::PhysicalDamageCause;

    #[test]
    fn rejection_counts_preserve_family_and_cause() {
        let mut counters = PhysicalIntegrityObservationCounters::empty(
            PhysicalIntegrityArtifactFamily::CurrentRootSelector,
        );
        counters
            .record_rejected(
                107,
                PhysicalIntegrityRejectionClass::Damaged(PhysicalDamageCause::ChecksumMismatch),
            )
            .unwrap();
        assert_eq!(counters.inspected_frames(), 1);
        assert_eq!(counters.inspected_bytes(), 107);
        assert_eq!(
            counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(
                PhysicalDamageCause::ChecksumMismatch
            )),
            1
        );
        assert_eq!(
            counters.rejected_for(PhysicalIntegrityRejectionClass::Unsupported),
            0
        );
    }

    #[test]
    fn phase_four_rejection_causes_have_distinct_counter_slots() {
        let mut counters = PhysicalIntegrityObservationCounters::empty(
            PhysicalIntegrityArtifactFamily::CheckpointFooter,
        );
        let causes = [
            PhysicalDamageCause::RecordKindMismatch,
            PhysicalDamageCause::SequenceMismatch,
            PhysicalDamageCause::AggregateMismatch,
        ];
        for cause in causes {
            counters
                .record_rejected(64, PhysicalIntegrityRejectionClass::Damaged(cause))
                .unwrap();
        }
        for cause in causes {
            assert_eq!(
                counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(cause)),
                1
            );
        }
        assert_eq!(
            counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(
                PhysicalDamageCause::ChecksumMismatch
            )),
            0
        );
    }
}
