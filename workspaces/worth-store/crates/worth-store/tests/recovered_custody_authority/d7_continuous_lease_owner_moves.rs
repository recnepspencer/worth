use worth_store::physical_runtime::{
    AdmittedPhysicalRuntime, FilesystemMediaAdmission, PhysicalRecordOpen,
    RecoveredMediaOwnedPhysicalRuntime, RecoveredPhysicalRuntimeCore,
    RecoveredPhysicalServingHandoff,
};

fn recovery_owner_moves(core: RecoveredPhysicalRuntimeCore) {
    let _first = core.into_serving_handoff();
    let _second = core.into_serving_handoff();
}

fn handoff_owner_moves(
    handoff: RecoveredPhysicalServingHandoff,
    runtime: AdmittedPhysicalRuntime,
    admission: FilesystemMediaAdmission,
) {
    let _admitted = handoff.admit_serving(runtime, admission);
    drop(handoff);
}

fn admitted_owner_moves(media: RecoveredMediaOwnedPhysicalRuntime, request: PhysicalRecordOpen) {
    let _serving = media.open_record_store(request);
    drop(media);
}

fn main() {}
