use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, FilesystemMediaAdmission,
    ManifestEntryCapacity, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordOpen, PhysicalRecordPlacementPolicy, PhysicalRuntimeAdmission, PhysicalStore,
    SegmentPageCount, ServingPhysicalRuntime,
};

use super::super::production_profile::ProductionWorldProfile;

pub(super) fn open(
    root: &Path,
    admission: FilesystemMediaAdmission,
) -> (ServingPhysicalRuntime, AdmittedRecordPlacementPolicy) {
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("PW source media admission failed"),
    };
    let durability = super::super::production_store::admit_durability(&media).unwrap();
    let profile = ProductionWorldProfile::Primary16KiB;
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(profile.page_size())
            .admit()
            .unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(16).unwrap())
        .segment_pages(SegmentPageCount::new(4).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let request = PhysicalRecordOpen::new(format, access, durability)
        .with_residency_policy(profile.residency(format));
    let serving = match media.open_record_store(request).into_raw() {
        TransitionOutcome::Success(serving) => serving,
        TransitionOutcome::Denied(denial) => {
            panic!("PW source Store open denied: {:?}", denial.reason())
        }
        TransitionOutcome::Failed(failure) => {
            panic!("PW source Store open failed: {:?}", failure.cause())
        }
        TransitionOutcome::RebindRequired(denial) => {
            panic!("PW source Store open rebind: {:?}", denial.reason())
        }
        TransitionOutcome::Stale(denial) => {
            panic!("PW source Store open stale: {:?}", denial.reason())
        }
        TransitionOutcome::Deferred(never) => match never {},
    };
    (serving, placement)
}
