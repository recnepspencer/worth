use super::super::{selected_segment_rewrite::damaged, RecordPublicationDirector};
use super::session::{CopyPhase, ExtentCopySession};
use crate::physical_runtime::record_serving::{
    CanonicalRecordReadFailure, RecordAppendDenial, RecordAppendError,
};
use sha2::Digest;
use worth_store_physical_format::{
    ExtentArenaFrameLayout, ExtentChunkCoordinate, PhysicalPageLsn, RecordArtifactFile,
    RecordFrameCoordinate, DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_physical_integrity::{
    validate_extent_arena_frame, ExtentArenaFrameExpectation, ExtentArenaFrameIntegrityValidation,
    PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact,
};

impl RecordPublicationDirector {
    /// Acquires one synchronized destination frame through scheduled physical
    /// work, then validates its exact C9 arena scope before publication.
    pub(super) fn verify_copy_quantum(
        &self,
        copy: &mut ExtentCopySession,
    ) -> Result<(), RecordAppendError> {
        let intent = copy.intent.ok_or_else(damaged)?;
        let manifest = intent
            .destination_manifest(self.format.declaration())
            .ok_or_else(damaged)?;
        let range = intent.destination().arena_range();
        let artifact = RecordArtifactFile::ExtentArena {
            arena: range.arena().get(),
        };
        if copy.next_ordinal == 0 {
            let expected = manifest.encode(self.format.declaration());
            let coordinate =
                RecordFrameCoordinate::new(artifact, range.offset(), expected.len() as u32)
                    .ok_or_else(damaged)?;
            let scope_range = PhysicalByteRange::new(range.offset(), expected.len() as u64)
                .map_err(|_| damaged())?;
            let (observed, failure) = self.read_copy_frame_fresh(coordinate)?;
            let input = UntrustedPhysicalArtifact::from_bounded_bytes(&observed);
            let scope = PhysicalArtifactScope::extent_manifest(
                self.durability.store_identity(),
                self.format.declaration(),
                intent.destination(),
                scope_range,
            );
            match validate_extent_arena_frame(input, ExtentArenaFrameExpectation::Manifest(scope)).0
            {
                ExtentArenaFrameIntegrityValidation::Manifest(validated)
                    if observed.as_ref() == expected =>
                {
                    copy.verified_membership = Some(validated.membership());
                }
                _ => {
                    failure.consume();
                    return Err(damaged());
                }
            }
            copy.next_ordinal = 1;
            return Ok(());
        }
        if copy.next_ordinal > intent.chunk_count() {
            let digest: [u8; 32] = copy.digest.clone().finalize().into();
            if digest != intent.source_digest() || copy.completed != intent.source().payload_bytes()
            {
                return Err(damaged());
            }
            copy.phase = CopyPhase::Complete;
            return Ok(());
        }
        let layout = ExtentArenaFrameLayout::new(self.format.declaration(), intent.alignment())
            .ok_or_else(damaged)?;
        let expected_payload = (manifest.logical_bytes() - copy.completed)
            .min(u64::from(manifest.chunk_payload_capacity()))
            as usize;
        let length =
            DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES + expected_payload;
        let offset = range.offset()
            + layout.manifest_stride()
            + u64::from(copy.next_ordinal - 1) * layout.chunk_stride();
        let frame =
            RecordFrameCoordinate::new(artifact, offset, length as u32).ok_or_else(damaged)?;
        let scope_range = PhysicalByteRange::new(offset, length as u64).map_err(|_| damaged())?;
        let coordinate = ExtentChunkCoordinate::new(
            manifest.record(),
            manifest.extent_cell(),
            manifest.logical_bytes(),
            copy.completed,
            copy.next_ordinal,
        )
        .ok_or_else(damaged)?;
        let membership = copy.verified_membership.ok_or_else(damaged)?;
        let intent_lsn = copy.durable.as_ref().ok_or_else(damaged)?.interval().2;
        let scope = PhysicalArtifactScope::extent_chunk(
            self.durability.store_identity(),
            self.format.declaration(),
            coordinate,
            scope_range,
            range,
        );
        let (observed, failure) = self.read_copy_frame_fresh(frame)?;
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(&observed);
        let validated = match validate_extent_arena_frame(
            input,
            ExtentArenaFrameExpectation::Chunk { scope, membership },
        )
        .0
        {
            ExtentArenaFrameIntegrityValidation::Chunk(validated) => validated,
            _ => {
                failure.consume();
                return Err(damaged());
            }
        };
        let projection = match validated.project_chunk(input, coordinate) {
            Ok(projection) => projection,
            Err(_) => {
                failure.consume();
                return Err(damaged());
            }
        };
        let payload = match observed.get(projection.payload_range()) {
            Some(payload) => payload,
            None => {
                failure.consume();
                return Err(damaged());
            }
        };
        if payload.len() != expected_payload
            || projection.page_lsn() != PhysicalPageLsn::new(intent_lsn)
        {
            failure.consume();
            return Err(damaged());
        }
        copy.digest.update(payload);
        copy.completed += payload.len() as u64;
        copy.next_ordinal += 1;
        Ok(())
    }

    fn read_copy_frame_fresh(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Result<
        (
            Box<[u8]>,
            crate::physical_runtime::instance::PhysicalProjectionFailureCapability,
        ),
        RecordAppendError,
    > {
        self.residency
            .read_fresh_exact(coordinate)
            .map_err(|failure| match failure.failure() {
                CanonicalRecordReadFailure::Backend(_)
                | CanonicalRecordReadFailure::Terminal(_)
                | CanonicalRecordReadFailure::SettlementMismatch => damaged(),
                _ => RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure),
            })
    }
}
