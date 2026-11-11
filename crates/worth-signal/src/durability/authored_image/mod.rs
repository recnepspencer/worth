//! A declaration-only image: no values, dependency snapshots or scheduling state.
mod codec;
pub(crate) use codec::RuntimeDefinitionRows;
#[cfg(test)]
pub(crate) use codec::{read_runtime_rows, write_runtime_rows, Write};
mod framing;
pub(crate) use framing::admit_work;
mod runtime_image;
pub(crate) use codec::{write_definition, AuthoredGraphRows};
pub(crate) use codec::{ComputationDefinitionView, KeyedNodeDefinitionView, RuntimeDefinitionView};
use framing::AuthoredSectionKind;
pub(crate) use runtime_image::validate as validate_runtime_bytes;
pub use runtime_image::{SignalRuntimeAuthoredImage, SIGNAL_RUNTIME_AUTHORED_IMAGE_PROTOCOL};
pub(crate) fn encode(
    execution: &ExecutionRequest<'_, '_>,
    branch: worth_foundational::facade::DurableSignalBranchIdentity,
    emit: &mut dyn FnMut(&mut codec::Write<'_>) -> Result<(), SignalAuthoredImageDenial>,
) -> Result<SignalAuthoredImage, SignalAuthoredImageDenial> {
    framing::encode(AuthoredSectionKind::Graph, branch, execution, emit)
        .map(|bytes| SignalAuthoredImage::from_encoded(bytes, branch))
}
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

pub const SIGNAL_AUTHORED_IMAGE_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.signal.authored-image");
pub const SIGNAL_AUTHORED_IMAGE_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);

pub(super) fn validate_bytes(
    bytes: &[u8],
) -> Result<worth_foundational::facade::DurableSignalBranchIdentity, SignalAuthoredImageDenial> {
    codec::validate(framing::payload(AuthoredSectionKind::Graph, bytes)?)
}

/// Descriptive source bytes. Admission validates the format, not operational ownership.
/// Only Signal owner readmission may turn this source into live branch authority.
#[derive(Debug, Clone)]
pub struct SignalAuthoredImage {
    branch: worth_foundational::facade::DurableSignalBranchIdentity,
    range: std::ops::Range<usize>,
    bytes: ExecutionImmutableBytes,
}

impl SignalAuthoredImage {
    pub const fn branch(&self) -> worth_foundational::facade::DurableSignalBranchIdentity {
        self.branch
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes.bytes()[self.range.clone()]
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
        let payload = framing::payload(AuthoredSectionKind::Graph, bytes)?;
        let branch = codec::validate(payload)?;
        let mut out =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, bytes.len())
                .map_err(SignalAuthoredImageDenial::Backing)?;
        out.extend_from_slice(bytes)
            .map_err(SignalAuthoredImageDenial::Backing)?;
        Ok(Self {
            branch,
            range: 0..bytes.len(),
            bytes: out
                .seal()
                .map_err(SignalAuthoredImageDenial::Backing)?
                .into_immutable_bytes(),
        })
    }

    pub(in crate::durability) fn from_frame_bytes(
        bytes: ExecutionImmutableBytes,
        range: std::ops::Range<usize>,
        branch: worth_foundational::facade::DurableSignalBranchIdentity,
    ) -> Self {
        Self {
            bytes,
            range,
            branch,
        }
    }

    pub(crate) fn read_graph_rows(
        &self,
        graph_instance: u64,
    ) -> Result<AuthoredGraphRows, SignalAuthoredImageDenial> {
        codec::read_graph_rows(
            framing::payload(AuthoredSectionKind::Graph, self.as_bytes())?,
            graph_instance,
        )
    }

    pub(crate) fn from_encoded(
        bytes: ExecutionImmutableBytes,
        branch: worth_foundational::facade::DurableSignalBranchIdentity,
    ) -> Self {
        Self {
            range: 0..bytes.bytes().len(),
            bytes,
            branch,
        }
    }
}

#[derive(Debug)]
pub enum SignalAuthoredImageDenial {
    Request(worth_execution::WorkCeilingDenial),
    Malformed,
    ForeignBranch,
    ConditionalLoweringOwnerMissing,
    ConditionalNodeMissing,
    ConditionalDefinitionMissing,
    ForeignConditionalBinding,
    ConditionalInstallation(crate::data::conditional_execution::SignalConditionalContractDenial),
    Topology(Box<crate::data::error::SignalError>),
    TierVocabulary(super::SignalDurableTierVocabularyDenial),
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
