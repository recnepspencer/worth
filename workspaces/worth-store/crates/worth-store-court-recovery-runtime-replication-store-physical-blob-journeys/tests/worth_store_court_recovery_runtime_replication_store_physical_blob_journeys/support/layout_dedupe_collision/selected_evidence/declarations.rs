pub(crate) use {worth_store::physical_runtime::PhysicalRecordId};
pub(crate) use {worth_store_physical_format::DecodedBlobChunkFrameV1, worth_store_physical_format::PersistedRecordIdentity};

pub(crate) fn persisted(id: PhysicalRecordId) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(id.allocation_epoch(), id.ordinal()).unwrap()
}

pub(crate) fn selected_chunk<'a>(
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
    worth_store_physical_format::DecodedBlobChunkFrameV1::decode(bytes).unwrap()
}