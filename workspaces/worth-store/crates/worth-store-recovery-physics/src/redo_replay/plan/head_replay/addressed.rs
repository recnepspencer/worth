//! Pre-edge C.9 path admission, followed by exact ordered-edge binding.
//! The first token is not an ordered-history edge or a selected-media seal.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PersistedPhysicalRecoveryOperation, ReleaseCustodyHeadMutationV1,
};

use super::*;
use crate::VerifiedReleasedRootEdge;

impl VerifiedSelectedReleaseHeadReplayV14 {
    /// Admit the exact semantics-admitted C.9 member before the inventory
    /// transition is built. The addressed callback must read actual source
    /// head frames; the ordered edge is bound separately after transition.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_addressed_member<Read, ReadError>(
        member: AdmittedRootStepMemberView<'_>,
        source_root: &DurablePhysicalRootManifest,
        result_root: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        maximum_effect_bytes: u64,
        remaining_additional_heap_bytes: u64,
        mut read: Read,
    ) -> Result<Self, SelectedReleaseHeadReplayDenial>
    where
        Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    {
        // The ingress observation and its `to_vec` copy can coexist for one
        // addressed frame. Check before the first callback, not after reading.
        require_two_page_window(format, remaining_additional_heap_bytes)?;
        let projection = member.materialization();
        let PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            head_effect: Some(effect),
            ..
        } = projection.operation()
        else {
            return Err(SelectedReleaseHeadReplayDenial::NotAdmittedUpsert);
        };
        let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = effect.mutation() else {
            return Err(SelectedReleaseHeadReplayDenial::NotAdmittedUpsert);
        };
        if binding.record() != next.descriptor_record()
            || binding.record_payload_sha256() != next.descriptor_frame_sha256()
            || binding.candidate_root_generation() != result_root.generation()
        {
            return Err(SelectedReleaseHeadReplayDenial::NotAdmittedUpsert);
        }
        if projection.source_root_generation() != source_root.generation()
            || source_root.generation().checked_add(1) != Some(result_root.generation())
            || source_root.tree_identity() != result_root.tree_identity()
            || result_root.release_custody_head_root() != Some(effect.result_root())
            || result_root.next_release_custody_head_block() != effect.result_next_block()
        {
            return Err(SelectedReleaseHeadReplayDenial::SourceRoot);
        }
        Self::admit_effect(
            Some(effect),
            source_root,
            format,
            maximum_effect_bytes,
            remaining_additional_heap_bytes,
            &mut read,
            Some(member.lsn_range()),
            member.operation(),
            member.group(),
            member.fate(),
            member.canonical_redo_sha256(),
        )
    }
}

impl VerifiedOrderedReleasedHeadReplayV14 {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.replay.owned_heap_bytes()
    }

    /// Consume the pre-admitted member replay only after the ordered release
    /// edge exists. No media is reread and no effect is cloned here.
    #[allow(clippy::too_many_arguments)]
    pub fn bind_edge(
        edge: &VerifiedReleasedRootEdge,
        replay: VerifiedSelectedReleaseHeadReplayV14,
        source_root: &DurablePhysicalRootManifest,
        result_root: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        remaining_additional_heap_bytes: u64,
    ) -> Result<Self, SelectedReleaseHeadReplayDenial> {
        require_two_page_window(format, remaining_additional_heap_bytes)?;
        let source_sha: [u8; 32] = Sha256::digest(source_root.encode(format)).into();
        let result_sha: [u8; 32] = Sha256::digest(result_root.encode(format)).into();
        let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = replay.effect().mutation() else {
            return Err(SelectedReleaseHeadReplayDenial::NotAdmittedUpsert);
        };
        if source_sha != edge.source_root_frame_sha256()
            || result_sha != edge.result_root_frame_sha256()
            || replay.lsn_range() != Some(edge.lsn())
            || replay.operation() != edge.operation()
            || replay.group() != edge.group()
            || replay.fate() != edge.fate()
            || replay.canonical_redo_sha256() != edge.redo_sha256()
            || source_root.generation().checked_add(1) != Some(result_root.generation())
            || source_root.tree_identity() != result_root.tree_identity()
            || replay.effect().source_root() != source_root.release_custody_head_root()
            || replay.effect().source_next_block() != source_root.next_release_custody_head_block()
            || result_root.release_custody_head_root() != Some(replay.result_root())
            || result_root.next_release_custody_head_block() != replay.result_next_block()
            || next.source_root_generation() != source_root.generation()
            || next.descriptor_record() != edge.descriptor_record()
            || next.descriptor_frame_sha256() != edge.descriptor_frame_sha256()
            || edge.candidate_root_generation() != result_root.generation()
        {
            return Err(SelectedReleaseHeadReplayDenial::NotAdmittedUpsert);
        }
        Ok(Self {
            replay,
            source_root_frame_sha256: source_sha,
            result_root_frame_sha256: result_sha,
        })
    }

    pub const fn replay(&self) -> &VerifiedSelectedReleaseHeadReplayV14 {
        &self.replay
    }
    pub const fn source_root_frame_sha256(&self) -> [u8; 32] {
        self.source_root_frame_sha256
    }
    pub const fn result_root_frame_sha256(&self) -> [u8; 32] {
        self.result_root_frame_sha256
    }
}

fn require_two_page_window(
    format: PhysicalRecordFormatDeclaration,
    remaining_additional_heap_bytes: u64,
) -> Result<(), SelectedReleaseHeadReplayDenial> {
    let required = u64::from(format.page_size().bytes())
        .checked_mul(2)
        .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
    if required > remaining_additional_heap_bytes {
        return Err(SelectedReleaseHeadReplayDenial::BoundExceeded);
    }
    Ok(())
}
