//! An additive Complete source artifact binds unchanged CHK18 to original roles.
use super::{RelationalInitialInputsRefusal as Refusal, RelationalSourceInitialInputsArtifact};
use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::data::RelationalNativeCheckpoint;
use crate::publication::patch::data::PatchStreamPosition;
use sha2::{Digest, Sha256};
use std::ops::Range;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion};

const MAGIC: &[u8; 8] = b"WRTHRCS1";
const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(2);
const HEADER: usize = 84;

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
        let mut header = [0u8; HEADER];
        header[..8].copy_from_slice(MAGIC);
        header[8..12].copy_from_slice(&VERSION.get().to_le_bytes());
        for (offset, value) in [
            (12, covered.0),
            (20, through.0),
            (28, cp.len() as u64),
            (36, init.len() as u64),
            (44, original.len() as u64),
        ] {
            header[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        }
        let mut digest = Sha256::new();
        hash(&mut digest, &header[..52], request)?;
        for bytes in [cp, init, original] {
            hash(&mut digest, bytes, request)?;
        }
        header[52..].copy_from_slice(&digest.finalize());
        let length = HEADER
            .checked_add(cp.len())
            .and_then(|n| n.checked_add(init.len()))
            .and_then(|n| n.checked_add(original.len()))
            .ok_or(Refusal::Malformed)?;
        let mut buffer = ExecutionRequestArrayBuilder::allocate(*request, length).map_err(array)?;
        for bytes in [header.as_slice(), cp, init, original] {
            buffer.extend_from_slice(bytes).map_err(array)?;
        }
        let bytes = buffer.seal_immutable_bytes().map_err(array)?;
        Self::from_untrusted_region(bytes, 0..length, request)
    }

    /// Framing eligibility is descriptive only. Recovery admits the whole graph.
    pub fn from_untrusted_region(
        bytes: ExecutionImmutableBytes,
        region: Range<usize>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        request
            .consult()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let raw = bytes.get(region.clone()).ok_or(Refusal::Malformed)?;
        if raw.get(..8) != Some(MAGIC.as_slice()) {
            return Err(Refusal::Malformed);
        }
        let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(
            raw.get(8..12)
                .ok_or(Refusal::Malformed)?
                .try_into()
                .map_err(|_| Refusal::Malformed)?,
        ))
        .map_err(|_| Refusal::Malformed)?;
        BoundaryProtocolCompatibilityWindow::inclusive(VERSION, VERSION)
            .admit(version)
            .map_err(Refusal::UnsupportedVersion)?;
        let integer = |offset| -> Result<u64, Refusal> {
            Ok(u64::from_le_bytes(
                raw.get(offset..offset + 8)
                    .ok_or(Refusal::Malformed)?
                    .try_into()
                    .map_err(|_| Refusal::Malformed)?,
            ))
        };
        let covered = PatchStreamPosition(integer(12)?);
        let through = RelationalBranchTransitionSequence(integer(20)?);
        let cp_end = HEADER
            .checked_add(usize::try_from(integer(28)?).map_err(|_| Refusal::Malformed)?)
            .ok_or(Refusal::Malformed)?;
        let init_end = cp_end
            .checked_add(usize::try_from(integer(36)?).map_err(|_| Refusal::Malformed)?)
            .ok_or(Refusal::Malformed)?;
        let end = init_end
            .checked_add(usize::try_from(integer(44)?).map_err(|_| Refusal::Malformed)?)
            .ok_or(Refusal::Malformed)?;
        if raw.len() != end {
            return Err(Refusal::Malformed);
        }
        let mut digest = Sha256::new();
        hash(&mut digest, &raw[..52], request)?;
        hash(&mut digest, &raw[HEADER..], request)?;
        if raw[52..HEADER] != digest.finalize()[..] {
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

fn hash(
    digest: &mut Sha256,
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    for chunk in bytes.chunks(64 * 1024) {
        request
            .consult()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        digest.update(chunk);
    }
    Ok(())
}
fn array(cause: worth_execution::ExecutionRequestArrayDenial) -> Refusal {
    Refusal::Native(
        crate::durability::checkpoint_history::capture::array_capture_stop(cause).into(),
    )
}
