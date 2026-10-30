pub use {
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PageFillPercent,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::SegmentPageCount,
};

pub use crate::physical_record_journeys::configuration;

pub fn dense_configuration(
    segment_pages: u32,
) -> (
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::AdmittedRecordAccessPolicy,
) {
    let (format, _, access) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(segment_pages).unwrap())
        .extent_threshold(worth_store::physical_runtime::RecordByteLimit::new(8_000).unwrap())
        .page_fill(worth_store::physical_runtime::PageFillPercent::new(50).unwrap())
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(128).unwrap())
        .admit(format)
        .unwrap();
    (format, placement, access)
}

pub fn courtroom_configuration() -> (
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::AdmittedRecordAccessPolicy,
) {
    let (format, _, _) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(32).unwrap())
        .extent_threshold(worth_store::physical_runtime::RecordByteLimit::new(8_192).unwrap())
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(16).unwrap())
        .admit(format)
        .unwrap();
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .transfer_limit(worth_store::physical_runtime::RecordByteLimit::new(65_536).unwrap())
        .scratch_limit(worth_store::physical_runtime::RecordByteLimit::new(131_072).unwrap())
        .scan_record_limit(worth_store::physical_runtime::RecordCountLimit::new(17).unwrap())
        .admit(format)
        .unwrap();
    (format, placement, access)
}
