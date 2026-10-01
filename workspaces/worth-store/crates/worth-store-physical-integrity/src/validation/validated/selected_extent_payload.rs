//! Sealed digest of a selected extent payload assembled only from exact C.9
//! validated manifest membership and every original validated chunk frame.
//! A caller-provided byte vector or digest cannot construct this witness.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurableExtentRecordPlacement, DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};

use super::super::{PhysicalArtifactScope, UntrustedPhysicalArtifact};
use super::{IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentMembership};
use crate::PhysicalByteRange;

#[derive(Debug)]
pub struct SelectedExtentPayloadBuilder {
    membership: IntegrityValidatedExtentMembership,
    placement: DurableExtentRecordPlacement,
    next_ordinal: u32,
    next_offset: u64,
    digest: Sha256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegrityValidatedSelectedExtentPayload {
    placement: DurableExtentRecordPlacement,
    payload_sha256: [u8; 32],
    logical_bytes: u64,
}

impl SelectedExtentPayloadBuilder {
    pub fn new(
        membership: IntegrityValidatedExtentMembership,
        placement: DurableExtentRecordPlacement,
    ) -> Option<Self> {
        if membership.scope().extent_manifest_placement() != Some(placement)
            || membership.logical_bytes() != placement.payload_bytes()
        {
            return None;
        }
        Some(Self {
            membership,
            placement,
            next_ordinal: 1,
            next_offset: 0,
            digest: Sha256::new(),
        })
    }

    pub fn append(
        &mut self,
        validated: &IntegrityValidatedExtentChunkFrame<'_>,
        exact_input: UntrustedPhysicalArtifact<'_>,
    ) -> Option<()> {
        let expected = self.membership.chunk_membership(self.next_ordinal)?;
        let scope = self.membership.scope();
        let relative = self
            .membership
            .frame_layout()
            .chunk_offset(self.next_ordinal)?;
        let absolute = self
            .membership
            .arena_range()
            .offset()
            .checked_add(relative)?;
        let frame_length = (DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES) as u64
            + expected.payload_bytes();
        let range = PhysicalByteRange::new(absolute, frame_length).ok()?;
        let expected_scope = PhysicalArtifactScope::extent_chunk(
            scope.store_identity(),
            self.membership.record_format(),
            expected.coordinate(),
            range,
            self.membership.arena_range(),
        );
        if validated.scope() != expected_scope {
            return None;
        }
        let projection = validated
            .project_chunk(exact_input, expected.coordinate())
            .ok()?;
        let bytes = exact_input.bytes().get(projection.payload_range())?;
        if expected.coordinate().logical_offset() != self.next_offset
            || bytes.len() as u64 != expected.payload_bytes()
        {
            return None;
        }
        self.digest.update(bytes);
        self.next_offset = self.next_offset.checked_add(bytes.len() as u64)?;
        self.next_ordinal = self.next_ordinal.checked_add(1)?;
        Some(())
    }

    pub fn finish(self) -> Option<IntegrityValidatedSelectedExtentPayload> {
        if self.next_offset != self.membership.logical_bytes()
            || self
                .membership
                .chunk_membership(self.next_ordinal)
                .is_some()
        {
            return None;
        }
        Some(IntegrityValidatedSelectedExtentPayload {
            placement: self.placement,
            payload_sha256: self.digest.finalize().into(),
            logical_bytes: self.next_offset,
        })
    }
}

impl IntegrityValidatedSelectedExtentPayload {
    pub const fn placement(self) -> DurableExtentRecordPlacement {
        self.placement
    }

    pub const fn payload_sha256(self) -> [u8; 32] {
        self.payload_sha256
    }

    pub const fn logical_bytes(self) -> u64 {
        self.logical_bytes
    }

    pub fn matches_frame(self, bytes: &[u8]) -> bool {
        bytes.len() as u64 == self.logical_bytes
            && <[u8; 32]>::from(Sha256::digest(bytes)) == self.payload_sha256
    }
}
