//! Bounded untrusted transport DATA; framing never issues admission authority.
use super::wire;
use contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
use std::ops::Range;
use worth_execution::{ExecutionArray, ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{BoundaryProtocolIdentity, CheckpointAdmissionCut};
use worth_relational_branch::internal_port::durability::checkpoint_admission::read_image::{
    CheckpointAdmissionReadImage, DecodedDirectory,
};
use worth_relational_contracts::internal_port as contracts;

/// Parsed and independently hashed projections, without a protected selection
/// or capture origin. It cannot construct a Native root or performed result.
#[derive(Debug, Clone)]
pub struct RelationalCheckpointAdmissionArtifact(CheckpointAdmissionReadImage);

impl RelationalCheckpointAdmissionArtifact {
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        self.0.same_owner(&other.0)
    }
    pub const PROTOCOL: BoundaryProtocolIdentity =
        BoundaryProtocolIdentity::new("worth.relational.checkpoint-admission");

    pub fn from_untrusted_region(
        bytes: ExecutionImmutableBytes,
        region: Range<usize>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let decoded = wire::read(&bytes, region.clone(), request)?;
        CheckpointAdmissionReadImage::from_directory(bytes, region, decoded, request).map(Self)
    }

    pub(super) fn from_emission(
        bytes: ExecutionImmutableBytes,
        directory: DecodedDirectory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let region = 0..bytes.len();
        CheckpointAdmissionReadImage::from_directory(bytes, region, directory, request).map(Self)
    }

    pub(crate) fn carry_rows(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionArray<super::EncodedAdmissionRow>, Refusal> {
        let mut rows =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, self.0.rows().len())
                .map_err(Refusal::Buffer)?;
        for row in self.0.rows() {
            let mut groups: [Option<ExecutionImmutableBytes>; 8] = std::array::from_fn(|_| None);
            for (slot, range) in groups.iter_mut().zip(&row.groups) {
                let bytes = self.0.group(range);
                let mut buffer =
                    worth_execution::ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
                        .map_err(Refusal::Buffer)?;
                contracts::durability::data::recovery_stop::account_work(
                    request,
                    bytes.len() as u64,
                )
                .map_err(contracts::durability::checkpoint_admission::refusal::native_stop)?;
                buffer.extend_from_slice(bytes).map_err(Refusal::Buffer)?;
                *slot = Some(buffer.seal_immutable_bytes().map_err(Refusal::Buffer)?);
            }
            rows.push(super::EncodedAdmissionRow {
                key: row.key,
                groups: groups.map(|group| group.expect("all eight original meanings copied")),
            })
            .map_err(Refusal::Buffer)?;
        }
        rows.seal().map_err(Refusal::Buffer)
    }

    pub(super) fn selected_image(&self) -> &CheckpointAdmissionReadImage {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn cut(&self) -> &CheckpointAdmissionCut {
        self.0.cut()
    }

    pub(crate) fn require_manifest(
        &self,
        manifest: &crate::durability::data::RelationalCheckpointManifest,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        crate::durability::authority::checkpoint_complete::admission_projection::root_directory::validate_manifest(
            self.0.roots(), manifest, request,
        )
    }
}
