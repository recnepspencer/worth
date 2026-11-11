//! Authored runtime registrations, explicitly separate from memoization.
use super::{codec, framing, SignalAuthoredImageDenial as Denial};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::BoundaryProtocolIdentity;

pub const SIGNAL_RUNTIME_AUTHORED_IMAGE_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.signal.runtime-authored-image");
#[derive(Clone, Debug)]
pub struct SignalRuntimeAuthoredImage {
    bytes: ExecutionImmutableBytes,
    range: std::ops::Range<usize>,
    branch: worth_foundational::facade::DurableSignalBranchIdentity,
}
impl SignalRuntimeAuthoredImage {
    pub const fn branch(&self) -> worth_foundational::facade::DurableSignalBranchIdentity {
        self.branch
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes.bytes()[self.range.clone()]
    }
    pub fn readmit_bytes(
        execution: &ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, Denial> {
        framing::admit_work(execution, bytes.len().saturating_mul(2))?;
        let branch = validate(bytes)?;
        let mut output =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, bytes.len())
                .map_err(Denial::Backing)?;
        output.extend_from_slice(bytes).map_err(Denial::Backing)?;
        Ok(Self {
            branch,
            range: 0..bytes.len(),
            bytes: output
                .seal()
                .map_err(Denial::Backing)?
                .into_immutable_bytes(),
        })
    }
    pub(crate) fn capture<T: Copy + Ord>(
        execution: &ExecutionRequest<'_, '_>,
        branch: worth_foundational::facade::DurableSignalBranchIdentity,
        view: &codec::RuntimeDefinitionView<'_, T>,
    ) -> Result<Self, Denial> {
        framing::encode(
            framing::AuthoredSectionKind::Runtime,
            branch,
            execution,
            &mut |out| codec::write_runtime_rows(out, view),
        )
        .map(|bytes| Self {
            range: 0..bytes.bytes().len(),
            bytes,
            branch,
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
    pub(crate) fn rows(&self, graph_instance: u64) -> Result<codec::RuntimeDefinitionRows, Denial> {
        codec::read_runtime_rows(
            framing::payload(framing::AuthoredSectionKind::Runtime, self.as_bytes())?,
            true,
            graph_instance,
        )
    }
}
pub(crate) fn validate(
    bytes: &[u8],
) -> Result<worth_foundational::facade::DurableSignalBranchIdentity, Denial> {
    codec::read_runtime_rows(
        framing::payload(framing::AuthoredSectionKind::Runtime, bytes)?,
        false,
        0,
    )
    .map(|rows| rows.branch)
}
impl worth_foundational::facade::DurableArtifactRetention for SignalRuntimeAuthoredImage {
    const RETENTION_CLASS: worth_foundational::facade::RetentionClass =
        worth_foundational::facade::RetentionClass::Authoritative;
}
