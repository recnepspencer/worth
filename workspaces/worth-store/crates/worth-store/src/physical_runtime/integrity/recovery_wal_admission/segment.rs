//! Sealed allocation-first roster construction for one exact C4 observation.

use super::allocation::{
    allocator_denial, arc_bytes, vector_bytes, RecoveryWalAllocationDenial, WalAllocation,
};
use super::{IntegrityAdmittedRecoveryWalFrame, RecoveryWalIntegrityAdmissionDenial as Denial};
use crate::physical_runtime::PhysicalRecoveryCoordination;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::ObservedWalArtifact;
use worth_store_wal::{
    LogSequenceNumber, WalLsnRange, WalSegmentArtifactIdentity, WalSegmentInspection,
};

#[derive(Debug, Clone)]
pub struct IntegrityAdmittedRecoveryWalSegment {
    backing: Arc<AdmittedWalSegmentData>,
}

#[derive(Debug)]
struct AdmittedWalSegmentData {
    inspection: WalSegmentInspection,
    frames: Vec<IntegrityAdmittedRecoveryWalFrame>,
    grant: OperationAllocationGrant,
}

/// No constructor or raw-roster import is public. The issuer binds the source
/// and the original native Recovery ceiling before any roster allocation.
pub struct IntegrityAdmittedRecoveryWalSegmentBuilder<'coordination, 'source> {
    source: &'source ObservedWalArtifact,
    identity: WalSegmentArtifactIdentity,
    allocation: WalAllocation<'coordination>,
    frames: Vec<IntegrityAdmittedRecoveryWalFrame>,
    grant: Option<OperationAllocationGrant>,
    poisoned: Option<RecoveryWalAllocationDenial>,
}

impl<'coordination, 'source> IntegrityAdmittedRecoveryWalSegmentBuilder<'coordination, 'source> {
    pub(in crate::physical_runtime) fn begin(
        coordination: &'coordination PhysicalRecoveryCoordination,
        source: &'source ObservedWalArtifact,
        identity: WalSegmentArtifactIdentity,
    ) -> Result<Self, Denial> {
        let allocation = WalAllocation::from_coordination(coordination)?;
        Ok(Self {
            source,
            identity,
            allocation,
            frames: Vec::new(),
            grant: None,
            poisoned: None,
        })
    }

    pub fn frames(&self) -> &[IntegrityAdmittedRecoveryWalFrame] {
        &self.frames
    }
    pub fn len(&self) -> usize {
        self.frames.len()
    }
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn push(&mut self, frame: IntegrityAdmittedRecoveryWalFrame) -> Result<(), Denial> {
        self.require_usable()?;
        self.validate_next(&frame)?;
        self.prepare_append()?;
        self.frames.push(frame);
        Ok(())
    }

    pub fn finish(self) -> Result<IntegrityAdmittedRecoveryWalSegment, Denial> {
        self.require_usable()?;
        let first = self.frames.first().ok_or(Denial::MissingBoundedArtifact)?;
        let last = self.frames.last().ok_or(Denial::MissingBoundedArtifact)?;
        let byte_count = self
            .frames
            .iter()
            .try_fold(0u64, |bytes, frame| {
                bytes.checked_add(frame.encoded_byte_count())
            })
            .ok_or(RecoveryWalAllocationDenial::SizeOverflow)?;
        let mut artifact_digest = Sha256::new();
        for frame in &self.frames {
            artifact_digest.update(&frame.backing.encoded);
        }
        let range = WalLsnRange::new(
            LogSequenceNumber::new(first.lsn_start()),
            LogSequenceNumber::new(last.lsn_end()),
        )
        .map_err(|_| Denial::ScopeMismatch)?;
        let inspection = WalSegmentInspection::from_admitted_frames(
            self.identity,
            range,
            self.frames.len() as u64,
            byte_count,
            artifact_digest.finalize().into(),
        )
        .ok_or(Denial::ScopeMismatch)?;
        let grant = self
            .grant
            .ok_or(RecoveryWalAllocationDenial::SizeOverflow)?;
        Ok(IntegrityAdmittedRecoveryWalSegment {
            backing: Arc::new(AdmittedWalSegmentData {
                inspection,
                frames: self.frames,
                grant,
            }),
        })
    }

    fn validate_next(&self, frame: &IntegrityAdmittedRecoveryWalFrame) -> Result<(), Denial> {
        if !self.allocation.owns(&frame.backing.grant)
            || frame.source_incarnation != self.source.observation_identity()
            || frame.source_name() != self.source.name()
            || frame.source_entry_type() != self.source.entry_type()
            || frame.scope().store_identity() != self.source.store_identity()
            || frame.segment_identity() != self.identity.format_identity()
        {
            return Err(Denial::SourceIncarnationMismatch);
        }
        let expected = self
            .frames
            .last()
            .map_or(0, |previous| previous.scope().byte_range().end_exclusive());
        let range = frame.scope().byte_range();
        if range.offset() != expected
            || self
                .frames
                .last()
                .is_some_and(|previous| previous.lsn_end() != frame.lsn_start())
        {
            return Err(Denial::ScopeMismatch);
        }
        let bytes = self.source.bytes().ok_or(Denial::MissingBoundedArtifact)?;
        let source = usize::try_from(range.offset())
            .ok()
            .zip(usize::try_from(range.end_exclusive()).ok())
            .and_then(|(start, end)| bytes.get(start..end));
        if source != Some(frame.backing.encoded.as_slice()) {
            return Err(Denial::SourceIncarnationMismatch);
        }
        Ok(())
    }

    fn prepare_append(&mut self) -> Result<(), Denial> {
        if self.frames.len() < self.frames.capacity() {
            return Ok(());
        }
        let old_capacity = self.frames.capacity();
        let prospective = old_capacity
            .checked_mul(2)
            .map(|capacity| capacity.max(1))
            .ok_or(RecoveryWalAllocationDenial::SizeOverflow)?;
        let control = arc_bytes::<AdmittedWalSegmentData>()?;
        let old = vector_bytes::<IntegrityAdmittedRecoveryWalFrame>(old_capacity)?;
        let next = vector_bytes::<IntegrityAdmittedRecoveryWalFrame>(prospective)?;
        let peak = control
            .checked_add(old)
            .and_then(|bytes| bytes.checked_add(next))
            .ok_or(RecoveryWalAllocationDenial::SizeOverflow)?;
        match &mut self.grant {
            Some(grant) => self.allocation.resize(grant, peak)?,
            None => self.grant = Some(self.allocation.reserve(peak)?),
        }
        self.frames
            .try_reserve_exact(prospective - self.frames.len())
            .map_err(|cause| allocator_denial(next, cause))?;
        let actual = self.accept_allocated_roster(next)?;
        // Vec growth has disposed the old allocation; the future Arc control
        // remains funded until finish, cancellation or final shared disposal.
        let retained = control
            .checked_add(actual)
            .ok_or(RecoveryWalAllocationDenial::SizeOverflow)?;
        self.allocation.resize(
            self.grant.as_mut().expect("append prepared native grant"),
            retained,
        )?;
        Ok(())
    }

    fn require_usable(&self) -> Result<(), Denial> {
        match &self.poisoned {
            Some(cause) => Err(cause.clone().into()),
            None => Ok(()),
        }
    }

    fn accept_allocated_roster(&mut self, requested: u64) -> Result<u64, Denial> {
        match vector_bytes::<IntegrityAdmittedRecoveryWalFrame>(self.frames.capacity()) {
            Ok(actual) if actual <= requested => Ok(actual),
            result => {
                let cause = match result {
                    Ok(actual) => RecoveryWalAllocationDenial::AllocatorExceededReservation {
                        requested,
                        actual,
                    },
                    Err(cause) => cause,
                };
                // An already oversized allocation cannot be justified by a
                // later grant. Dispose it under the existing charge, then
                // prevent this reusable owner from publishing or appending.
                drop(std::mem::take(&mut self.frames));
                drop(self.grant.take());
                self.poisoned = Some(cause.clone());
                Err(cause.into())
            }
        }
    }
}

impl IntegrityAdmittedRecoveryWalSegment {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let roster = arc_bytes::<AdmittedWalSegmentData>().ok()?.checked_add(
            vector_bytes::<IntegrityAdmittedRecoveryWalFrame>(self.backing.frames.capacity())
                .ok()?,
        )?;
        self.frames().iter().try_fold(roster, |bytes, frame| {
            bytes.checked_add(frame.owned_heap_bytes()?)
        })
    }
    pub fn charged_bytes(&self) -> u64 {
        self.backing.grant.bytes()
    }
    pub fn inspection(&self) -> WalSegmentInspection {
        self.backing.inspection
    }
    pub fn frames(&self) -> &[IntegrityAdmittedRecoveryWalFrame] {
        &self.backing.frames
    }
}

#[cfg(test)]
mod tests;
