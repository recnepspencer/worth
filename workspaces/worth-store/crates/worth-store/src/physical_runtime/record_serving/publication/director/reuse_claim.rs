use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobChunkReuseClaimV2, BlobDedupeQuarantineV1, BlobSessionDeclarationV1,
    BLOB_CHUNK_FRAME_MAX_BYTES, BLOB_RECORD_HEADER_BYTES,
};

use crate::physical_runtime::record_serving::{
    publication::{batch::RecordAppendInput, PreparedReuseDeclarationBasis},
    RecordAppendBatch, RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    blob::{verify_selected_quarantine, verify_source_for_new_reuse_claim},
    PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordReadLimits,
};

use super::RecordPublicationDirector;

impl RecordPublicationDirector {
    /// The final data planner checks the selected original source under a
    /// protected root, then fences that exact generation through cutover.
    /// The earlier derived-index hit is never a source-authority grant.
    pub(super) fn verify_new_reuse_claim(
        &self,
        batch: &RecordAppendBatch,
        expected_root_generation: u64,
        declaration_basis: PreparedReuseDeclarationBasis,
    ) -> Result<(), RecordAppendError> {
        let [RecordAppendInput::Bytes(bytes)] = batch.records.as_slice() else {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::ReuseSourceInvalid,
            ));
        };
        let claim = BlobChunkReuseClaimV2::decode(bytes)
            .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::ReuseSourceInvalid))?;
        let base = claim.claim();
        self.with_selected_claim_reader(expected_root_generation, |reader| {
            let expected_length = verify_selected_destination(
                reader,
                declaration_basis,
                base.store(),
                base.destination_session(),
                base.scope(),
                base.chunk_size(),
                base.destination_ordinal(),
            )?;
            if expected_length != base.chunk_length() {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::ReuseDestinationInvalid,
                ));
            }
            let mut scratch = Vec::new();
            scratch
                .try_reserve_exact(BLOB_CHUNK_FRAME_MAX_BYTES)
                .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::ReuseSourceInvalid))?;
            scratch.resize(BLOB_CHUNK_FRAME_MAX_BYTES, 0);
            verify_source_for_new_reuse_claim(reader, claim, &mut scratch)
                .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::ReuseSourceInvalid))?;
            Ok(())
        })
    }

    pub(super) fn verify_new_quarantine_claim(
        &self,
        batch: &RecordAppendBatch,
        expected_root_generation: u64,
        declaration_basis: PreparedReuseDeclarationBasis,
    ) -> Result<(), RecordAppendError> {
        let [RecordAppendInput::Bytes(bytes)] = batch.records.as_slice() else {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::ReuseSourceInvalid,
            ));
        };
        let claim = BlobDedupeQuarantineV1::decode(bytes)
            .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::ReuseSourceInvalid))?;
        self.with_selected_claim_reader(expected_root_generation, |reader| {
            let expected_length = verify_selected_destination(
                reader,
                declaration_basis,
                claim.store(),
                claim.destination_session(),
                claim.scope(),
                claim.chunk_size(),
                claim.destination_ordinal(),
            )?;
            let observed_length = verify_selected_quarantine(reader, claim)
                .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::ReuseSourceInvalid))?;
            if observed_length != expected_length {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::ReuseDestinationInvalid,
                ));
            }
            Ok(())
        })
    }

    fn with_selected_claim_reader(
        &self,
        expected_root_generation: u64,
        verify: impl FnOnce(&PhysicalRecordReader) -> Result<(), RecordAppendError>,
    ) -> Result<(), RecordAppendError> {
        let (root, protection) = self
            .capture_read_root()
            .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::ReuseSourceChanged))?;
        if root.generation() != expected_root_generation {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::ReuseSourceChanged,
            ));
        }
        let runtime = self.runtime.upgrade().ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublicationAuthorityReleased,
        ))?;
        let _allocation = self
            .residency
            .begin_foreground_write_operation(
                NonZeroU64::new(2 * BLOB_CHUNK_FRAME_MAX_BYTES as u64)
                    .expect("fixed source verification charge is nonzero"),
            )
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let reader = PhysicalRecordReader {
            execution: crate::physical_runtime::instance::PhysicalStoreWorkRuntime::execution(
                &runtime,
                self.generation,
            ),
            protection,
            store: self.durability.store_identity(),
            format: self.format,
            access: self.access,
            current_root: root,
            generation: self.generation,
            runtime: std::sync::Arc::downgrade(&runtime),
            lifecycle: self.reader_factory.reader(),
            residency: self.residency.clone().for_rebuild(),
        };
        verify(&reader)
    }
}

const DECLARATION_FRAME_BYTES: usize = BLOB_RECORD_HEADER_BYTES + 108;

fn verify_selected_destination(
    reader: &PhysicalRecordReader,
    basis: PreparedReuseDeclarationBasis,
    store: [u8; 16],
    session: [u8; 16],
    scope: [u8; 32],
    chunk_size: u32,
    ordinal: u64,
) -> Result<u32, RecordAppendError> {
    let denial = || RecordAppendError::Denied(RecordAppendDenial::ReuseDestinationInvalid);
    let limit = RecordByteLimit::new(DECLARATION_FRAME_BYTES as u32)
        .expect("fixed declaration frame is nonzero");
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(basis.record),
            RecordReadLimits::new(limit),
        )
        .map_err(|_| denial())?;
    let mut bytes = [0_u8; DECLARATION_FRAME_BYTES];
    let mut used = 0;
    while used < bytes.len() {
        let count = stream.read_next(&mut bytes[used..]).map_err(|_| denial())?;
        if count == 0 {
            return Err(denial());
        }
        used += count;
    }
    let mut excess = [0_u8; 1];
    if stream.read_next(&mut excess).map_err(|_| denial())? != 0 {
        return Err(denial());
    }
    let declaration = BlobSessionDeclarationV1::decode(&bytes).map_err(|_| denial())?;
    let frame_digest: [u8; 32] = Sha256::digest(bytes).into();
    if frame_digest != basis.digest
        || declaration.store() != reader.store_identity().bytes()
        || store != declaration.store()
        || session != declaration.session()
        || scope != declaration.key_scope()
        || chunk_size != declaration.chunk_size()
    {
        return Err(denial());
    }
    expected_chunk_length(
        declaration.declared_bytes(),
        declaration.chunk_size(),
        ordinal,
    )
    .ok_or_else(denial)
}

fn expected_chunk_length(total: u64, chunk_size: u32, ordinal: u64) -> Option<u32> {
    let start = ordinal.checked_mul(u64::from(chunk_size))?;
    let remaining = total.checked_sub(start)?;
    if remaining == 0 {
        return None;
    }
    u32::try_from(remaining.min(u64::from(chunk_size))).ok()
}

#[cfg(test)]
mod tests {
    use super::expected_chunk_length;

    #[test]
    fn destination_chunk_boundaries_reject_out_of_range_and_overflow() {
        assert_eq!(expected_chunk_length(131_073, 65_536, 0), Some(65_536));
        assert_eq!(expected_chunk_length(131_073, 65_536, 1), Some(65_536));
        assert_eq!(expected_chunk_length(131_073, 65_536, 2), Some(1));
        assert_eq!(expected_chunk_length(131_073, 65_536, 3), None);
        assert_eq!(expected_chunk_length(131_073, 65_536, u64::MAX), None);
    }
}
