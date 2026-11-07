//! A declaration-only image: no values, dependency snapshots or scheduling state.
mod codec;
mod framing;
pub(crate) use codec::write_definition;
pub(crate) use framing::encode;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

pub const SIGNAL_AUTHORED_IMAGE_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.signal.authored-image");
pub const SIGNAL_AUTHORED_IMAGE_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);

pub(super) fn validate_bytes(bytes: &[u8]) -> Result<(), SignalAuthoredImageDenial> {
    codec::validate(framing::payload(bytes)?)
}

/// Descriptive source bytes. Admission validates the format, not operational ownership.
/// Only Signal owner readmission may turn this source into live branch authority.
#[derive(Debug, Clone)]
pub struct SignalAuthoredImage {
    bytes: ExecutionImmutableBytes,
}

impl SignalAuthoredImage {
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }

    pub fn readmit_bytes(
        execution: &ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, SignalAuthoredImageDenial> {
        execution
            .run_native_with_checkpoint(|context| context.checkpoint(bytes.len() as u64))
            .map_err(SignalAuthoredImageDenial::Request)?
            .0
            .map_err(|cause| {
                SignalAuthoredImageDenial::Request(worth_execution::WorkCeilingDenial::Stopped(
                    cause,
                ))
            })?;
        let payload = framing::payload(bytes)?;
        codec::validate(payload)?;
        let mut out =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, bytes.len())
                .map_err(SignalAuthoredImageDenial::Backing)?;
        out.extend_from_slice(bytes)
            .map_err(SignalAuthoredImageDenial::Backing)?;
        Ok(Self {
            bytes: out
                .seal()
                .map_err(SignalAuthoredImageDenial::Backing)?
                .into_immutable_bytes(),
        })
    }

    pub(crate) fn from_encoded(bytes: ExecutionImmutableBytes) -> Self {
        Self { bytes }
    }
}

#[derive(Debug)]
pub enum SignalAuthoredImageDenial {
    Request(worth_execution::WorkCeilingDenial),
    Malformed,
    SizeLimit,
    ChecksumMismatch,
    UnsupportedVersion(worth_foundational::facade::BoundaryProtocolUnsupportedVersion),
    StorageProfileMismatch {
        recorded_aspects: u32,
        supported_aspects: u32,
    },
    Backing(worth_execution::ExecutionRequestArrayDenial),
}

impl worth_foundational::facade::DurableArtifactRetention for SignalAuthoredImage {
    const RETENTION_CLASS: worth_foundational::facade::RetentionClass =
        worth_foundational::facade::RetentionClass::Authoritative;
}
