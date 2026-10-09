use worth_store_physical_format::{
    BlobDedupeQuarantineV1, DecodedBlobChunkFrameV1, BLOB_CHUNK_FRAME_MAX_BYTES,
};

use crate::physical_runtime::PhysicalRecordReader;

use super::{
    key::DedupeIndexValue,
    source::{read_selected, verify_source},
    BlobDedupeFailure,
};

/// Verifies both selected chunk coordinates and the unequal-byte witness.
/// The claim is a durable suppression fact only after C.5 selects it.
pub(in crate::physical_runtime) fn verify_selected_quarantine(
    reader: &PhysicalRecordReader,
    claim: BlobDedupeQuarantineV1,
) -> Result<u32, BlobDedupeFailure> {
    let (frame, _) = read_selected(
        reader,
        claim.conflicting_chunk(),
        BLOB_CHUNK_FRAME_MAX_BYTES,
    )?;
    let conflicting =
        DecodedBlobChunkFrameV1::decode(&frame).map_err(BlobDedupeFailure::SourceFormat)?;
    if claim.store() != reader.store_identity().bytes()
        || conflicting.occurrence().store() != claim.store()
        || conflicting.occurrence().session() != claim.destination_session()
        || conflicting.occurrence().ordinal() != claim.destination_ordinal()
        || conflicting.chunk_size() != claim.chunk_size()
    {
        return Err(BlobDedupeFailure::SourceTreeDamaged);
    }
    let locator = DedupeIndexValue::new(
        claim.source_publication(),
        claim.source_ordinal(),
        claim.source_chunk(),
    );
    match verify_source(
        reader,
        locator,
        claim.scope(),
        claim.disputed_digest(),
        conflicting.bytes(),
        claim.chunk_size(),
    ) {
        Err(BlobDedupeFailure::DigestCollisionDenied {
            scope,
            digest,
            source_publication,
            source_ordinal,
            source_chunk,
        }) if scope == claim.scope()
            && digest == claim.disputed_digest()
            && source_publication == claim.source_publication()
            && source_ordinal == claim.source_ordinal()
            && source_chunk == claim.source_chunk() =>
        {
            Ok(u32::try_from(conflicting.bytes().len()).expect("admitted chunk fits u32"))
        }
        Ok(_) => Err(BlobDedupeFailure::SourceTreeDamaged),
        Err(error) => Err(error),
    }
}
