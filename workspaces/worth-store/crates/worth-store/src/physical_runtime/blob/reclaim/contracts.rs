use std::num::{NonZeroU16, NonZeroU64};

use worth_proof::AdmittedBlobReleaseProof;
use worth_store_physical_format::BlobRecordDenial;

use crate::physical_runtime::{
    AdmittedBlobScope, AdmittedRecordPlacementPolicy, BlobIngestClaimDenial, BlobResumeToken,
    PhysicalMutationDeadline, PhysicalReadProtectionDenial, PhysicalScopedAllocationFailure,
    RecordReadError, RecordScanError, RecordStreamFailure,
};

/// A released Drop's decoded source directory plus its canonical successor
/// frame, admitted once inside the reclaim envelope.
pub(super) const RELEASED_DIRECTORY_REBINDING_BYTES: u64 = 16 * 1024;

/// Bounds each selected-root pass and the combined payload inspection work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReclaimLimits {
    maximum_selected_records: NonZeroU64,
    maximum_inspected_bytes: NonZeroU64,
    maximum_dropped_records: NonZeroU16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReclaimLimitDenial {
    DropSetTooLarge,
    MetadataSizeOverflow,
}

impl BlobReclaimLimits {
    pub fn new(
        maximum_selected_records: NonZeroU64,
        maximum_inspected_bytes: NonZeroU64,
        maximum_dropped_records: NonZeroU16,
    ) -> Result<Self, BlobReclaimLimitDenial> {
        if maximum_dropped_records.get() > 1024 {
            return Err(BlobReclaimLimitDenial::DropSetTooLarge);
        }
        let limits = Self {
            maximum_selected_records,
            maximum_inspected_bytes,
            maximum_dropped_records,
        };
        limits.memory_bytes()?;
        Ok(limits)
    }

    pub const fn maximum_selected_records(self) -> u64 {
        self.maximum_selected_records.get()
    }

    pub const fn maximum_inspected_bytes(self) -> u64 {
        self.maximum_inspected_bytes.get()
    }

    pub const fn maximum_dropped_records(self) -> u16 {
        self.maximum_dropped_records.get()
    }

    pub(super) fn memory_bytes(self) -> Result<NonZeroU64, BlobReclaimLimitDenial> {
        // The failed-ingest and released-generation selectors do not run at
        // once. Charge the larger peak: selected rows and chain links, all
        // bounded historical IDs, and the traversal/protection stacks. The
        // fixed budget covers the read window and one decoded maximum tree.
        let failed_per_record = std::mem::size_of::<super::selection::ResidueCandidate>()
            + std::mem::size_of::<super::manifest_residue::DescriptorLink>()
            + std::mem::size_of::<super::manifest_residue::ReservedLink>()
            + super::selection::SELECTION_ROSTER_ENTRY_BYTES
            + 32;
        let released_per_record = super::released::SELECTION_ROSTER_ENTRY_BYTES;
        self.maximum_selected_records()
            .checked_mul(failed_per_record.max(released_per_record) as u64)
            .and_then(|bytes| {
                bytes.checked_add(
                    u64::from(self.maximum_dropped_records())
                        * std::mem::size_of::<worth_store_physical_format::PersistedRecordIdentity>(
                        ) as u64,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(2 * 1024 * 1024 + RELEASED_DIRECTORY_REBINDING_BYTES)
            })
            .and_then(NonZeroU64::new)
            .ok_or(BlobReclaimLimitDenial::MetadataSizeOverflow)
    }
}

/// A semantic release proof names exactly one published generation. A resume
/// token instead names abandoned ingest custody; neither is drop authority by
/// itself, because selected physical records must still be inspected.
pub struct BlobReclaimRequest<'scope> {
    pub(super) source: BlobReclaimSource<'scope>,
    pub(super) placement: AdmittedRecordPlacementPolicy,
    pub(super) deadline: PhysicalMutationDeadline,
    pub(super) limits: BlobReclaimLimits,
}

pub(super) enum BlobReclaimSource<'scope> {
    Abandoned {
        token: BlobResumeToken,
        scope: &'scope AdmittedBlobScope,
    },
    Released(AdmittedBlobReleaseProof),
}

impl<'scope> BlobReclaimRequest<'scope> {
    pub const fn abandoned(
        token: BlobResumeToken,
        scope: &'scope AdmittedBlobScope,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        limits: BlobReclaimLimits,
    ) -> Self {
        Self {
            source: BlobReclaimSource::Abandoned { token, scope },
            placement,
            deadline,
            limits,
        }
    }

    pub const fn released(
        proof: AdmittedBlobReleaseProof,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        limits: BlobReclaimLimits,
    ) -> Self {
        Self {
            source: BlobReclaimSource::Released(proof),
            placement,
            deadline,
            limits,
        }
    }
}

#[derive(Debug)]
pub enum BlobReclaimFailure {
    ServingRequiresInspection,
    Claim(BlobIngestClaimDenial),
    Allocation(PhysicalScopedAllocationFailure),
    ScratchUnavailable,
    ScanBoundExhausted,
    InspectedByteBoundExhausted,
    InspectionWindowExhausted,
    ForeignStore,
    DeclarationMismatch,
    ScopeMismatch,
    NotAbandoned,
    AlreadyPublished,
    ConflictingSelectedFate,
    Format(BlobRecordDenial),
    ReadProtection(PhysicalReadProtectionDenial),
    Read(RecordReadError),
    ReleasedDirectoryRead(crate::physical_runtime::layout::PhysicalLayoutPageReadFailure),
    Stream(RecordStreamFailure),
    Scan(RecordScanError),
    Deferred(super::BlobReclaimDeferral),
    /// Mandatory release-certificate backing denied before any control or
    /// root publication. Retains the actual budget or allocator boundary.
    ReleaseCertificateBacking(crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial),
    /// Selection denied before the reclaim attempt or any control publication.
    ReleaseCustodySelection(crate::physical_runtime::SelectedReleaseHeadDenial),
    /// Manifest and reservation are already selected; retain their identities
    /// so recovery can reconcile the denied pending-drop certification.
    ReleaseCustodyCertification {
        manifest_record: worth_store_physical_format::PersistedRecordIdentity,
        reservation_record: worth_store_physical_format::PersistedRecordIdentity,
        cause: crate::physical_runtime::SelectedReleaseHeadDenial,
    },
    AttemptIdentityUnavailable,
    RouteUnavailable,
    FenceLost,
    ManifestResiduePublication(crate::physical_runtime::PhysicalRetirementDenial),
    /// The manifest is durable selected custody, but the descriptor is proved
    /// absent. Ordinary serving is unfenced; later bounded cleanup owns residue.
    ManifestRetained {
        manifest_record: worth_store_physical_format::PersistedRecordIdentity,
        cause: crate::physical_runtime::BlobAppendFailure,
    },
    Publication {
        stage: super::BlobReclaimPublicationStage,
        manifest_record: Option<worth_store_physical_format::PersistedRecordIdentity>,
        cause: crate::physical_runtime::BlobAppendFailure,
    },
}

impl From<super::super::ingest::selected_session::SelectedSessionFailure> for BlobReclaimFailure {
    fn from(cause: super::super::ingest::selected_session::SelectedSessionFailure) -> Self {
        use super::super::ingest::selected_session::SelectedSessionFailure as Cause;
        match cause {
            Cause::Format(cause) => Self::Format(cause),
            Cause::Read(cause) => Self::Read(cause),
            Cause::Stream(cause) => Self::Stream(cause),
            Cause::ForeignStore => Self::ForeignStore,
            Cause::DeclarationMismatch => Self::DeclarationMismatch,
            Cause::ScopeMismatch => Self::ScopeMismatch,
        }
    }
}
