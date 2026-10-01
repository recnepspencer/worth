//! Exact C.9-admitted release-head replay against the selected source tree.
//! The persisted path is a WAL claim until every named frame is re-read from
//! selected media; the claimed result blocks remain WAL-chosen, not allocated
//! again during recovery.

use worth_store_physical_format::{
    DurablePhysicalRootManifest, PersistedReleaseCustodyHeadEffectV1,
    PhysicalRecordFormatDeclaration, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1,
};
use worth_store_wal::WalLsnRange;

use crate::{PhysicalRedoGroupBinding, PhysicalSourceSelection, RecoveryOperationFate};

use super::{AdmittedRootStepMemberView, ImmutablePhysicalRedoPlan, PhysicalRedoProjection};

#[path = "head_replay/addressed.rs"]
mod addressed;
#[cfg(test)]
#[path = "head_replay/budget_tests.rs"]
mod budget_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedReleaseHeadReplayDenial {
    NotAdmittedUpsert,
    SourceRoot,
    SourcePath,
    BoundExceeded,
    Read,
}

/// Borrowed proof of the exact WAL-chosen head effect. It can only borrow a
/// semantics-admitted member view, never caller-supplied projection bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSelectedReleaseHeadReplayV14 {
    effect: PersistedReleaseCustodyHeadEffectV1,
    lsn_range: Option<WalLsnRange>,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    canonical_redo_sha256: [u8; 32],
    source_frame_bytes: u64,
}

/// A historical C.9 member and an addressed source root are joined to the
/// corresponding already-verified ordered release edge before replaying it.
#[derive(Debug)]
pub struct VerifiedOrderedReleasedHeadReplayV14 {
    replay: VerifiedSelectedReleaseHeadReplayV14,
    source_root_frame_sha256: [u8; 32],
    result_root_frame_sha256: [u8; 32],
}

impl VerifiedSelectedReleaseHeadReplayV14 {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.effect.owned_heap_bytes()
    }

    /// `read` must address the selected physical artifact namespace. C.8's
    /// media ingress supplies that closure; Store later rereads independently.
    pub fn admit_selected<Read, ReadError>(
        member: AdmittedRootStepMemberView<'_>,
        selected: &PhysicalSourceSelection,
        maximum_effect_bytes: u64,
        remaining_additional_heap_bytes: u64,
        mut read: Read,
    ) -> Result<Self, SelectedReleaseHeadReplayDenial>
    where
        Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    {
        Self::admit_effect(
            member.materialization().release_head_effect(),
            selected.root().selected().manifest(),
            selected.root().selected().selector().format(),
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

    /// Later C.8 planning retains the immutable plan rather than the earlier
    /// admitted-member view. Pointer identity and the C.9-set digest bind this
    /// effect to precisely one semantics-admitted member in that plan.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_projection<Read, ReadError>(
        projection: &PhysicalRedoProjection,
        plan: &ImmutablePhysicalRedoPlan,
        selected: &PhysicalSourceSelection,
        maximum_effect_bytes: u64,
        remaining_additional_heap_bytes: u64,
        mut read: Read,
    ) -> Result<Self, SelectedReleaseHeadReplayDenial>
    where
        Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    {
        let digest = projection
            .semantics_admitted_redo_sha256()
            .filter(|digest| plan.admits_exact_member_redo_digest(projection, *digest))
            .ok_or(SelectedReleaseHeadReplayDenial::NotAdmittedUpsert)?;
        Self::admit_effect(
            projection.materialization().release_head_effect(),
            selected.root().selected().manifest(),
            selected.root().selected().selector().format(),
            maximum_effect_bytes,
            remaining_additional_heap_bytes,
            &mut read,
            None,
            projection.operation(),
            projection.group(),
            projection.fate(),
            digest,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn admit_effect<Read, ReadError>(
        effect: Option<&PersistedReleaseCustodyHeadEffectV1>,
        source: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        maximum_effect_bytes: u64,
        remaining_additional_heap_bytes: u64,
        read: &mut Read,
        lsn_range: Option<WalLsnRange>,
        operation: [u8; 32],
        group: PhysicalRedoGroupBinding,
        fate: RecoveryOperationFate,
        canonical_redo_sha256: [u8; 32],
    ) -> Result<Self, SelectedReleaseHeadReplayDenial>
    where
        Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    {
        let effect = effect.ok_or(SelectedReleaseHeadReplayDenial::NotAdmittedUpsert)?;
        let next = match effect.mutation() {
            worth_store_physical_format::ReleaseCustodyHeadMutationV1::Upsert { next, .. } => next,
            _ => return Err(SelectedReleaseHeadReplayDenial::NotAdmittedUpsert),
        };
        if source.generation() != next.source_root_generation()
            || source.tree_identity() != effect.tree_identity()
            || source.release_custody_head_root() != effect.source_root()
            || source.next_release_custody_head_block() != effect.source_next_block()
        {
            return Err(SelectedReleaseHeadReplayDenial::SourceRoot);
        }
        let total = effect
            .framed_bytes()
            .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
        if total > maximum_effect_bytes {
            return Err(SelectedReleaseHeadReplayDenial::BoundExceeded);
        }
        let verification_peak = effect
            .verification_additional_peak_bytes(format)
            .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
        let clone_bytes = effect
            .owned_heap_bytes()
            .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
        let read_buffer = u64::from(format.page_size().bytes());
        if verification_peak.max(read_buffer).max(clone_bytes) > remaining_additional_heap_bytes {
            return Err(SelectedReleaseHeadReplayDenial::BoundExceeded);
        }
        effect
            .verify_exact(source.generation(), format)
            .map_err(|_| SelectedReleaseHeadReplayDenial::NotAdmittedUpsert)?;
        let mut read_bytes = 0_u64;
        for node in effect.source_path() {
            let frame = read(node.reference(), u64::from(format.page_size().bytes()))
                .map_err(|_| SelectedReleaseHeadReplayDenial::Read)?;
            read_bytes = read_bytes
                .checked_add(
                    u64::try_from(frame.len())
                        .map_err(|_| SelectedReleaseHeadReplayDenial::BoundExceeded)?,
                )
                .ok_or(SelectedReleaseHeadReplayDenial::BoundExceeded)?;
            if read_bytes > maximum_effect_bytes || frame != node.frame() {
                return Err(SelectedReleaseHeadReplayDenial::SourcePath);
            }
        }
        Ok(Self {
            effect: effect.clone(),
            lsn_range,
            operation,
            group,
            fate,
            canonical_redo_sha256,
            source_frame_bytes: read_bytes,
        })
    }

    pub const fn effect(&self) -> &PersistedReleaseCustodyHeadEffectV1 {
        &self.effect
    }
    pub const fn lsn_range(&self) -> Option<WalLsnRange> {
        self.lsn_range
    }
    pub const fn operation(&self) -> [u8; 32] {
        self.operation
    }
    pub const fn group(&self) -> PhysicalRedoGroupBinding {
        self.group
    }
    pub const fn fate(&self) -> RecoveryOperationFate {
        self.fate
    }
    pub const fn canonical_redo_sha256(&self) -> [u8; 32] {
        self.canonical_redo_sha256
    }
    pub const fn source_frame_bytes(&self) -> u64 {
        self.source_frame_bytes
    }
    pub fn source_path(&self) -> &[ReleaseCustodyHeadPathNodeV1] {
        self.effect.source_path()
    }
    pub const fn result_root(&self) -> ReleaseCustodyHeadBlockReferenceV1 {
        self.effect.result_root()
    }
    pub const fn result_next_block(&self) -> u64 {
        self.effect.result_next_block()
    }
    pub fn node_writes(&self) -> &[ReleaseCustodyHeadNodeWriteV1] {
        self.effect.node_writes()
    }
}
