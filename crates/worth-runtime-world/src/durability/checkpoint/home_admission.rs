//! Home carries durable-owner journal evidence, never a projected memory registry.
use super::{PreparedWorldCheckpoint, WorldCheckpointSubject};
use sha2::{Digest, Sha256};
use worth_execution::{
    decode_framed_envelope, encode_framed_bytes, ExecutionImmutableBytes, ExecutionRequest,
    FramedEnvelopeError, FramedEnvelopeSpec,
};
use worth_foundational::facade::*;
use worth_relational::facade::durability::{
    CheckpointAdmissionAuthenticationMismatch as Authentication,
    CheckpointAdmissionBindingMismatch as Binding, RelationalCheckpointAdmissionArtifact as Image,
    RelationalCheckpointAdmissionRefusal as Refusal,
};

const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(2);
const PAYLOAD: usize = 217;
const SPEC: FramedEnvelopeSpec = FramedEnvelopeSpec {
    magic: *b"WRWMREG1",
    supported_versions: &[VERSION.get()],
    max_payload_bytes: PAYLOAD as u64,
};

fn frame_refusal(error: FramedEnvelopeError) -> Refusal {
    match error {
        FramedEnvelopeError::Execution(cause) => Refusal::Request(cause.into()),
        FramedEnvelopeError::UnsupportedVersion { found } => {
            BoundaryProtocolVersion::try_new(found).map_or(Refusal::InvalidVersion, |found| {
                match BoundaryProtocolCompatibilityWindow::inclusive(VERSION, VERSION).admit(found)
                {
                    Ok(_) => Refusal::Malformed,
                    Err(cause) => Refusal::UnsupportedVersion(cause),
                }
            })
        }
        FramedEnvelopeError::ChecksumMismatch => {
            Refusal::AuthenticationMismatch(Authentication::Directory)
        }
        _ => Refusal::Malformed,
    }
}

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
        } = image.cut().scope();
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
        let bytes = encode_framed_bytes(
            SPEC,
            VERSION.get(),
            request,
            |sink| sink(&payload),
            CompanionEmission::Buffer,
        )
        .map_err(|error| match error {
            CompanionEmission::Frame(cause) => frame_refusal(cause),
            CompanionEmission::Buffer(cause) => Refusal::Buffer(cause),
        })?;
        Ok(bytes)
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
        let (_, companion) =
            decode_framed_envelope(SPEC, companion, request).map_err(frame_refusal)?;
        let CheckpointAdmissionScope::Home {
            home: recorded,
            world_image_digest,
            ..
        } = image.cut().scope();
        if recorded != &home {
            return Err(Refusal::BindingMismatch(Binding::Home));
        }
        if world_image_digest != &digest(world, request)? {
            return Err(Refusal::BindingMismatch(Binding::WorldImage));
        }
        // Tag1 is DurableOwnerJournal. No memory movement or registry can be
        // inferred from Home bytes; interim capture-proof restoration is separate.
        if companion.first() != Some(&1) {
            return Err(Refusal::BindingMismatch(Binding::Scope));
        }
        let expected = payload(image, request)?;
        if companion != expected {
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
    } = image.cut().scope();
    let native_image = digest(image.as_bytes(), request)?;
    let mut bytes = [0; PAYLOAD];
    let mut cursor = 0;
    for field in [
        [1].as_slice(),
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
#[derive(Debug)]
enum CompanionEmission {
    Frame(FramedEnvelopeError),
    Buffer(worth_execution::ExecutionRequestArrayDenial),
}

impl From<FramedEnvelopeError> for CompanionEmission {
    fn from(error: FramedEnvelopeError) -> Self {
        Self::Frame(error)
    }
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
