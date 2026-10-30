//! Exact retained identity observations during generated materialization custody.
use super::*;
use crate::identity::data::{EntityId, KindId};

impl RelationalMaterializationPort {
    /// Observe the live kind of an entity retained outside this exact suspension.
    /// Requires the owner-admitted suspension basis and sealed custody. This does
    /// not expose payload, authorize ordinary reads, or admit rematerialization
    /// of the retained entity.
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn retained_entity_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        custody: &RelationalMaterializationCustody,
        entity: EntityId,
    ) -> Result<KindId, RelationalMaterializationError> {
        let runtime = self
            .owner
            .admitted_runtime()
            .ok_or(RelationalMaterializationError::OwnerUnavailable)?;
        if basis.descriptor().runtime_instance_id() != runtime.runtime_instance_id() {
            return Err(RelationalMaterializationError::BranchMismatch);
        }
        validate_custody_basis(basis, custody)?;
        // Custody is canonical owner-issued record order: entities precede
        // relations, ordered by entity identity. This observation stays indexed.
        if custody
            .records()
            .binary_search_by(|record| match record {
                RelationalMaterializationRecord::Entity { entity_id, .. } => entity_id.cmp(&entity),
                RelationalMaterializationRecord::Relation { .. } => std::cmp::Ordering::Greater,
            })
            .is_ok()
        {
            return Err(RelationalMaterializationError::CandidateManifestMismatch);
        }
        basis
            .inner
            .root
            .demand_regions_head(execution, &runtime.services.symbols, [entity.partition_id])
            .map_err(|failure| RelationalMaterializationError::Read(failure.into()))?;
        let partition = basis
            .inner
            .root
            .accepted_head_region_view(entity.partition_id)
            .ok_or_else(|| missing_entity(entity))?;
        partition
            .entity_arena()
            .get(&entity)
            .filter(|row| row.is_live())
            .and_then(|row| row.kind_id())
            .ok_or_else(|| missing_entity(entity))
    }
}
