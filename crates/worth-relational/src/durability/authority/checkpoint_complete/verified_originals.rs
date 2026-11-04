//! Own the verified prefix, fold and protected read through image capture.
use super::{
    roots::CapturedRoots,
    source::{FrozenOriginalInputs, OriginalCanonicalCommit},
};
use crate::durability::{
    data::RelationalNativeCheckpoint, source_initial_inputs::RelationalCompleteSourceArtifact,
};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest,
};
use worth_proof::{AuthorityMarker, NativeCheckpointOriginalsVerified};

pub(super) struct VerifiedOriginalCustody {
    pub(super) manifest: crate::durability::data::RelationalCheckpointManifest,
    pub(super) checkpoint: RelationalNativeCheckpoint,
    pub(super) source_artifact: RelationalCompleteSourceArtifact,
    pub(super) commits: ExecutionArray<OriginalCanonicalCommit>,
    pub(super) roots: CapturedRoots,
    pub(super) original: FrozenOriginalInputs,
    // The same actual input owners above prevent address reuse for this local
    // schema-identity lookup. No address is written into the wire image.
    pub(super) carriers: ExecutionOrderedMap<usize, [u8; 32]>,
}

/// Private concrete payload. Descriptive bytes cannot construct this phase.
/// Its thin owner avoids carrying the entire prefix through phase return frames.
pub(super) struct VerifiedCheckpointOriginals {
    custody: Box<VerifiedOriginalCustody>,
    // The Box and its original inputs are destroyed before their allocation bill.
    _memory: Option<ExecutionMemoryReservation>,
}
impl AuthorityMarker for VerifiedCheckpointOriginals {}

impl VerifiedCheckpointOriginals {
    pub(super) fn from_verified(
        custody: VerifiedOriginalCustody,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<NativeCheckpointOriginalsVerified<Self>, super::Denial> {
        let memory = request
            .reserve_retained_memory(size_of::<VerifiedOriginalCustody>() as u64)
            .map_err(|cause| super::Denial::Request(cause.into()))?;
        Ok(NativeCheckpointOriginalsVerified::from_owner(Self {
            custody: Box::new(custody),
            _memory: memory,
        }))
    }

    pub(super) fn custody(&self) -> &VerifiedOriginalCustody {
        &self.custody
    }

    pub(super) fn into_parts(
        self,
    ) -> (RelationalNativeCheckpoint, RelationalCompleteSourceArtifact) {
        // Named locals drop in reverse order. Keep the bill until after the
        // moved-out Box's allocation has been deallocated, too.
        let _memory = self._memory;
        let custody = self.custody;
        let VerifiedOriginalCustody {
            checkpoint,
            source_artifact,
            commits,
            roots,
            original,
            carriers,
            manifest,
        } = *custody;
        drop((commits, roots, original, carriers, manifest));
        (checkpoint, source_artifact)
    }
}
