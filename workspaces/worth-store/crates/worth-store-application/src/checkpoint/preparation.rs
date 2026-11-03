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

pub(super) fn prepare_attempt(
    serving: &ServingPhysicalRuntime,
    owner: &super::owner::HomeCheckpointOwner,
    execution: &ExecutionRequest<'_, '_>,
    world: &worth_runtime_world::facade::WorldCheckpointPort,
    native: impl FnOnce()
        -> Result<PreparedCompleteRelationalSection, RelationalSectionCheckpointDenial>,
) -> Result<
    (
        std::sync::Arc<super::owner::HomeCheckpointAttempt>,
        super::physical_publication::PublicationDriverLoan,
        PreparedHomeInput,
    ),
    ApplicationHomeCheckpointDenial,
> {
    use ApplicationHomeCheckpointDenial as Denial;
    // Keep the actual capture outside Execution's generic scope carriers. The
    // Native/World owners are large; copying their possible result through
    // each carrier consumed the default thread stack before source admission.
    let mut captured = None;
    execution
        .prepare_native(|_| {
            captured = Some(capture_in_scope(serving, execution, world, native));
            // Execution still sees domain refusal as refusal. Its carrier need
            // not copy the retained physical failure or prepared owner.
            match captured.as_ref().expect("capture just completed") {
                Ok(_) => Ok(()),
                Err(_) => Err(CaptureRefused),
            }
        })
        .map_err(Denial::Execution)?
        .unwrap_or(());
    let (native, world, input) = captured
        .expect("completed preparation retained its original capture")?
        .into_parts();
    let (attempt, driver) = owner.install(native, world, execution)?;
    Ok((attempt, driver, input))
}

struct CaptureRefused;

fn capture_in_scope(
    serving: &ServingPhysicalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    world: &worth_runtime_world::facade::WorldCheckpointPort,
    native: impl FnOnce()
        -> Result<PreparedCompleteRelationalSection, RelationalSectionCheckpointDenial>,
) -> Result<PreparedHomeCheckpoint, ApplicationHomeCheckpointDenial> {
    use ApplicationHomeCheckpointDenial as Denial;
    let world = world
        .prepare_home_checkpoint(execution)
        .map_err(Denial::WorldReadmission)?;
    capture_after_world(serving, execution, world, native)
}

fn capture_after_world(
    serving: &ServingPhysicalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    world: worth_runtime_world::facade::PreparedWorldCheckpoint,
    native: impl FnOnce()
        -> Result<PreparedCompleteRelationalSection, RelationalSectionCheckpointDenial>,
) -> Result<PreparedHomeCheckpoint, ApplicationHomeCheckpointDenial> {
    use ApplicationHomeCheckpointDenial as Denial;
    let native = native().map_err(Denial::Native)?;
    // Native owns the actual read-custody fence BEFORE reading the anchor.
    let previous = super::read(serving, execution)?;
    validate_previous(&native, &world, previous.as_ref(), execution)?;
    frame(native, world, previous.as_ref(), execution)
}

fn validate_previous(
    native: &PreparedCompleteRelationalSection,
    world: &worth_runtime_world::facade::PreparedWorldCheckpoint,
    previous: Option<&super::SelectedHomeManifest>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeCheckpointDenial> {
    use ApplicationHomeCheckpointDenial as Denial;
    if let Some(previous) = previous {
        world
            .validate_previous_selected(previous.world(), execution)
            .map_err(Denial::WorldReadmission)?;
        native
            .validate_previous(previous.relational().0)
            .map_err(Denial::Native)?;
    }
    Ok(())
}

fn frame(
    native: PreparedCompleteRelationalSection,
    world: worth_runtime_world::facade::PreparedWorldCheckpoint,
    previous: Option<&super::SelectedHomeManifest>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<PreparedHomeCheckpoint, ApplicationHomeCheckpointDenial> {
    use ApplicationHomeCheckpointDenial as Denial;
    let generation = previous.map_or(0, |manifest| manifest.generation());
    let (members, member_memory) = native
        .prepare_members(
            previous.map_or(&[], |manifest| manifest.members()),
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
