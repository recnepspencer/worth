use worth_store_physical_backend::ArtifactTreeFailure;

use super::super::RecordStreamFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordPlacementClass {
    InlinePage,
    ExtentBacked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordAppendDenial {
    EmptyBatch,
    BatchRecordLimitExceeded,
    BatchByteLimitExceeded,
    RecordTooLarge,
    InlinePageFull,
    RootGenerationExhausted,
    RecordIdentityExhausted,
    PhysicalIdentityExhausted,
    IdentityEntropyUnavailable,
    BackendUnavailable(ArtifactTreeFailure),
    ServingRequiresInspection,
    PublicationAuthorityReleased,
    PhysicalWorkUnavailable(Box<super::super::PhysicalRecordMutationFailureEvidence>),
    PhysicalReadWorkUnavailable(super::super::RecordReadWorkDenial),
    PlacementFormatMismatch,
    ManifestCapacityMigrationRequired,
    PublishedLayoutDamaged,
    /// A selected rewrite names no live source: a span reaches a frame the
    /// current root does not read from the tail generation, or a selected
    /// extent rewrite names a record the current root does not place in an
    /// extent.
    RewriteSpanNotLive,
    PhysicalPressure,
    /// Retained-storage growth for the candidate would exceed the hard allowance
    /// left after progress headroom.
    RetentionPressure,
    ResidencyUnavailable(super::super::PhysicalRecordResidencyFailure),
}

/// Failure to publish an ordinary physical record append.
///
/// Physical-pressure failures retain exact Store-owned evidence through
/// `pressure`; lower pool authority is never exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordAppendError {
    Denied(RecordAppendDenial),
    PhysicalPressure {
        evidence: super::super::PhysicalRecordPressureEvidence,
    },
    StreamFailed(RecordStreamFailure),
}

impl RecordAppendError {
    /// Returns exact physical-pressure evidence when pressure caused failure.
    ///
    /// The evidence describes the observed denial and is not retry authority.
    pub const fn pressure(&self) -> Option<super::super::PhysicalRecordPressureEvidence> {
        match self {
            Self::PhysicalPressure { evidence } => Some(*evidence),
            Self::StreamFailed(failure) => failure.pressure(),
            _ => None,
        }
    }

    /// Returns the broad pressure classification without discarding evidence.
    pub const fn pressure_denial(&self) -> Option<RecordAppendDenial> {
        match self {
            Self::PhysicalPressure { .. } => Some(RecordAppendDenial::PhysicalPressure),
            Self::StreamFailed(failure) if failure.pressure().is_some() => {
                Some(RecordAppendDenial::PhysicalPressure)
            }
            _ => None,
        }
    }
}

impl RecordAppendDenial {
    pub(in crate::physical_runtime::record_serving) fn from_residency(
        denial: worth_store_buffer_pool::PhysicalResidencyDenial,
    ) -> Self {
        Self::ResidencyUnavailable(denial.into())
    }
}

pub(in crate::physical_runtime::record_serving) fn next_nonzero_random(
) -> Result<u64, RecordAppendError> {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes)
        .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::IdentityEntropyUnavailable))?;
    let value = u64::from_le_bytes(bytes);
    if value == 0 {
        return Err(RecordAppendError::Denied(
            RecordAppendDenial::IdentityEntropyUnavailable,
        ));
    }
    Ok(value)
}
