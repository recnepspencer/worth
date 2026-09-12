use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, FilesystemMediaAdmission, ManifestEntryCapacity,
    PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration, PhysicalRecordInitialization,
    PhysicalRecordPlacementPolicy, PhysicalRuntimeAdmission, PhysicalStore, RecordCountLimit,
    RecordScanRequest, SegmentPageCount,
};
use worth_store_physical_backend::FilesystemAccessPosture;

#[path = "support/durability.rs"]
mod durability;

#[allow(dead_code)]
fn capabilities_typecheck() {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .segment_pages(SegmentPageCount::new(4).unwrap())
        .manifest_capacity(ManifestEntryCapacity::new(4).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let owner = PhysicalStore::admit(
        PhysicalRuntimeAdmission::new(
            std::env::temp_dir().join("worth-store-independent-scan-and-mutation"),
        )
        .unwrap(),
    )
    .unwrap();
    let media = match owner
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => unreachable!(),
    };
    let durability = durability::admitted(&media);
    let runtime = match media
        .initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(runtime) => runtime,
        _ => unreachable!(),
    };
    let mut scan = runtime
        .records()
        .expect("read protection admission")
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let _writer = runtime.record_submission();
    let mut scratch = [0_u8; 16];
    let _ = scan.read_next_into(&mut scratch);
}

fn main() {}
