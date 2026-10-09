use worth_store::physical_runtime::PhysicalRecordId;
use worth_store_physical_format::{DecodedBlobChunkFrameV1, PersistedRecordIdentity};

pub(super) fn persisted(id: PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(id.allocation_epoch(), id.ordinal()).unwrap()
}

pub(super) fn selected_chunk<'a>(
    selected: &'a [(PhysicalRecordId, Vec<u8>)],
    identity: PersistedRecordIdentity,
) -> DecodedBlobChunkFrameV1<'a> {
    let (_, bytes) = selected
        .iter()
        .find(|(record, _)| {
            record.allocation_epoch() == identity.allocation_epoch()
                && record.ordinal() == identity.ordinal()
        })
        .expect("quarantine edge is selected");
    DecodedBlobChunkFrameV1::decode(bytes).unwrap()
}
