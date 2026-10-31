//! Capture and frame preparation under one original request scope, before effect.
use super::ApplicationHomeCheckpointDenial;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_store::physical_runtime::{AnchoredRecordMember, ServingPhysicalRuntime};
use worth_store_relational::{PreparedRelationalSection, RelationalSectionCheckpointDenial};

pub(super) struct PreparedHomeCheckpoint<'section> {
    pub(super) native: PreparedRelationalSection<'section>,
    pub(super) payload: crate::frame_encoding::EncodedHomeFrame,
    pub(super) members: Vec<AnchoredRecordMember>,
    pub(super) member_memory: Option<ExecutionMemoryReservation>,
    pub(super) generation: u64,
}

pub(super) fn capture<'section>(
    serving: &ServingPhysicalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    native: impl FnOnce() -> Result<
        PreparedRelationalSection<'section>,
        RelationalSectionCheckpointDenial,
    >,
) -> Result<PreparedHomeCheckpoint<'section>, ApplicationHomeCheckpointDenial> {
    use ApplicationHomeCheckpointDenial as Denial;
    crate::request_stage::prepare(
        execution,
        || {
            let native = native().map_err(Denial::Native)?;
            // Native owns the actual read-custody fence BEFORE reading the anchor.
            let previous = super::read(serving)?;
            let generation = previous
                .as_ref()
                .map_or(0, |manifest| manifest.generation());
            if let Some(previous) = &previous {
                native
                    .validate_previous(previous.relational().0)
                    .map_err(Denial::Native)?;
            }
            let (members, member_memory) = native
                .prepare_members(
                    previous.as_ref().map_or(&[], |manifest| manifest.members()),
                    execution,
                )
                .map_err(Denial::Native)?;
            let size = super::manifest::HEADER_BYTES
                .checked_add(native.bytes().len())
                .filter(|size| *size <= 1 << 30)
                .ok_or(Denial::SizeLimit)?;
            let payload = crate::frame_encoding::encode(size, execution, |bytes| {
                super::manifest::emit(native.bytes(), bytes)
            })
            .map_err(Denial::Execution)?;
            Ok(PreparedHomeCheckpoint {
                native,
                payload,
                members,
                member_memory,
                generation,
            })
        },
        Denial::Execution,
    )
}
