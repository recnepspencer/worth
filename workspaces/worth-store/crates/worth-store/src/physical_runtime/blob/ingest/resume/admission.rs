use worth_store_physical_format::BlobTreeNodeV1;

use super::super::super::{
    AdmittedBlobScope, BlobIngestAllocation, BlobMemoryDenial, BlobResidentComponent, BlobSessionId,
};
use super::super::{
    append_pressure::BlobAppendPressure, content::BlobIngestContent, node_writer::BlobNodeWriter,
    BlobIngestFailure, BlobIngestSession,
};
use super::{
    rehash, selection, tree_reconstruction, BlobResumeFailure, BlobResumeLimits, BlobResumeToken,
    RetainedBlobNodes,
};
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, ServingPhysicalRuntime,
};

impl ServingPhysicalRuntime {
    pub(in crate::physical_runtime) fn resume_blob_ingest(
        &self,
        token: BlobResumeToken,
        scope: &AdmittedBlobScope,
        placement: AdmittedRecordPlacementPolicy,
        source_window: u64,
        deadline: PhysicalMutationDeadline,
        limits: BlobResumeLimits,
    ) -> Result<BlobIngestSession<'_>, BlobResumeFailure> {
        if token.store != self.store_identity().bytes() {
            return Err(BlobResumeFailure::ForeignStore);
        }
        if source_window >= token.total_bytes {
            return Err(memory_failure(BlobMemoryDenial::WindowCoversObject));
        }
        let admission = self.physical_allocations();
        let mut allocation =
            BlobIngestAllocation::admit(&admission, source_window).map_err(memory_failure)?;
        allocation
            .set_live(BlobResidentComponent::SourceWindow, source_window)
            .map_err(memory_failure)?;
        let session = BlobSessionId::from_selected(token.session);
        let (reader, mut live_claim, completed_checkpoint) = self
            .claimed_blob_reader_with_checkpoint(session)
            .map_err(|cause| BlobResumeFailure::Claim(cause.into()))?;
        let selected_sequence = completed_checkpoint.map_or(0, |value| value.sequence().get());
        let mut selected = selection::select(
            reader,
            token,
            scope,
            limits,
            &mut allocation,
            completed_checkpoint,
        )?;
        if selected_sequence > selected.declaration.max_checkpoint_sequence() {
            return Err(BlobResumeFailure::Expired);
        }
        // Reuse-source proof keeps one additional bounded selected-record
        // frame while the destination reconstruction buffer is live.
        let proves_reuse = selected.claims.iter().any(|claim| {
            matches!(
                claim,
                super::claims::SelectedResumeClaim::ReusedChunk { .. }
            )
        });
        if proves_reuse {
            allocation
                .set_live(BlobResidentComponent::Scratch, 512 * 1024)
                .map_err(memory_failure)?;
        }
        let (logical, progress) = rehash::rehash(&mut selected, token)?;
        if proves_reuse {
            allocation
                .set_live(BlobResidentComponent::Scratch, 0)
                .map_err(memory_failure)?;
        }
        tree_reconstruction::validate_existing_nodes(&mut selected)?;

        // The protected root and admitted claim metadata survive the scratch
        // release. No reconstruction-only payload buffer overlaps new writes.
        drop(std::mem::take(&mut selected.scratch));
        allocation
            .set_live(BlobResidentComponent::ResumeScratch, 0)
            .map_err(memory_failure)?;
        if tree_reconstruction::missing_full_nodes(&selected) {
            drop(
                BlobAppendPressure::admit(
                    &mut allocation,
                    BlobTreeNodeV1::maximum_encoded_bytes() as u64,
                )
                .map_err(memory_failure)?,
            );
        }
        live_claim
            .promote_live()
            .map_err(|cause| BlobResumeFailure::Claim(cause.into()))?;
        let mut empty_retained = RetainedBlobNodes::empty();
        let mut writer = BlobNodeWriter {
            runtime: self,
            placement,
            session,
            deadline,
            allocation: &mut allocation,
            retained: &mut empty_retained,
        };
        let (tree, retained_nodes) = tree_reconstruction::restore(&mut selected, &mut writer)?;
        drop(selected.claims);
        allocation
            .set_live(BlobResidentComponent::ResumeMetadata, 0)
            .map_err(memory_failure)?;
        let mut pending = Vec::new();
        pending
            .try_reserve_exact(selected.declaration.chunk_size() as usize)
            .map_err(|_| BlobResumeFailure::ScratchUnavailable)?;
        let frontier = progress.frontier();
        Ok(BlobIngestSession {
            runtime: self,
            placement,
            declaration: BlobIngestContent::from_selected(
                selected.declaration,
                self.store_identity(),
                deadline,
            ),
            session,
            allocation,
            progress,
            _claimed_reader: selected.reader,
            _live_claim: live_claim,
            pending,
            received: frontier.bytes(),
            chunk_ordinal: frontier.next_chunk_ordinal(),
            logical,
            tree,
            retained_nodes,
            resume_observation: Some(selected.observation),
            poisoned: false,
            #[cfg(feature = "certification-test-authority")]
            forced_dedupe_digest: None,
            #[cfg(feature = "certification-test-authority")]
            fail_before_quarantine: false,
        })
    }
}

fn memory_failure(cause: BlobMemoryDenial) -> BlobResumeFailure {
    BlobResumeFailure::Ingest(BlobIngestFailure::Memory(cause))
}
