use super::super::{selected_segment_rewrite::damaged, RecordPublicationDirector};
use super::{
    session::{CopyPhase, CopyProducer, ExtentCopySession},
    ExtentCopySynchronization,
};
use crate::physical_runtime::record_serving::{
    RecordAppendDenial, RecordAppendError, RecordPublicationStage,
};
use crate::physical_runtime::{PhysicalPublicationEffect, PhysicalWorkSettlementEvidence};
use sha2::Digest;
use worth_store_physical_format::{
    encode_data_frame_page_lsn, prepare_extent_chunk, DurableFrameKind, ExtentArenaFrameLayout,
    ExtentChunkCoordinate, PhysicalPageLsn, RecordArtifactFile, RecordFrameCoordinate,
};

impl RecordPublicationDirector {
    pub(super) fn copy_frame_quantum(
        &self,
        copy: &mut ExtentCopySession,
    ) -> Result<(), RecordAppendError> {
        let intent = copy.intent.ok_or_else(damaged)?;
        let format = self.format.declaration();
        let range = intent.destination().arena_range();
        let layout = ExtentArenaFrameLayout::new(format, intent.alignment()).ok_or_else(damaged)?;
        if copy.pending.is_none() {
            let Some(payload) = copy
                .cursor
                .next_chunk(&copy.allocation)
                .map_err(super::source::source_failure)?
            else {
                let digest: [u8; 32] = copy.digest.clone().finalize().into();
                if digest != intent.source_digest()
                    || copy.completed != intent.source().payload_bytes()
                    || copy.next_ordinal != intent.chunk_count() + 1
                {
                    return Err(damaged());
                }
                copy.phase = CopyPhase::Manifest;
                return Ok(());
            };
            let chunk = ExtentChunkCoordinate::new(
                intent.destination().record(),
                intent.destination().extent_cell(),
                intent.destination().payload_bytes(),
                copy.completed,
                copy.next_ordinal,
            )
            .ok_or_else(damaged)?;
            let mut frame =
                prepare_extent_chunk(format, chunk, payload.len()).map_err(|_| damaged())?;
            frame.payload_mut().copy_from_slice(payload);
            copy.digest.update(payload);
            copy.completed += payload.len() as u64;
            let mut bytes = frame.seal();
            encode_data_frame_page_lsn(
                &mut bytes,
                DurableFrameKind::Extent,
                PhysicalPageLsn::new(copy.durable.as_ref().ok_or_else(damaged)?.interval().2),
            )
            .map_err(|_| damaged())?;
            let offset = range.offset()
                + layout.manifest_stride()
                + u64::from(copy.next_ordinal - 1) * layout.chunk_stride();
            let coordinate = RecordFrameCoordinate::new(
                RecordArtifactFile::ExtentArena {
                    arena: range.arena().get(),
                },
                offset,
                bytes.len() as u32,
            )
            .ok_or_else(damaged)?;
            copy.pending = Some((coordinate, bytes));
        }
        self.write_copy_pending(copy)?;
        copy.next_ordinal += 1;
        Ok(())
    }

    pub(super) fn copy_manifest_quantum(
        &self,
        copy: &mut ExtentCopySession,
    ) -> Result<(), RecordAppendError> {
        if copy.pending.is_none() {
            let intent = copy.intent.ok_or_else(damaged)?;
            let bytes = intent
                .destination_manifest(self.format.declaration())
                .ok_or_else(damaged)?
                .encode(self.format.declaration());
            let range = intent.destination().arena_range();
            let coordinate = RecordFrameCoordinate::new(
                RecordArtifactFile::ExtentArena {
                    arena: range.arena().get(),
                },
                range.offset(),
                bytes.len() as u32,
            )
            .ok_or_else(damaged)?;
            copy.pending = Some((coordinate, bytes));
        }
        self.write_copy_pending(copy)?;
        copy.phase = CopyPhase::Synchronizing;
        Ok(())
    }

    fn write_copy_pending(&self, copy: &mut ExtentCopySession) -> Result<(), RecordAppendError> {
        let (coordinate, bytes) = copy.pending.as_ref().ok_or_else(damaged)?;
        let prepared = match copy.producer {
            CopyProducer::ArenaEvacuation => {
                let admission = self
                    .mutation
                    .admit_compaction_frame(bytes.len() as u64)
                    .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure))?;
                self.mutation.prepare_compaction_artifact(
                    RecordPublicationStage::CandidateDataWrite,
                    *coordinate,
                    bytes,
                    admission,
                )
            }
            CopyProducer::BlobMovement => {
                let admission = self
                    .mutation
                    .admit_blob_movement_frame(bytes.len() as u64)
                    .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure))?;
                self.mutation.prepare_blob_movement_artifact(
                    RecordPublicationStage::CandidateDataWrite,
                    *coordinate,
                    bytes,
                    admission,
                )
            }
        }
        .map_err(|failure| copy_work_failure(copy.producer, failure))?;
        let completed = prepared
            .execute()
            .map_err(|failure| copy_work_failure(copy.producer, failure))?;
        copy.writes
            .as_mut()
            .ok_or_else(damaged)?
            .observe(
                self.durability.store_identity(),
                *coordinate,
                bytes,
                completed.into_physical(),
            )
            .map_err(|_| damaged())?;
        copy.pending = None;
        Ok(())
    }

    pub(super) fn synchronize_extent_copy(
        &self,
        copy: &mut ExtentCopySession,
    ) -> Result<(), RecordAppendError> {
        let intent = copy.intent.ok_or_else(damaged)?;
        let artifact = RecordArtifactFile::ExtentArena {
            arena: intent.destination().arena_range().arena().get(),
        };
        let (file, file_work) =
            self.copy_sync_effect(artifact, PhysicalPublicationEffect::SynchronizeArtifact)?;
        let (parent, parent_work) = self.copy_sync_effect(
            artifact,
            PhysicalPublicationEffect::SynchronizeArtifactParent,
        )?;
        copy.synchronization = Some(
            ExtentCopySynchronization::new(intent, file, parent, file_work, parent_work)
                .map_err(|_| damaged())?,
        );
        copy.digest = sha2::Sha256::new();
        copy.completed = 0;
        copy.next_ordinal = 0;
        copy.phase = CopyPhase::Verifying;
        Ok(())
    }

    fn copy_sync_effect(
        &self,
        artifact: RecordArtifactFile,
        effect: PhysicalPublicationEffect,
    ) -> Result<
        (
            crate::physical_runtime::CompletedPhysicalPublicationEffect,
            crate::physical_runtime::PhysicalWorkIdentity,
        ),
        RecordAppendError,
    > {
        let settled = self
            .root_work
            .execute_record_effect(artifact, effect, 1, None)
            .map_err(|_| damaged())?;
        let identity = settled.intent().identity();
        if settled
            .effect_identity()
            .is_none_or(|effect| effect.work() != identity)
        {
            return Err(damaged());
        }
        match settled.into_evidence() {
            PhysicalWorkSettlementEvidence::PublicationEffect {
                physical,
                scheduler: worth_store_io_scheduler::QueueExecutionOutcome::Executed(_),
            } => Ok((physical, identity)),
            _ => Err(damaged()),
        }
    }
}

fn copy_work_failure(
    producer: CopyProducer,
    failure: crate::physical_runtime::record_serving::CanonicalRecordMutationFailure,
) -> RecordAppendError {
    match producer {
        CopyProducer::BlobMovement => RecordAppendError::Denied(
            RecordAppendDenial::PhysicalWorkUnavailable(Box::new(failure.evidence())),
        ),
        CopyProducer::ArenaEvacuation => damaged(),
    }
}
