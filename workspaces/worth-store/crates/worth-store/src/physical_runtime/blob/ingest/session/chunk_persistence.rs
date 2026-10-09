use worth_store_physical_format::{
    BlobChunkFrameV1, BlobChunkOccurrenceV1, BlobChunkReuseClaimV1, BlobChunkReuseClaimV2,
    BlobDedupeQuarantineV1, BlobRecordKind, DecodedBlobChunkFrameV1, PersistedRecordIdentity,
};

use crate::physical_runtime::blob::{
    append::{append_blob_dedupe_quarantine, append_blob_record, append_blob_reuse_claim},
    dedupe::lookup_reusable_chunk,
    ingest::append_pressure::BlobAppendPressure,
    BlobDedupeFailure, VerifiedDedupeSource,
};

use super::{BlobIngestFailure, BlobIngestSession};

impl BlobIngestSession<'_> {
    pub(super) fn append_selected_chunk(
        &mut self,
        ordinal: u64,
    ) -> Result<(PersistedRecordIdentity, [u8; 32]), BlobIngestFailure> {
        let chunk_size = u32::try_from(self.declaration.chunk_size().bytes())
            .expect("admitted chunk size fits u32");
        #[cfg(feature = "certification-test-authority")]
        let forced_digest = self.forced_dedupe_digest.take();
        #[cfg(not(feature = "certification-test-authority"))]
        let forced_digest = None;
        let reusable = lookup_reusable_chunk(
            self.runtime,
            &mut self.allocation,
            self.declaration.scope_fingerprint(),
            chunk_size,
            &self.pending,
            forced_digest,
        );
        match reusable {
            Ok(Some(source)) => self.append_reuse_claim(ordinal, chunk_size, source),
            Ok(None) => self.append_original_chunk(ordinal, chunk_size),
            Err(BlobDedupeFailure::DigestCollisionDenied {
                scope,
                digest,
                source_publication,
                source_ordinal,
                source_chunk,
            }) => {
                self.append_dedupe_quarantine(
                    ordinal,
                    chunk_size,
                    scope,
                    digest,
                    source_publication,
                    source_ordinal,
                    source_chunk,
                )?;
                Err(BlobIngestFailure::Dedupe(
                    BlobDedupeFailure::DigestCollisionDenied {
                        scope,
                        digest,
                        source_publication,
                        source_ordinal,
                        source_chunk,
                    },
                ))
            }
            Err(error) => Err(BlobIngestFailure::Dedupe(error)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn append_dedupe_quarantine(
        &mut self,
        ordinal: u64,
        chunk_size: u32,
        scope: [u8; 32],
        digest: [u8; 32],
        source_publication: PersistedRecordIdentity,
        source_ordinal: u64,
        source_chunk: PersistedRecordIdentity,
    ) -> Result<(), BlobIngestFailure> {
        // The conflicting bytes become a selected original chunk before the
        // classified quarantine append advances the C.5 root marker. If the
        // second append fails, no successful quarantine is reported.
        let (conflicting_chunk, _) = self.append_original_chunk(ordinal, chunk_size)?;
        #[cfg(feature = "certification-test-authority")]
        if std::mem::take(&mut self.fail_before_quarantine) {
            return Err(BlobIngestFailure::CertificationQuarantineGap);
        }
        let quarantine = BlobDedupeQuarantineV1::new(
            self.runtime.store_identity().bytes(),
            scope,
            digest,
            chunk_size,
            source_publication,
            source_ordinal,
            source_chunk,
            self.session.bytes(),
            ordinal,
            conflicting_chunk,
        )
        .map_err(BlobIngestFailure::Format)?;
        let encoded = quarantine.encode();
        let token = self.resume_token();
        let _pressure = BlobAppendPressure::admit(&mut self.allocation, encoded.len() as u64)
            .map_err(BlobIngestFailure::Memory)?;
        append_blob_dedupe_quarantine(
            self.runtime,
            self.placement,
            self.session,
            ordinal,
            self.declaration.deadline(),
            encoded,
            token.declaration_record,
            token.declaration_digest,
        )
        .map_err(|cause| BlobIngestFailure::Append {
            session: self.session,
            kind: BlobRecordKind::DedupeQuarantine,
            ordinal,
            cause,
        })?;
        Ok(())
    }

    fn append_reuse_claim(
        &mut self,
        ordinal: u64,
        chunk_size: u32,
        source: VerifiedDedupeSource,
    ) -> Result<(PersistedRecordIdentity, [u8; 32]), BlobIngestFailure> {
        let claim = BlobChunkReuseClaimV1::new(
            self.runtime.store_identity().bytes(),
            self.session.bytes(),
            ordinal,
            self.declaration.scope_fingerprint(),
            chunk_size,
            source.chunk_length,
            source.stored_digest,
            source.chunk,
            source.publication,
            source.source_ordinal,
        )
        .map_err(BlobIngestFailure::Format)?;
        let encoded = BlobChunkReuseClaimV2::new(
            claim,
            source.source_publication,
            source.source_publication_frame_sha256,
        )
        .map_err(BlobIngestFailure::Format)?
        .encode();
        let token = self.resume_token();
        let _pressure = BlobAppendPressure::admit(&mut self.allocation, encoded.len() as u64)
            .map_err(BlobIngestFailure::Memory)?;
        let claim_record = append_blob_reuse_claim(
            self.runtime,
            self.placement,
            self.session,
            ordinal,
            self.declaration.deadline(),
            encoded,
            token.declaration_record,
            token.declaration_digest,
        )
        .map_err(|cause| BlobIngestFailure::Append {
            session: self.session,
            kind: BlobRecordKind::ChunkReuseClaimV2,
            ordinal,
            cause,
        })?;
        // The tree points to the selected claim. Readers follow its authenticated
        // source edge directly, without scanning the selected C.5 record set.
        Ok((claim_record, source.stored_digest))
    }

    fn append_original_chunk(
        &mut self,
        ordinal: u64,
        chunk_size: u32,
    ) -> Result<(PersistedRecordIdentity, [u8; 32]), BlobIngestFailure> {
        let occurrence = BlobChunkOccurrenceV1::new(
            self.runtime.store_identity().bytes(),
            self.session.bytes(),
            ordinal,
        )
        .map_err(BlobIngestFailure::Format)?;
        let encoded = BlobChunkFrameV1::encode(occurrence, chunk_size, &self.pending)
            .map_err(BlobIngestFailure::Format)?;
        let digest = DecodedBlobChunkFrameV1::decode(&encoded)
            .map_err(BlobIngestFailure::Format)?
            .stored_digest();
        let _pressure = BlobAppendPressure::admit(&mut self.allocation, encoded.len() as u64)
            .map_err(BlobIngestFailure::Memory)?;
        let record = append_blob_record(
            self.runtime,
            self.placement,
            self.session,
            BlobRecordKind::Chunk,
            ordinal,
            self.declaration.deadline(),
            encoded,
        )
        .map_err(|cause| BlobIngestFailure::Append {
            session: self.session,
            kind: BlobRecordKind::Chunk,
            ordinal,
            cause,
        })?;
        Ok((record, digest))
    }
}
