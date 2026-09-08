use worth_store_physical_format::{
    CheckpointSelectiveRecordAggregate, CheckpointSelectiveRecordSummary,
    PhysicalCheckpointIdentity,
};

use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause};
use crate::validation::{
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointDirtyBasis, IntegrityValidatedCheckpointStreamHeader,
    PhysicalArtifactScope, PhysicalIntegrityRejection,
};

use super::record_rejection::damaged;

pub struct CheckpointFooterValidationBasis<'records, 'media> {
    header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
    dirty_basis: CheckpointDirtyBasisEvidence<'records, 'media>,
    binding_compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
    bindings: CheckpointBindingEvidence<'records, 'media>,
}

enum CheckpointDirtyBasisEvidence<'records, 'media> {
    Values(&'records [IntegrityValidatedCheckpointDirtyBasis<'media>]),
    References(&'records [&'records IntegrityValidatedCheckpointDirtyBasis<'media>]),
}

enum CheckpointBindingEvidence<'records, 'media> {
    Values(&'records [IntegrityValidatedCheckpointBinding<'media>]),
    References(&'records [&'records IntegrityValidatedCheckpointBinding<'media>]),
}

pub(super) struct CheckpointFooterExpectedBindings {
    pub(super) dirty: CheckpointSelectiveRecordSummary,
    pub(super) compaction_offset: u64,
    pub(super) compaction_generation: u64,
    pub(super) wal_cutoff_lsn_exclusive: u64,
    pub(super) bindings: CheckpointSelectiveRecordSummary,
}

impl<'records, 'media> CheckpointFooterValidationBasis<'records, 'media> {
    pub const fn new(
        header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
        dirty_basis: &'records [IntegrityValidatedCheckpointDirtyBasis<'media>],
        binding_compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
        bindings: &'records [IntegrityValidatedCheckpointBinding<'media>],
    ) -> Self {
        Self {
            header,
            dirty_basis: CheckpointDirtyBasisEvidence::Values(dirty_basis),
            binding_compaction,
            bindings: CheckpointBindingEvidence::Values(bindings),
        }
    }

    pub fn from_record_references(
        header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
        dirty_basis: &'records [&'records IntegrityValidatedCheckpointDirtyBasis<'media>],
        binding_compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
        bindings: &'records [&'records IntegrityValidatedCheckpointBinding<'media>],
    ) -> Self {
        Self {
            header,
            dirty_basis: CheckpointDirtyBasisEvidence::References(dirty_basis),
            binding_compaction,
            bindings: CheckpointBindingEvidence::References(bindings),
        }
    }

    pub(super) fn expected_bindings(
        self,
        footer_scope: PhysicalArtifactScope,
    ) -> Result<CheckpointFooterExpectedBindings, PhysicalIntegrityRejection> {
        let identity = footer_scope
            .checkpoint_identity()
            .expect("checkpoint-footer scope carries admitted identity");
        if self.header.checkpoint_identity() != identity {
            return Err(identity_mismatch(self.header.scope()));
        }
        let header_range = self.header.scope().byte_range();
        let mut next_offset = header_range.end_exclusive();
        let mut dirty = CheckpointSelectiveRecordAggregate::new();
        match self.dirty_basis {
            CheckpointDirtyBasisEvidence::Values(records) => {
                for record in records {
                    next_offset = include_dirty(record, identity, next_offset, &mut dirty)?;
                }
            }
            CheckpointDirtyBasisEvidence::References(records) => {
                for record in records {
                    next_offset = include_dirty(record, identity, next_offset, &mut dirty)?;
                }
            }
        }
        let compaction_scope = self.binding_compaction.scope();
        next_offset = validate_record_scope(compaction_scope, identity, next_offset)?;
        let compaction_offset = compaction_scope
            .byte_range()
            .offset()
            .checked_sub(header_range.offset())
            .expect("ordered checkpoint records cannot precede their header");
        let mut bindings = CheckpointSelectiveRecordAggregate::new();
        match self.bindings {
            CheckpointBindingEvidence::Values(records) => {
                for record in records {
                    next_offset = include_binding(record, identity, next_offset, &mut bindings)?;
                }
            }
            CheckpointBindingEvidence::References(records) => {
                for record in records {
                    next_offset = include_binding(record, identity, next_offset, &mut bindings)?;
                }
            }
        }
        if footer_scope.byte_range().offset() != next_offset {
            return Err(sequence_mismatch(footer_scope));
        }
        Ok(CheckpointFooterExpectedBindings {
            dirty: dirty.summary(),
            compaction_offset,
            compaction_generation: self.binding_compaction.generation(),
            wal_cutoff_lsn_exclusive: self.binding_compaction.wal_cutoff_lsn_exclusive(),
            bindings: bindings.summary(),
        })
    }
}

fn include_dirty(
    record: &IntegrityValidatedCheckpointDirtyBasis<'_>,
    identity: PhysicalCheckpointIdentity,
    next_offset: u64,
    aggregate: &mut CheckpointSelectiveRecordAggregate,
) -> Result<u64, PhysicalIntegrityRejection> {
    let next_offset = validate_record_scope(record.scope(), identity, next_offset)?;
    include_record(aggregate, record.scope(), record.inspected_bytes())?;
    Ok(next_offset)
}

fn include_binding(
    record: &IntegrityValidatedCheckpointBinding<'_>,
    identity: PhysicalCheckpointIdentity,
    next_offset: u64,
    aggregate: &mut CheckpointSelectiveRecordAggregate,
) -> Result<u64, PhysicalIntegrityRejection> {
    let next_offset = validate_record_scope(record.scope(), identity, next_offset)?;
    include_record(aggregate, record.scope(), record.inspected_bytes())?;
    Ok(next_offset)
}

fn validate_record_scope(
    scope: PhysicalArtifactScope,
    identity: PhysicalCheckpointIdentity,
    expected_offset: u64,
) -> Result<u64, PhysicalIntegrityRejection> {
    if scope.checkpoint_identity() != Some(identity) {
        return Err(identity_mismatch(scope));
    }
    if scope.byte_range().offset() != expected_offset {
        return Err(sequence_mismatch(scope));
    }
    Ok(scope.byte_range().end_exclusive())
}

fn include_record(
    aggregate: &mut CheckpointSelectiveRecordAggregate,
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> Result<(), PhysicalIntegrityRejection> {
    aggregate
        .include(bytes)
        .map_err(|_| sequence_mismatch(scope))
}

fn identity_mismatch(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    damaged(
        scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

pub(super) fn sequence_mismatch(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    damaged(
        scope,
        PhysicalDamageCause::SequenceMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    )
}
