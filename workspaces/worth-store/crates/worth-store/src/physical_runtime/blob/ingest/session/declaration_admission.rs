use std::num::NonZeroU64;

use worth_store_physical_format::BlobSessionDeclarationV1;

use super::super::super::{read::selected_blob_identity_exists, BlobObjectId};
use super::*;

impl ServingPhysicalRuntime {
    /// Issues an opaque object ID only after an exhaustive selected-root
    /// collision check. A bounded scan exhaustion is a denial, not absence.
    pub(in crate::physical_runtime) fn issue_blob_object_id(
        &self,
        max_selected_records: NonZeroU64,
    ) -> Result<BlobObjectId, BlobIngestFailure> {
        for _ in 0..16 {
            let candidate = super::super::super::random_nonzero_identity()
                .ok_or(BlobIngestFailure::EntropyUnavailable)?;
            if !selected_blob_identity_exists(self, candidate, max_selected_records)
                .map_err(BlobIngestFailure::IdentityInspection)?
            {
                return Ok(BlobObjectId::from_selected(
                    self.store_identity(),
                    candidate,
                ));
            }
        }
        Err(BlobIngestFailure::EntropyUnavailable)
    }

    /// Commits one durable object claim before any chunk. The C5 selected
    /// declaration is authority; the mutex only closes an in-runtime race
    /// between the complete selected scan and that root publication.
    pub(in crate::physical_runtime) fn begin_blob_ingest(
        &self,
        declaration: BlobIngestDeclaration,
        placement: AdmittedRecordPlacementPolicy,
        source_window: u64,
        max_selected_records: NonZeroU64,
    ) -> Result<BlobIngestSession<'_>, BlobIngestFailure> {
        if declaration.object().store() != self.store_identity() {
            return Err(BlobIngestFailure::ForeignObject);
        }
        if source_window >= declaration.total_bytes() {
            return Err(BlobIngestFailure::Memory(
                BlobMemoryDenial::WindowCoversObject,
            ));
        }
        let admission = self.physical_allocations();
        let mut allocation = BlobIngestAllocation::admit(&admission, source_window)
            .map_err(BlobIngestFailure::Memory)?;
        allocation
            .set_live(BlobResidentComponent::SourceWindow, source_window)
            .map_err(BlobIngestFailure::Memory)?;
        let _claim = self.lock_blob_declaration();
        if selected_blob_identity_exists(self, declaration.object().bytes(), max_selected_records)
            .map_err(BlobIngestFailure::IdentityInspection)?
        {
            return Err(BlobIngestFailure::ObjectAlreadyDeclared);
        }
        let session = {
            let mut fresh = None;
            for _ in 0..16 {
                let candidate = super::super::super::random_nonzero_identity()
                    .ok_or(BlobIngestFailure::EntropyUnavailable)?;
                if !selected_blob_identity_exists(self, candidate, max_selected_records)
                    .map_err(BlobIngestFailure::IdentityInspection)?
                {
                    fresh = Some(BlobSessionId::from_selected(candidate));
                    break;
                }
            }
            fresh.ok_or(BlobIngestFailure::EntropyUnavailable)?
        };
        let (claimed_reader, mut live_claim, selected_sequence) = self
            .claimed_blob_reader(session)
            .map_err(|cause| BlobIngestFailure::Claim(cause.into()))?;
        let max_checkpoint_sequence = selected_sequence
            .checked_add(declaration.checkpoint_limit().horizon())
            .ok_or(BlobIngestFailure::CheckpointHorizonExhausted)?;
        let store = self.store_identity().bytes();
        let encoded = BlobSessionDeclarationV1::new(
            store,
            session.bytes(),
            declaration.object().bytes(),
            declaration.scope_fingerprint(),
            u32::try_from(declaration.chunk_size().bytes()).expect("admitted chunk size fits u32"),
            declaration.total_bytes(),
            source_window + super::super::super::allocation::INGEST_OVERHEAD,
            max_checkpoint_sequence,
        )
        .map_err(BlobIngestFailure::Format)?
        .encode();
        let declaration_digest = Sha256::digest(&encoded).into();
        let _pressure = BlobAppendPressure::admit(&mut allocation, encoded.len() as u64)
            .map_err(BlobIngestFailure::Memory)?;
        // Once the selected-root inspection succeeds, hold the same session
        // claim across declaration publication and the returned ingest handle.
        // A promotion denial must never follow a durable declaration effect.
        live_claim
            .promote_live()
            .map_err(|cause| BlobIngestFailure::Claim(cause.into()))?;
        let declaration_record = append_blob_record(
            self,
            placement,
            session,
            BlobRecordKind::SessionDeclared,
            0,
            declaration.deadline(),
            encoded,
        )
        .map_err(|cause| BlobIngestFailure::Append {
            session,
            kind: BlobRecordKind::SessionDeclared,
            ordinal: 0,
            cause,
        })?;
        drop(_pressure);
        let token = super::super::BlobResumeToken {
            store,
            session: session.bytes(),
            declaration_record,
            declaration_digest,
            chunk_size: declaration.chunk_size().bytes() as u32,
            total_bytes: declaration.total_bytes(),
            max_checkpoint_sequence,
        };
        Ok(BlobIngestSession {
            runtime: self,
            placement,
            pending: Vec::with_capacity(declaration.chunk_size().bytes() as usize),
            received: 0,
            chunk_ordinal: 0,
            logical: Sha256::new(),
            tree: BlobTreeBuilder::new(store, session.bytes()),
            declaration: BlobIngestContent::from_request(&declaration),
            session,
            _claimed_reader: claimed_reader,
            _live_claim: live_claim,
            allocation,
            progress: BlobIngestProgress::new(token),
            retained_nodes: RetainedBlobNodes::empty(),
            resume_observation: None,
            poisoned: false,
            #[cfg(feature = "certification-test-authority")]
            forced_dedupe_digest: None,
            #[cfg(feature = "certification-test-authority")]
            fail_before_quarantine: false,
        })
    }
}
