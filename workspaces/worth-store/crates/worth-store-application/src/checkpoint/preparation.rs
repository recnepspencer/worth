//! Capture and frame preparation under one original request scope, before effect.
use super::ApplicationHomeCheckpointDenial;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_store::physical_runtime::{AnchoredRecordMember, ServingPhysicalRuntime};
use worth_store_relational::{
    PreparedCompleteRelationalSection, RelationalSectionCheckpointDenial,
};

pub(super) struct PreparedHomeCheckpoint {
    pub(super) native: PreparedCompleteRelationalSection,
    pub(super) world: worth_runtime_world::facade::PreparedWorldCheckpoint,
    pub(super) payload: crate::frame_encoding::EncodedHomeFrame,
    pub(super) members: Vec<AnchoredRecordMember>,
    pub(super) member_memory: Option<ExecutionMemoryReservation>,
    pub(super) generation: u64,
    pub(super) identity: super::owner::HomeManifestPublication,
}

pub(super) struct PreparedHomeInput {
    pub(super) payload: crate::frame_encoding::EncodedHomeFrame,
    pub(super) members: Vec<AnchoredRecordMember>,
    pub(super) member_memory: Option<ExecutionMemoryReservation>,
    pub(super) generation: u64,
    pub(super) identity: super::owner::HomeManifestPublication,
}

impl PreparedHomeCheckpoint {
    pub(super) fn into_parts(
        self,
    ) -> (
        PreparedCompleteRelationalSection,
        worth_runtime_world::facade::PreparedWorldCheckpoint,
        PreparedHomeInput,
    ) {
        (
            self.native,
            self.world,
            PreparedHomeInput {
                payload: self.payload,
                members: self.members,
                member_memory: self.member_memory,
                generation: self.generation,
                identity: self.identity,
            },
        )
    }
}

pub(super) fn capture(
    serving: &ServingPhysicalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    world: &worth_runtime_world::facade::WorldCheckpointPort,
    native: impl FnOnce()
        -> Result<PreparedCompleteRelationalSection, RelationalSectionCheckpointDenial>,
) -> Result<PreparedHomeCheckpoint, ApplicationHomeCheckpointDenial> {
    use ApplicationHomeCheckpointDenial as Denial;
    crate::request_stage::prepare(
        execution,
        || {
            let world = world
                .prepare_home_checkpoint(execution)
                .map_err(Denial::WorldReadmission)?;
            let native = native().map_err(Denial::Native)?;
            // Native owns the actual read-custody fence BEFORE reading the anchor.
            let previous = super::read(serving, execution)?;
            let generation = previous
                .as_ref()
                .map_or(0, |manifest| manifest.generation());
            if let Some(previous) = &previous {
                world
                    .validate_previous_selected(previous.world(), execution)
                    .map_err(Denial::WorldReadmission)?;
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
                .and_then(|size| size.checked_add(world.bytes().len()))
                .filter(|size| *size <= 1 << 30)
                .ok_or(Denial::SizeLimit)?;
            let payload = crate::frame_encoding::encode(size, execution, |bytes| {
                super::manifest::emit(native.bytes(), world.bytes(), bytes)
            })
            .map_err(Denial::Execution)?;
            let identity = publication_identity(&payload.bytes, generation, execution)?;
            Ok(PreparedHomeCheckpoint {
                native,
                world,
                payload,
                members,
                member_memory,
                generation,
                identity,
            })
        },
        Denial::Execution,
    )
}
fn publication_identity(
    bytes: &[u8],
    previous: u64,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<super::owner::HomeManifestPublication, ApplicationHomeCheckpointDenial> {
    use sha2::{Digest, Sha256};
    use ApplicationHomeCheckpointDenial as Denial;
    let generation = previous.checked_add(1).ok_or(Denial::SizeLimit)?;
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(64 << 10) {
        execution
            .run_native_with_checkpoint(|context| context.checkpoint(chunk.len() as u64))
            .map_err(Denial::Execution)?
            .0
            .map_err(|stop| Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        digest.update(chunk);
    }
    Ok(super::owner::HomeManifestPublication {
        generation,
        payload_sha256: digest.finalize().into(),
        payload_bytes: bytes.len(),
    })
}
