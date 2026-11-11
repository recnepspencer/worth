//! Only the fixed metadata allocation is funded here; Extra retains its own bill.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionRequest;
impl VersionedEntityMetadata {
    /// Ordinary image/maintenance construction carries no controlled allocation claim.
    pub(crate) fn from_ordinary(
        effective_at: VersionId,
        retired_at: Option<VersionId>,
        generation: u32,
        kind_id: KindId,
        lineage_id: Option<LineageId>,
        extra: StorageAllocation<EntityExtra>,
    ) -> Self {
        Self {
            effective_at,
            retired_at,
            generation,
            kind_id,
            lineage_id,
            extra,
            _funding: None,
        }
    }
    pub(crate) fn fund_for_request(
        mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        // This value is still inline. Reserve BEFORE its actual StorageAllocation is made.
        self._funding =
            request.reserve_retained_memory(StorageAllocation::<Self>::layout_bytes())?;
        Ok(self)
    }
}
impl Clone for VersionedEntityMetadata {
    fn clone(&self) -> Self {
        // The copied fields are fixed data and a shared Extra handle, with no new heap.
        Self::from_ordinary(
            self.effective_at,
            self.retired_at,
            self.generation,
            self.kind_id,
            self.lineage_id,
            self.extra.clone(),
        )
    }
}
impl std::fmt::Debug for VersionedEntityMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VersionedEntityMetadata")
            .field("effective_at", &self.effective_at)
            .field("retired_at", &self.retired_at)
            .field("generation", &self.generation)
            .field("kind_id", &self.kind_id)
            .field("lineage_id", &self.lineage_id)
            .field("extra", &self.extra)
            .finish()
    }
}
impl VersionedRelationMetadata {
    /// Ordinary image/maintenance construction carries no controlled allocation claim.
    pub(crate) fn from_ordinary(
        effective_at: VersionId,
        retired_at: Option<VersionId>,
        generation: u32,
        kind_id: KindId,
        endpoints: RelationEndpoints,
        extra: StorageAllocation<RelationExtra>,
    ) -> Self {
        Self {
            effective_at,
            retired_at,
            generation,
            kind_id,
            endpoints,
            extra,
            _funding: None,
        }
    }
    pub(crate) fn fund_for_request(
        mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        // This value is still inline. Reserve BEFORE its actual StorageAllocation is made.
        self._funding =
            request.reserve_retained_memory(StorageAllocation::<Self>::layout_bytes())?;
        Ok(self)
    }
}
impl Clone for VersionedRelationMetadata {
    fn clone(&self) -> Self {
        // The copied fields are fixed data and a shared Extra handle, with no new heap.
        Self::from_ordinary(
            self.effective_at,
            self.retired_at,
            self.generation,
            self.kind_id,
            self.endpoints.clone(),
            self.extra.clone(),
        )
    }
}
impl std::fmt::Debug for VersionedRelationMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VersionedRelationMetadata")
            .field("effective_at", &self.effective_at)
            .field("retired_at", &self.retired_at)
            .field("generation", &self.generation)
            .field("kind_id", &self.kind_id)
            .field("endpoints", &self.endpoints)
            .field("extra", &self.extra)
            .finish()
    }
}
