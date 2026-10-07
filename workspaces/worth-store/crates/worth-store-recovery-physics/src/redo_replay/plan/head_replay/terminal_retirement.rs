//! Exact C.9-admitted terminal head retirement replay against the selected
//! source tree. The retired entry must be the exact terminal head that tree
//! holds: every carried path frame is re-read from selected media, and the
//! recomputed result is that tree minus exactly that entry.

use worth_store_physical_format::{
    DurablePhysicalRootManifest, PersistedPhysicalRecoveryOperation, PersistedReleaseHeadTreeClaim,
    PersistedTerminalReleaseHeadRetirementV1, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1,
};
use worth_store_wal::WalLsnRange;

use super::selected_path::{require_claim_bounds, reread_source_path};
use super::{AdmittedRootStepMemberView, SelectedReleaseHeadReplayDenial};
use crate::{PhysicalRedoGroupBinding, PhysicalSourceSelection, RecoveryOperationFate};

#[cfg(test)]
#[path = "terminal_retirement_tests.rs"]
mod tests;

/// Proof that one C.9-admitted record-less member retires exactly one
/// terminal head of the selected tree. It borrows a semantics-admitted member
/// view, never caller-supplied projection bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSelectedTerminalHeadRetirementReplay {
    retirement: PersistedTerminalReleaseHeadRetirementV1,
    lsn_range: WalLsnRange,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    canonical_redo_sha256: [u8; 32],
    source_frame_bytes: u64,
}

impl VerifiedSelectedTerminalHeadRetirementReplay {
    /// `read` must address the selected physical artifact namespace.
    pub fn admit_selected<Read, ReadError>(
        member: AdmittedRootStepMemberView<'_>,
        selected: &PhysicalSourceSelection,
        maximum_effect_bytes: u64,
        remaining_additional_heap_bytes: u64,
        read: Read,
    ) -> Result<Self, SelectedReleaseHeadReplayDenial>
    where
        Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    {
        Self::admit_member(
            member,
            selected.root().selected().manifest(),
            selected.root().selected().selector().format(),
            maximum_effect_bytes,
            remaining_additional_heap_bytes,
            read,
        )
    }

    fn admit_member<Read, ReadError>(
        member: AdmittedRootStepMemberView<'_>,
        source: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        maximum_effect_bytes: u64,
        remaining_additional_heap_bytes: u64,
        mut read: Read,
    ) -> Result<Self, SelectedReleaseHeadReplayDenial>
    where
        Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    {
        let PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(retirement) =
            member.materialization().operation()
        else {
            return Err(SelectedReleaseHeadReplayDenial::NotAdmittedTerminalHeadRetirement);
        };
        if source.generation() != retirement.source_root_generation()
            || source.tree_identity() != retirement.tree_identity()
            || source.release_custody_head_root() != Some(retirement.source_root())
            || source.next_release_custody_head_block() != retirement.source_next_block()
        {
            return Err(SelectedReleaseHeadReplayDenial::SourceRoot);
        }
        let claim = PersistedReleaseHeadTreeClaim::TerminalHeadRetired(retirement);
        require_claim_bounds(
            claim,
            format,
            maximum_effect_bytes,
            remaining_additional_heap_bytes,
        )?;
        retirement
            .verify_exact(format)
            .map_err(|_| SelectedReleaseHeadReplayDenial::NotAdmittedTerminalHeadRetirement)?;
        let source_frame_bytes = reread_source_path(claim, format, &mut read)?;
        Ok(Self {
            retirement: retirement.clone(),
            lsn_range: member.lsn_range(),
            operation: member.operation(),
            group: member.group(),
            fate: member.fate(),
            canonical_redo_sha256: member.canonical_redo_sha256(),
            source_frame_bytes,
        })
    }

    /// The retired entry, the selected source tree it left, and the exact
    /// WAL-chosen result tree.
    pub const fn retirement(&self) -> &PersistedTerminalReleaseHeadRetirementV1 {
        &self.retirement
    }
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.retirement.owned_heap_bytes()
    }
    pub const fn lsn_range(&self) -> WalLsnRange {
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
}
