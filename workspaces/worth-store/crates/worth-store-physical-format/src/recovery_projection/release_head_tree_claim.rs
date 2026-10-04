//! One measurement of the release-head tree frames a WAL member claims.
//! A keyed upsert and a terminal head retirement carry the same path and node
//! write rosters, so every budget owner charges them through this one view.

use crate::{
    PhysicalRecordFormatDeclaration, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1,
    ReleaseCustodyHeadTransitionLimitsV1,
};

use super::{
    PersistedReleaseCustodyHeadEffectV1, PersistedTerminalReleaseHeadRetirementV1,
    PhysicalRecoveryProjectionDenial,
};

/// The head-tree frames carried by an operation, whichever mutation owns them.
#[derive(Debug, Clone, Copy)]
pub enum PersistedReleaseHeadTreeClaim<'a> {
    Upsert(&'a PersistedReleaseCustodyHeadEffectV1),
    TerminalHeadRetired(&'a PersistedTerminalReleaseHeadRetirementV1),
}

/// The head-tree claim one planned operation owns until its WAL member is
/// encoded. An operation carries at most one: an upsert or a retirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistedReleaseHeadClaim {
    Upsert(PersistedReleaseCustodyHeadEffectV1),
    TerminalHeadRetired(PersistedTerminalReleaseHeadRetirementV1),
}

impl PersistedReleaseHeadClaim {
    pub fn tree_claim(&self) -> PersistedReleaseHeadTreeClaim<'_> {
        match self {
            Self::Upsert(effect) => PersistedReleaseHeadTreeClaim::Upsert(effect),
            Self::TerminalHeadRetired(retirement) => {
                PersistedReleaseHeadTreeClaim::TerminalHeadRetired(retirement)
            }
        }
    }

    pub fn upsert(&self) -> Option<&PersistedReleaseCustodyHeadEffectV1> {
        match self {
            Self::Upsert(effect) => Some(effect),
            Self::TerminalHeadRetired(_) => None,
        }
    }

    pub fn terminal_head_retired(&self) -> Option<&PersistedTerminalReleaseHeadRetirementV1> {
        match self {
            Self::TerminalHeadRetired(retirement) => Some(retirement),
            Self::Upsert(_) => None,
        }
    }
}

impl<'a> PersistedReleaseHeadTreeClaim<'a> {
    /// The node frames the successor root publishes for this claim.
    pub fn node_writes(self) -> &'a [ReleaseCustodyHeadNodeWriteV1] {
        match self {
            Self::Upsert(effect) => effect.node_writes(),
            Self::TerminalHeadRetired(retirement) => retirement.node_writes(),
        }
    }

    pub fn mutation(self) -> ReleaseCustodyHeadMutationV1 {
        match self {
            Self::Upsert(effect) => effect.mutation(),
            Self::TerminalHeadRetired(retirement) => retirement.mutation(),
        }
    }

    /// Absent only for the first upsert into an empty tree.
    pub fn source_root(self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        match self {
            Self::Upsert(effect) => effect.source_root(),
            Self::TerminalHeadRetired(retirement) => Some(retirement.source_root()),
        }
    }

    /// Absent only when a retirement removed the last head.
    pub fn result_root(self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        match self {
            Self::Upsert(effect) => Some(effect.result_root()),
            Self::TerminalHeadRetired(retirement) => retirement.result_root(),
        }
    }

    /// The carried source path, still a WAL claim until each frame is re-read.
    pub fn source_path(self) -> &'a [ReleaseCustodyHeadPathNodeV1] {
        match self {
            Self::Upsert(effect) => effect.source_path(),
            Self::TerminalHeadRetired(retirement) => retirement.source_path(),
        }
    }

    pub fn verification_additional_peak_bytes(
        self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Option<u64> {
        match self {
            Self::Upsert(effect) => effect.verification_additional_peak_bytes(format),
            Self::TerminalHeadRetired(retirement) => {
                retirement.verification_additional_peak_bytes(format)
            }
        }
    }

    pub fn entry_count(self) -> Option<u64> {
        match self {
            Self::Upsert(effect) => effect.entry_count(),
            Self::TerminalHeadRetired(retirement) => retirement.entry_count(),
        }
    }

    pub fn framed_bytes(self) -> Option<u64> {
        match self {
            Self::Upsert(effect) => effect.framed_bytes(),
            Self::TerminalHeadRetired(retirement) => retirement.framed_bytes(),
        }
    }

    pub fn owned_heap_bytes(self) -> Option<u64> {
        match self {
            Self::Upsert(effect) => effect.owned_heap_bytes(),
            Self::TerminalHeadRetired(retirement) => retirement.owned_heap_bytes(),
        }
    }
}

pub(super) fn entry_count(
    source_path: &[ReleaseCustodyHeadPathNodeV1],
    node_writes: &[ReleaseCustodyHeadNodeWriteV1],
) -> Option<u64> {
    u64::try_from(source_path.len())
        .ok()?
        .checked_add(u64::try_from(node_writes.len()).ok()?)
}

pub(super) fn framed_bytes(
    source_path: &[ReleaseCustodyHeadPathNodeV1],
    node_writes: &[ReleaseCustodyHeadNodeWriteV1],
) -> Option<u64> {
    let paths = source_path.iter().try_fold(0_u64, |sum, node| {
        sum.checked_add(node.frame().len() as u64)
    })?;
    let writes = node_writes.iter().try_fold(0_u64, |sum, write| {
        sum.checked_add(write.frame().len() as u64)
    })?;
    paths.checked_add(writes)
}

pub(super) fn owned_heap_bytes(
    source_path: &[ReleaseCustodyHeadPathNodeV1],
    node_writes: &[ReleaseCustodyHeadNodeWriteV1],
) -> Option<u64> {
    let path = u64::try_from(source_path.len())
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<ReleaseCustodyHeadPathNodeV1>()).ok()?)?;
    let writes = u64::try_from(node_writes.len())
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<ReleaseCustodyHeadNodeWriteV1>()).ok()?)?;
    let bytes = source_path
        .iter()
        .try_fold(path.checked_add(writes)?, |bytes, node| {
            bytes.checked_add(node.owned_heap_bytes()?)
        })?;
    node_writes.iter().try_fold(bytes, |sum, write| {
        sum.checked_add(write.owned_heap_bytes()?)
    })
}

/// Exact recomputation limits for one persisted claim: the carried path plus
/// at most `new_blocks` page-bounded node writes.
pub(super) fn exact_limits(
    source_path: &[ReleaseCustodyHeadPathNodeV1],
    new_blocks: usize,
    format: PhysicalRecordFormatDeclaration,
) -> Result<ReleaseCustodyHeadTransitionLimitsV1, PhysicalRecoveryProjectionDenial> {
    ReleaseCustodyHeadTransitionLimitsV1::new(
        u16::try_from(source_path.len().max(1))
            .map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
        u16::try_from(new_blocks).map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
        source_path
            .iter()
            .try_fold(0_u64, |sum, node| {
                sum.checked_add(node.frame().len() as u64)
            })
            .and_then(|path_bytes| {
                (new_blocks as u64)
                    .checked_mul(u64::from(format.page_size().bytes()))
                    .and_then(|maximum_writes| path_bytes.checked_add(maximum_writes))
            })
            .ok_or(PhysicalRecoveryProjectionDenial::EntryLimit)?,
    )
    .ok_or(PhysicalRecoveryProjectionDenial::EntryLimit)
}
