use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy,
    ManifestEntryCapacity, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordPlacementPolicy, RecordByteLimit,
};
use worth_store_physical_format::{DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES};

pub(super) fn record_configuration() -> (
    AdmittedPhysicalRecordFormat,
    AdmittedRecordPlacementPolicy,
    AdmittedRecordAccessPolicy,
) {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .admit()
            .expect("canonical v1 format declaration"),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).expect("nonzero capacity"))
        .extent_threshold(RecordByteLimit::new(8_192).expect("nonzero threshold"))
        .admit(format)
        .expect("ordinary writer placement is compatible");
    let access = PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .expect("ordinary writer access is compatible");
    (format, placement, access)
}

pub(super) fn capacity_transition_placement(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedRecordPlacementPolicy {
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(4).expect("nonzero transition capacity"))
        .extent_threshold(RecordByteLimit::new(8_192).expect("nonzero threshold"))
        .admit(format)
        .expect("capacity-transition placement is compatible")
}

pub(super) fn dirty_checkpoint_payload_length(format: AdmittedPhysicalRecordFormat) -> usize {
    let frame_bytes = usize::try_from(format.declaration().page_size().bytes())
        .expect("admitted page size fits usize");
    let frame_overhead = DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES;
    let chunk_payload = frame_bytes
        .checked_sub(frame_overhead)
        .expect("admitted page size contains extent framing");
    chunk_payload
        .checked_mul(2)
        .and_then(|length| length.checked_add(1))
        .expect("dirty checkpoint payload length fits usize")
}
