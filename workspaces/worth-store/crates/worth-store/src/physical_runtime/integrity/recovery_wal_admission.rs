//! Store-owned C9 WAL evidence with native backing shared by every clone.

use crate::physical_runtime::PhysicalRecoveryCoordination;
use std::{
    ffi::{OsStr, OsString},
    sync::Arc,
};
use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::ObservedWalArtifact;
use worth_store_physical_format::{store_namespace::NamespaceEntryType, WalSegmentIdentity};
use worth_store_physical_integrity::{
    IntegrityValidatedWalFrame, PhysicalArtifactScope, PhysicalByteRange,
    UntrustedPhysicalArtifact, WalPayloadProjectionDenial,
};

mod allocation;
mod segment;
use allocation::{allocator_denial, arc_bytes, WalAllocation};
pub use allocation::{PhysicalRecoveryWalInventoryBacking, RecoveryWalAllocationDenial};
pub use segment::{
    IntegrityAdmittedRecoveryWalSegment, IntegrityAdmittedRecoveryWalSegmentBuilder,
};

/// Exact C9 metadata is inline. Cloning shares bytes, source name and their
/// actual native reservation; no deep copy or uncharged byte owner escapes.
#[derive(Debug, Clone)]
pub struct IntegrityAdmittedRecoveryWalFrame {
    source_entry_type: NamespaceEntryType,
    source_incarnation: crate::physical_runtime::RecoveryWalObservationIdentity,
    scope: PhysicalArtifactScope,
    segment_identity: WalSegmentIdentity,
    lsn_start: u64,
    lsn_end: u64,
    identity_digest: [u8; 32],
    payload_digest: [u8; 32],
    payload_range: std::ops::Range<usize>,
    backing: Arc<AdmittedWalFrameData>,
}

#[derive(Debug)]
struct AdmittedWalFrameData {
    source_name: OsString,
    encoded: Vec<u8>,
    // Storage and the shared control object are disposed before this grant.
    grant: OperationAllocationGrant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryWalIntegrityAdmissionDenial {
    MissingBoundedArtifact,
    ScopeMismatch,
    SourceRangeOutsideObservation,
    SourceIncarnationMismatch,
    Allocation(RecoveryWalAllocationDenial),
}

impl From<RecoveryWalAllocationDenial> for RecoveryWalIntegrityAdmissionDenial {
    fn from(cause: RecoveryWalAllocationDenial) -> Self {
        Self::Allocation(cause)
    }
}

impl IntegrityAdmittedRecoveryWalFrame {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        arc_bytes::<AdmittedWalFrameData>()
            .ok()?
            .checked_add(u64::try_from(self.backing.source_name.capacity()).ok()?)?
            .checked_add(u64::try_from(self.backing.encoded.capacity()).ok()?)
    }

    pub fn charged_bytes(&self) -> u64 {
        self.backing.grant.bytes()
    }

    pub(in crate::physical_runtime) fn bind(
        coordination: &PhysicalRecoveryCoordination,
        observed: &ObservedWalArtifact,
        expected_scope: PhysicalArtifactScope,
        relative_range: PhysicalByteRange,
        validated: IntegrityValidatedWalFrame<'_>,
    ) -> Result<Self, RecoveryWalIntegrityAdmissionDenial> {
        let source = validate_frame_source(observed, expected_scope, relative_range, &validated)?;
        let backing = AdmittedWalFrameData::prepare(coordination, &source)?;
        Ok(Self {
            source_entry_type: observed.entry_type(),
            source_incarnation: observed.observation_identity(),
            scope: expected_scope,
            segment_identity: validated.segment_identity(),
            lsn_start: validated.lsn_start(),
            lsn_end: validated.lsn_end(),
            identity_digest: validated.identity_digest(),
            payload_digest: validated.payload_digest(),
            payload_range: source.payload_range,
            backing,
        })
    }

    pub fn source_name(&self) -> &OsStr {
        &self.backing.source_name
    }
    pub const fn source_entry_type(&self) -> NamespaceEntryType {
        self.source_entry_type
    }
    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }
    pub const fn segment_identity(&self) -> WalSegmentIdentity {
        self.segment_identity
    }
    pub const fn lsn_start(&self) -> u64 {
        self.lsn_start
    }
    pub const fn lsn_end(&self) -> u64 {
        self.lsn_end
    }
    pub fn lsn_range(&self) -> worth_store_wal::WalLsnRange {
        worth_store_wal::WalLsnRange::new(
            worth_store_wal::LogSequenceNumber::new(self.lsn_start),
            worth_store_wal::LogSequenceNumber::new(self.lsn_end),
        )
        .expect("C9 admitted WAL frame carries an ordered LSN range")
    }
    pub const fn identity_digest(&self) -> [u8; 32] {
        self.identity_digest
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub fn encoded_byte_count(&self) -> u64 {
        self.backing.encoded.len() as u64
    }
    pub fn payload_byte_count(&self) -> u64 {
        self.payload_range.len() as u64
    }
    pub(in crate::physical_runtime) fn payload(&self) -> &[u8] {
        &self.backing.encoded[self.payload_range.clone()]
    }
}

// Borrowed source meaning only: this creates no new admission authority.
struct ValidatedWalFrameSource<'source> {
    source_name: &'source OsStr,
    encoded: &'source [u8],
    payload_range: std::ops::Range<usize>,
}

fn validate_frame_source<'source>(
    observed: &'source ObservedWalArtifact,
    expected_scope: PhysicalArtifactScope,
    relative_range: PhysicalByteRange,
    validated: &IntegrityValidatedWalFrame<'_>,
) -> Result<ValidatedWalFrameSource<'source>, RecoveryWalIntegrityAdmissionDenial> {
    use RecoveryWalIntegrityAdmissionDenial as Denial;
    if expected_scope != validated.scope()
        || relative_range != expected_scope.byte_range()
        || observed.store_identity() != expected_scope.store_identity()
    {
        return Err(Denial::ScopeMismatch);
    }
    let bytes = observed.bytes().ok_or(Denial::MissingBoundedArtifact)?;
    let start = usize::try_from(relative_range.offset())
        .map_err(|_| Denial::SourceRangeOutsideObservation)?;
    let end = usize::try_from(relative_range.end_exclusive())
        .map_err(|_| Denial::SourceRangeOutsideObservation)?;
    let encoded = bytes
        .get(start..end)
        .ok_or(Denial::SourceRangeOutsideObservation)?;
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(encoded);
    if !validated.matches_input(input) {
        return Err(Denial::SourceIncarnationMismatch);
    }
    let payload = validated
        .project_payload(input, validated.segment_identity())
        .map_err(map_projection_denial)?;
    Ok(ValidatedWalFrameSource {
        source_name: observed.name(),
        encoded,
        payload_range: payload.payload_range(),
    })
}

impl AdmittedWalFrameData {
    fn prepare(
        coordination: &PhysicalRecoveryCoordination,
        source: &ValidatedWalFrameSource<'_>,
    ) -> Result<Arc<Self>, RecoveryWalIntegrityAdmissionDenial> {
        let allocation = WalAllocation::from_coordination(coordination)?;
        let encoded = source.encoded;
        let name_bytes = source.source_name.as_encoded_bytes().len();
        let requested = arc_bytes::<AdmittedWalFrameData>()?
            .checked_add(name_bytes as u64)
            .and_then(|bytes| bytes.checked_add(encoded.len() as u64))
            .ok_or(RecoveryWalAllocationDenial::SizeOverflow)?;
        let grant = allocation.reserve(requested)?;
        let mut source_name = OsString::new();
        source_name
            .try_reserve_exact(name_bytes)
            .map_err(|cause| allocator_denial(name_bytes as u64, cause))?;
        if source_name.capacity() > name_bytes {
            return Err(RecoveryWalAllocationDenial::AllocatorExceededReservation {
                requested: name_bytes as u64,
                actual: source_name.capacity() as u64,
            }
            .into());
        }
        source_name.push(source.source_name);
        let mut retained = Vec::new();
        retained
            .try_reserve_exact(encoded.len())
            .map_err(|cause| allocator_denial(encoded.len() as u64, cause))?;
        if retained.capacity() > encoded.len() {
            return Err(RecoveryWalAllocationDenial::AllocatorExceededReservation {
                requested: encoded.len() as u64,
                actual: retained.capacity() as u64,
            }
            .into());
        }
        retained.extend_from_slice(encoded);
        Ok(Arc::new(Self {
            source_name,
            encoded: retained,
            grant,
        }))
    }
}

fn map_projection_denial(
    denial: WalPayloadProjectionDenial,
) -> RecoveryWalIntegrityAdmissionDenial {
    match denial {
        WalPayloadProjectionDenial::InputIncarnationMismatch => {
            RecoveryWalIntegrityAdmissionDenial::SourceIncarnationMismatch
        }
        WalPayloadProjectionDenial::SegmentIdentityMismatch => {
            RecoveryWalIntegrityAdmissionDenial::ScopeMismatch
        }
    }
}

#[cfg(test)]
mod media_generation_tests;
#[cfg(test)]
mod tests;
