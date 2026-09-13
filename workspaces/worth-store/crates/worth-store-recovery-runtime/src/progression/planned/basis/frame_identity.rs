use worth_store_physical_format::PersistedPhysicalDataFrameSubject;
use worth_store_recovery_physics::PhysicalRedoTargetIdentity;

pub(super) fn frame_identity(
    subject: PersistedPhysicalDataFrameSubject,
) -> PhysicalRedoTargetIdentity {
    match subject {
        PersistedPhysicalDataFrameSubject::InlinePage(page) => {
            PhysicalRedoTargetIdentity::InlinePage {
                segment: page.segment_id().get(),
                page: page.page_id().get(),
                generation: page.generation().get(),
            }
        }
        PersistedPhysicalDataFrameSubject::ExtentChunk(chunk) => {
            PhysicalRedoTargetIdentity::ExtentChunk {
                extent: chunk.extent_cell().extent_id().get(),
                generation: chunk.extent_cell().generation().get(),
                chunk: chunk.ordinal(),
            }
        }
    }
}
