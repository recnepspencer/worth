//! An additive Complete source artifact binds unchanged CHK18 to original roles.
use super::frame::{backing_refusal, frame_refusal};
use super::{RelationalInitialInputsRefusal as Refusal, RelationalSourceInitialInputsArtifact};
use crate::durability::data::RelationalNativeCheckpoint;
use contracts::branch::transition::sequence::RelationalBranchTransitionSequence;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use std::ops::Range;
use worth_execution::{
    decode_framed_envelope, encode_framed_bytes, ExecutionImmutableBytes, ExecutionRequest,
    FramedEnvelopeSpec, FRAMED_ENVELOPE_HEADER_BYTES,
};
use worth_foundational::facade::BoundaryProtocolVersion;
use worth_relational_contracts::internal_port as contracts;

const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(3);
const METADATA: usize = 5 * 8;
const HEADER: usize = FRAMED_ENVELOPE_HEADER_BYTES + METADATA;
const SPEC: FramedEnvelopeSpec = FramedEnvelopeSpec {
    magic: *b"WRTHRCS1",
    supported_versions: &[VERSION.get()],
    max_payload_bytes: (isize::MAX as usize - FRAMED_ENVELOPE_HEADER_BYTES) as u64,
};

#[derive(Debug, Clone)]
pub struct RelationalCompleteSourceArtifact {
    bytes: ExecutionImmutableBytes,
    region: Range<usize>,
    checkpoint: Range<usize>,
    initial: RelationalSourceInitialInputsArtifact,
    normalized: Range<usize>,
    covered: PatchStreamPosition,
    through: RelationalBranchTransitionSequence,
}

impl RelationalCompleteSourceArtifact {
    pub const PROTOCOL: worth_foundational::facade::BoundaryProtocolIdentity =
        worth_foundational::facade::BoundaryProtocolIdentity::new(
            "worth.relational.complete-source",
        );
    pub(crate) fn emit(
        checkpoint: &RelationalNativeCheckpoint,
        initial: &RelationalSourceInitialInputsArtifact,
        normalized: &ExecutionImmutableBytes,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let covered = checkpoint.covered_position().ok_or(Refusal::Malformed)?;
        let through = checkpoint
            .covered_branch_transitions()
            .ok_or(Refusal::Malformed)?;
        let cp = checkpoint.bytes();
        let init = initial.as_bytes();
        let original = normalized.bytes();
        let metadata = [
            covered.0,
            through.0,
            cp.len() as u64,
            init.len() as u64,
            original.len() as u64,
        ]
        .map(u64::to_le_bytes);
        let bytes = encode_framed_bytes(
            SPEC,
            VERSION.get(),
            request,
            |sink| {
                sink(metadata.as_flattened())?;
                sink(cp)?;
                sink(init)?;
                sink(original)
            },
            backing_refusal,
        )?;
        let length = bytes.len();
        Self::from_untrusted_region(bytes, 0..length, request)
    }

    /// Framing eligibility is descriptive only. Recovery admits the whole graph.
    pub fn from_untrusted_region(
        bytes: ExecutionImmutableBytes,
        region: Range<usize>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let raw = bytes.get(region.clone()).ok_or(Refusal::Malformed)?;
        let (_, payload) = decode_framed_envelope(SPEC, raw, request)
            .map_err(|error| frame_refusal(error, VERSION))?;
        let metadata = payload.get(..METADATA).ok_or(Refusal::Malformed)?;
        let integer = |offset| u64::from_le_bytes(metadata[offset..offset + 8].try_into().unwrap());
        let covered = PatchStreamPosition(integer(0));
        let through = RelationalBranchTransitionSequence(integer(8));
        let cp_end = HEADER
            .checked_add(usize::try_from(integer(16)).map_err(|_| Refusal::Malformed)?)
            .ok_or(Refusal::Malformed)?;
        let init_end = cp_end
            .checked_add(usize::try_from(integer(24)).map_err(|_| Refusal::Malformed)?)
            .ok_or(Refusal::Malformed)?;
        let end = init_end
            .checked_add(usize::try_from(integer(32)).map_err(|_| Refusal::Malformed)?)
            .ok_or(Refusal::Malformed)?;
        if raw.len() != end {
            return Err(Refusal::Malformed);
        }
        let base = region.start;
        let initial = RelationalSourceInitialInputsArtifact::from_region(
            bytes.clone(),
            base + cp_end..base + init_end,
            request,
        )?;
        Ok(Self {
            bytes,
            region,
            covered,
            through,
            checkpoint: base + HEADER..base + cp_end,
            initial,
            normalized: base + init_end..base + end,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[self.region.clone()]
    }
    pub fn covered(&self) -> PatchStreamPosition {
        self.covered
    }
    pub fn covered_transitions(&self) -> RelationalBranchTransitionSequence {
        self.through
    }
    pub(crate) fn checkpoint(&self) -> Result<RelationalNativeCheckpoint, Refusal> {
        RelationalNativeCheckpoint::from_untrusted_bytes_region(
            self.bytes.clone(),
            self.checkpoint.clone(),
        )
        .map_err(|_| Refusal::Malformed)
    }
    pub fn initial_coordinate(&self) -> (u64, [u8; 32]) {
        (self.initial.revision(), self.initial.payload_digest())
    }
    pub(crate) fn initial_artifact(
        &self,
    ) -> Result<RelationalSourceInitialInputsArtifact, Refusal> {
        Ok(self.initial.clone())
    }
    pub(crate) fn normalized_bytes(&self) -> &[u8] {
        &self.bytes[self.normalized.clone()]
    }
}
