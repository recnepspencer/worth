use worth_store::physical_runtime::{
    AdmittedDirtyFrame, AdmittedPhysicalWriteback, PhysicalWorkSubmissionReceipt,
    PhysicalWritebackTransitionFailure, PreparedPhysicalWriteback, ReadyPhysicalWriteback,
};
use worth_store_physical_format::RecordFrameCoordinate;

fn unavailable<T>() -> T {
    loop {
        std::hint::spin_loop();
    }
}

fn forge_dirty(coordinate: RecordFrameCoordinate) -> AdmittedDirtyFrame {
    AdmittedDirtyFrame::new(coordinate, unavailable(), unavailable())
}

fn extract_dirty_frame(
    dirty: AdmittedDirtyFrame,
) -> worth_store_buffer_pool::DirtyPhysicalFrame {
    dirty.frame
}

fn extract_submission(
    prepared: PreparedPhysicalWriteback,
) -> PhysicalWorkSubmissionReceipt {
    prepared.receipt
}

fn extract_ready(
    ready: ReadyPhysicalWriteback,
) -> worth_store::physical_runtime::ReadyPhysicalWork {
    ready.ready
}

fn extract_admitted(
    admitted: AdmittedPhysicalWriteback,
) -> worth_store::physical_runtime::ResourceAdmittedPhysicalWork {
    admitted.work
}

fn extract_failed_dirty(
    failure: PhysicalWritebackTransitionFailure,
) -> AdmittedDirtyFrame {
    failure.dirty
}

fn main() {}
