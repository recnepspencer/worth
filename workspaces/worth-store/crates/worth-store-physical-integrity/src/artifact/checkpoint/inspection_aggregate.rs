use super::{footer_basis::sequence_mismatch, footer_basis::CheckpointFooterExpectedBindings};
use crate::{
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointDirtyBasis, IntegrityValidatedCheckpointStreamHeader,
    PhysicalArtifactScope, PhysicalIntegrityRejection,
};
use worth_store_physical_format::{CheckpointSelectiveRecordAggregate, PhysicalCheckpointSource};

/// Constant-space selective interpretation of validated records. This value
/// carries neither source admission nor lifecycle/repair/recovery authority.
pub(crate) struct CheckpointInspectionAggregate {
    source: PhysicalCheckpointSource,
    header_offset: u64,
    next_offset: u64,
    dirty: CheckpointSelectiveRecordAggregate,
    bindings: CheckpointSelectiveRecordAggregate,
    compaction: Option<(u64, u64, u64)>,
}

impl CheckpointInspectionAggregate {
    pub(super) fn accepts_scope(&self, scope: PhysicalArtifactScope) -> bool {
        scope.checkpoint_identity() == Some(self.source.identity())
            && scope.byte_range().offset() == self.next_offset
    }
    pub(super) fn accepts_footer(
        &self,
        scope: PhysicalArtifactScope,
        certificate_bytes: usize,
    ) -> bool {
        scope.checkpoint_identity() == Some(self.source.identity())
            && self.next_offset.checked_add(certificate_bytes as u64)
                == Some(scope.byte_range().offset())
    }
    pub(super) fn new(header: &IntegrityValidatedCheckpointStreamHeader<'_>) -> Self {
        Self {
            source: header.source(),
            header_offset: header.scope().byte_range().offset(),
            next_offset: header.scope().byte_range().end_exclusive(),
            dirty: CheckpointSelectiveRecordAggregate::new(),
            bindings: CheckpointSelectiveRecordAggregate::new(),
            compaction: None,
        }
    }
    pub(super) fn dirty(
        mut self,
        record: &IntegrityValidatedCheckpointDirtyBasis<'_>,
    ) -> Result<Self, PhysicalIntegrityRejection> {
        self.require_next(record.scope())?;
        if self.compaction.is_some() {
            return Err(sequence_mismatch(record.scope()));
        }
        self.dirty
            .include(record.inspected_bytes())
            .map_err(|_| sequence_mismatch(record.scope()))?;
        Ok(self)
    }
    pub(super) fn compaction(
        mut self,
        record: &IntegrityValidatedCheckpointBindingCompaction<'_>,
    ) -> Result<Self, PhysicalIntegrityRejection> {
        self.require_next(record.scope())?;
        if self.compaction.is_some() {
            return Err(sequence_mismatch(record.scope()));
        }
        self.compaction = Some((
            record.scope().byte_range().offset() - self.header_offset,
            record.generation(),
            record.wal_cutoff_lsn_exclusive(),
        ));
        Ok(self)
    }
    pub(super) fn binding(
        mut self,
        record: &IntegrityValidatedCheckpointBinding<'_>,
    ) -> Result<Self, PhysicalIntegrityRejection> {
        self.require_next(record.scope())?;
        if self.compaction.is_none() {
            return Err(sequence_mismatch(record.scope()));
        }
        self.bindings
            .include(record.inspected_bytes())
            .map_err(|_| sequence_mismatch(record.scope()))?;
        Ok(self)
    }
    fn require_next(
        &mut self,
        scope: PhysicalArtifactScope,
    ) -> Result<(), PhysicalIntegrityRejection> {
        if scope.checkpoint_identity() != Some(self.source.identity())
            || scope.byte_range().offset() != self.next_offset
        {
            return Err(missing_context(scope));
        }
        self.next_offset = scope.byte_range().end_exclusive();
        Ok(())
    }
    pub(super) fn expected(
        self,
        scope: PhysicalArtifactScope,
        certificates: &[u8],
    ) -> Result<CheckpointFooterExpectedBindings, PhysicalIntegrityRejection> {
        let certificates = super::certificate_summary::from_prefix(
            self.source,
            scope,
            self.next_offset,
            certificates,
        )?;
        let (compaction_offset, compaction_generation, wal_cutoff_lsn_exclusive) =
            self.compaction.ok_or_else(|| sequence_mismatch(scope))?;
        Ok(CheckpointFooterExpectedBindings {
            dirty: self.dirty.summary(),
            bindings: self.bindings.summary(),
            certificates,
            compaction_offset,
            compaction_generation,
            wal_cutoff_lsn_exclusive,
        })
    }
}

fn missing_context(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    PhysicalIntegrityRejection::Unknown(crate::UnknownPhysicalIntegrityPosture::new(
        scope,
        crate::UnknownPhysicalIntegrityCause::ExpectedScopeUnavailable,
    ))
}
