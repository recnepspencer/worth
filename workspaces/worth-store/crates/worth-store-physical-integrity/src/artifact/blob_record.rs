//! Descriptive C.9 validation of an already obtained C.5 record payload.
//!
//! This checks the inner C.11 frame and expected Store scope only. A caller
//! must independently establish selected extent routing and C.5 integrity;
//! a byte slice cannot certify either custody fact.

mod selected;

pub use selected::{validate_blob_record, BlobRecordIntegrityValidation};

use worth_store_physical_format::{
    decode_blob_record, BlobRecordDenial, BlobRecordKind, BlobRecordV1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobRecordPayloadValidationDenial {
    Format(BlobRecordDenial),
    WrongFamily,
    StoreScopeMismatch,
}

/// Validate one inner blob payload after its selected extent-backed C.5
/// record has been established by the physical owner.
pub fn validate_blob_record_payload_only(
    expected_store: [u8; 16],
    expected_kind: BlobRecordKind,
    payload: &[u8],
) -> Result<BlobRecordV1<'_>, BlobRecordPayloadValidationDenial> {
    let record = decode_blob_record(payload).map_err(BlobRecordPayloadValidationDenial::Format)?;
    if record.kind() != expected_kind {
        return Err(BlobRecordPayloadValidationDenial::WrongFamily);
    }
    let observed_store = match &record {
        BlobRecordV1::SessionDeclared(value) => value.store(),
        BlobRecordV1::Chunk(value) => value.occurrence().store(),
        BlobRecordV1::TreeNode(value) => value.occurrence().store(),
        BlobRecordV1::GenerationPublished(value) => value.store(),
        BlobRecordV1::SessionFrontier(value) => value.store(),
        BlobRecordV1::SessionAbandoned(value) => value.store(),
        BlobRecordV1::DropSetManifest(value) => value.store(),
        BlobRecordV1::DropSetManifestV2(value) => value.store(),
        BlobRecordV1::DropSetManifestV3(value) => value.store(),
        BlobRecordV1::OriginalDropReserved(value) => value.store(),
        BlobRecordV1::ReclaimDescriptor(value) => value.store(),
        BlobRecordV1::ReclaimDescriptorV2(value) => value.store(),
        BlobRecordV1::ReclaimDescriptorV3(value) => value.base().store(),
        BlobRecordV1::ChunkReuseClaim(value) => value.store(),
        BlobRecordV1::ChunkReuseClaimV2(value) => value.claim().store(),
        BlobRecordV1::DedupeQuarantine(value) => value.store(),
    };
    if expected_store != observed_store {
        return Err(BlobRecordPayloadValidationDenial::StoreScopeMismatch);
    }
    Ok(record)
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::{
        BlobAbandonmentReasonV1, BlobChunkFrameV1, BlobChunkOccurrenceV1, BlobChunkReuseClaimV1,
        BlobSessionAbandonedV1, PersistedRecordIdentity,
    };

    use super::*;

    #[test]
    fn validates_inner_claim_but_does_not_pretend_to_prove_route() {
        let payload = BlobChunkFrameV1::encode(
            BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).unwrap(),
            64 << 10,
            &[3; 64 << 10],
        )
        .unwrap();
        assert_eq!(
            validate_blob_record_payload_only([1; 16], BlobRecordKind::Chunk, &payload)
                .expect("valid payload")
                .kind(),
            BlobRecordKind::Chunk,
        );
        assert_eq!(
            validate_blob_record_payload_only([4; 16], BlobRecordKind::Chunk, &payload),
            Err(BlobRecordPayloadValidationDenial::StoreScopeMismatch),
        );
        assert_eq!(
            validate_blob_record_payload_only([1; 16], BlobRecordKind::TreeNode, &payload),
            Err(BlobRecordPayloadValidationDenial::WrongFamily),
        );
    }

    #[test]
    fn validates_explicit_terminal_payload_without_claiming_session_fate() {
        let abandoned = BlobSessionAbandonedV1::new(
            [1; 16],
            [2; 16],
            PersistedRecordIdentity::new([3; 16], 4).unwrap(),
            [5; 32],
            BlobAbandonmentReasonV1::ExplicitAbort,
        )
        .unwrap();
        let frame = abandoned.encode();
        assert_eq!(
            validate_blob_record_payload_only([1; 16], BlobRecordKind::SessionAbandoned, &frame,),
            Ok(BlobRecordV1::SessionAbandoned(abandoned))
        );
        assert_eq!(
            validate_blob_record_payload_only([9; 16], BlobRecordKind::SessionAbandoned, &frame,),
            Err(BlobRecordPayloadValidationDenial::StoreScopeMismatch)
        );
    }

    #[test]
    fn validates_reuse_claim_shape_without_promoting_source_linkage() {
        let claim = BlobChunkReuseClaimV1::new(
            [1; 16],
            [2; 16],
            3,
            [4; 32],
            64 << 10,
            64 << 10,
            [5; 32],
            PersistedRecordIdentity::new([6; 16], 7).unwrap(),
            PersistedRecordIdentity::new([8; 16], 9).unwrap(),
            10,
        )
        .unwrap();
        assert_eq!(
            validate_blob_record_payload_only(
                [1; 16],
                BlobRecordKind::ChunkReuseClaim,
                &claim.encode()
            ),
            Ok(BlobRecordV1::ChunkReuseClaim(claim)),
        );
        assert_eq!(
            validate_blob_record_payload_only(
                [11; 16],
                BlobRecordKind::ChunkReuseClaim,
                &claim.encode()
            ),
            Err(BlobRecordPayloadValidationDenial::StoreScopeMismatch),
        );
    }
}
