//! Bounded untrusted transport DATA; framing never issues admission authority.
use super::{wire, RelationalCheckpointAdmissionRefusal as Refusal};
use std::ops::Range;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionOrderedMap,
    ExecutionRequest,
};
use worth_foundational::facade::{BoundaryProtocolIdentity, CheckpointAdmissionCut};

/// Parsed and independently hashed projections, without a protected selection
/// or capture origin. It cannot construct a Native root or performed result.
#[derive(Debug, Clone)]
pub struct RelationalCheckpointAdmissionArtifact(Arc<ArtifactBacking>);

#[derive(Debug)]
struct ArtifactBacking {
    bytes: ExecutionImmutableBytes,
    region: Range<usize>,
    cut: CheckpointAdmissionCut,
    roots: Range<usize>,
    rows: ExecutionArray<wire::AdmissionRowIndex>,
    commits: ExecutionOrderedMap<u64, usize>,
    positions: ExecutionOrderedMap<
        crate::history::data::CommitId,
        crate::history::data::RequiredOutputPublicationPositions,
    >,
    _memory: Option<ExecutionMemoryReservation>,
}

impl RelationalCheckpointAdmissionArtifact {
    pub const PROTOCOL: BoundaryProtocolIdentity =
        BoundaryProtocolIdentity::new("worth.relational.checkpoint-admission");

    pub fn from_untrusted_region(
        bytes: ExecutionImmutableBytes,
        region: Range<usize>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let decoded = wire::read(&bytes, region.clone(), request)?;
        Self::from_directory(bytes, region, decoded, request)
    }

    pub(super) fn from_emission(
        bytes: ExecutionImmutableBytes,
        directory: wire::DecodedDirectory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let region = 0..bytes.len();
        Self::from_directory(bytes, region, directory, request)
    }

    fn from_directory(
        bytes: ExecutionImmutableBytes,
        region: Range<usize>,
        decoded: wire::DecodedDirectory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let size = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ArtifactBacking>())
            .map_err(|_| Refusal::CapacityOverflow)?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(size as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        Ok(Self(Arc::new(ArtifactBacking {
            bytes,
            region,
            cut: decoded.cut,
            roots: decoded.roots,
            rows: decoded.rows,
            commits: decoded.commits,
            positions: decoded.positions,
            _memory: memory,
        })))
    }

    pub(crate) fn carry_rows(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionArray<super::EncodedAdmissionRow>, Refusal> {
        let mut rows =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, self.0.rows.len())
                .map_err(Refusal::Buffer)?;
        for row in self.0.rows.iter() {
            let mut groups: [Option<ExecutionImmutableBytes>; 8] = std::array::from_fn(|_| None);
            for (slot, range) in groups.iter_mut().zip(&row.groups) {
                let bytes = self.group(range);
                let mut buffer =
                    worth_execution::ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
                        .map_err(Refusal::Buffer)?;
                crate::durability::artifact_payload_emission::request::account_work(
                    request,
                    bytes.len() as u64,
                )
                .map_err(super::native_stop)?;
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
    pub fn as_bytes(&self) -> &[u8] {
        &self.0.bytes[self.0.region.clone()]
    }
    pub fn cut(&self) -> &CheckpointAdmissionCut {
        &self.0.cut
    }

    pub(crate) fn require_manifest(
        &self,
        manifest: &crate::durability::data::RelationalCheckpointManifest,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        crate::durability::authority::checkpoint_complete::admission_projection::root_directory::validate_manifest(
            self.group(&self.0.roots), manifest, request,
        )
    }

    pub(super) fn group(&self, range: &Range<usize>) -> &[u8] {
        &self.0.bytes[range.clone()]
    }

    pub(super) fn row(
        &self,
        commit: crate::history::data::CommitId,
    ) -> Option<&wire::AdmissionRowIndex> {
        self.0
            .commits
            .get(&commit.0)
            .and_then(|ordinal| self.0.rows.get(*ordinal))
    }

    pub(super) fn positions(
        &self,
        commit: crate::history::data::CommitId,
    ) -> Option<&crate::history::data::RequiredOutputPublicationPositions> {
        self.0.positions.get(&commit)
    }
}
