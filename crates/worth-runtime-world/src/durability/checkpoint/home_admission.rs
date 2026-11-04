//! Home carries durable-owner journal evidence, never a projected memory registry.
use super::{PreparedWorldCheckpoint, WorldCheckpointSubject};
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::*;
use worth_relational::facade::durability::{
    CheckpointAdmissionAuthenticationMismatch as Authentication,
    CheckpointAdmissionBindingMismatch as Binding, RelationalCheckpointAdmissionArtifact as Image,
    RelationalCheckpointAdmissionRefusal as Refusal,
};

const MAGIC: &[u8; 8] = b"WRWMREG1";
const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
const PAYLOAD: usize = 229;
const DOMAIN: &[u8] = b"worth.world.receipt-registry.home.v1\0";

impl PreparedWorldCheckpoint {
    /// Coordinates of this actual protected capture; descriptive, not authority.
    pub fn home_admission_coordinates(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(HomeIdentity, [u64; 3], [u8; 32]), Refusal> {
        let subject = self.publication_subject();
        let cut = subject.coverage();
        Ok((
            subject.binding().owner().home(),
            [
                cut.branches.position,
                cut.custody.position,
                cut.occurrences.position,
            ],
            digest(self.bytes(), request)?,
        ))
    }
    pub fn capture_home_admission_companion(
        &self,
        image: &Image,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        let (home, through, world_digest) = self.home_admission_coordinates(request)?;
        let CheckpointAdmissionScope::Home {
            home: observed,
            world_through,
            world_image_digest,
            ..
        } = image.cut().scope()
        else {
            return Err(Refusal::BindingMismatch(Binding::Scope));
        };
        if observed != &home {
            return Err(Refusal::BindingMismatch(Binding::Home));
        }
        if world_through != &through {
            return Err(Refusal::BindingMismatch(Binding::Cut));
        }
        if world_image_digest != &world_digest {
            return Err(Refusal::BindingMismatch(Binding::WorldImage));
        }
        let payload = payload(image, request)?;
        let hash = companion_digest(&payload, request)?;
        let mut bytes = ExecutionRequestArrayBuilder::allocate(*request, PAYLOAD + 32)
            .map_err(Refusal::Buffer)?;
        bytes.extend_from_slice(&payload).map_err(Refusal::Buffer)?;
        bytes.extend_from_slice(&hash).map_err(Refusal::Buffer)?;
        bytes.seal_immutable_bytes().map_err(Refusal::Buffer)
    }
}

impl WorldCheckpointSubject {
    pub const HOME_ADMISSION_COMPANION_PROTOCOL: BoundaryProtocolIdentity =
        BoundaryProtocolIdentity::new("worth.world.receipt-registry");

    /// Validate selected companion DATA before descriptor effects. The unchanged
    /// World section/journals remain the sole durable World admission path.
    pub fn validate_home_admission_companion(
        image: &Image,
        companion: &[u8],
        world: &[u8],
        home: HomeIdentity,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        if companion.is_empty() {
            return Err(Refusal::MissingArtifact);
        }
        if companion.get(..8) != Some(MAGIC.as_slice()) {
            return Err(Refusal::Malformed);
        }
        let version = companion
            .get(8..12)
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or(Refusal::Malformed)?;
        let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(version))
            .map_err(|_| Refusal::InvalidVersion)?;
        BoundaryProtocolCompatibilityWindow::inclusive(VERSION, VERSION)
            .admit(version)
            .map_err(Refusal::UnsupportedVersion)?;
        let CheckpointAdmissionScope::Home {
            home: recorded,
            world_image_digest,
            ..
        } = image.cut().scope()
        else {
            return Err(Refusal::BindingMismatch(Binding::Scope));
        };
        if recorded != &home {
            return Err(Refusal::BindingMismatch(Binding::Home));
        }
        if world_image_digest != &digest(world, request)? {
            return Err(Refusal::BindingMismatch(Binding::WorldImage));
        }
        // Tag1 is DurableOwnerJournal. No memory movement or registry can be
        // inferred from Home bytes; interim capture-proof restoration is separate.
        if companion.get(12) != Some(&1) {
            return Err(Refusal::BindingMismatch(Binding::Scope));
        }
        let expected = payload(image, request)?;
        if companion.len() != PAYLOAD + 32
            || companion[..PAYLOAD] != expected
            || companion[PAYLOAD..] != companion_digest(&expected, request)?
        {
            return Err(Refusal::AuthenticationMismatch(Authentication::Directory));
        }
        Ok(())
    }
}

fn payload(image: &Image, request: &ExecutionRequest<'_, '_>) -> Result<[u8; PAYLOAD], Refusal> {
    let CheckpointAdmissionScope::Home {
        home,
        native_origin,
        complete_source_digest,
        world_through,
        world_image_digest,
    } = image.cut().scope()
    else {
        return Err(Refusal::BindingMismatch(Binding::Scope));
    };
    let native_image = digest(image.as_bytes(), request)?;
    let mut bytes = [0; PAYLOAD];
    let mut cursor = 0;
    for field in [
        MAGIC.as_slice(),
        &VERSION.get().to_le_bytes(),
        &[1],
        &home.bytes(),
        &image.cut().canonical_through().to_le_bytes(),
        &image.cut().transitions_through().to_le_bytes(),
        image.cut().checkpoint_digest(),
        native_origin,
        complete_source_digest,
        &world_through[0].to_le_bytes(),
        &world_through[1].to_le_bytes(),
        &world_through[2].to_le_bytes(),
        world_image_digest,
        &native_image,
    ] {
        bytes[cursor..cursor + field.len()].copy_from_slice(field);
        cursor += field.len();
    }
    Ok(bytes)
}
fn companion_digest(
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], Refusal> {
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    update(&mut hash, payload, request)?;
    Ok(hash.finalize().into())
}
fn digest(bytes: &[u8], request: &ExecutionRequest<'_, '_>) -> Result<[u8; 32], Refusal> {
    let mut hash = Sha256::new();
    update(&mut hash, bytes, request)?;
    Ok(hash.finalize().into())
}
fn update(
    hash: &mut Sha256,
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    for chunk in bytes.chunks(64 << 10) {
        request
            .run_native_with_checkpoint(|context| context.checkpoint(chunk.len() as u64))
            .map_err(|cause| Refusal::Request(cause.into()))?
            .0
            .map_err(|cause| {
                Refusal::Request(worth_execution::WorkCeilingDenial::Stopped(cause).into())
            })?;
        hash.update(chunk);
    }
    Ok(())
}
