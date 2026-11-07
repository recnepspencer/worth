//! Unpublished physical images share the ordinary root-region capture owner.
use super::*;

impl RelationalBranchRoot {
    pub(crate) fn capture_applied_candidate(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        issuer: &RelationalBranchRootIdentityIssuer,
        previous: &Arc<Self>,
        delta: &RelationalPublishedPartitionDelta,
        symbols: &crate::runtime::RuntimeSymbolTable,
        content: crate::content::ContentDirectory,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        issuer.validate_capture_capacity(delta.len())?;
        let id = issuer.issue_root_id()?;
        let (regions, _) = capture::build_incremental_regions(
            execution,
            issuer,
            id,
            delta,
            previous,
            symbols,
            &previous.schema_authority,
            None,
        )?;
        let (entity_slot_count, relation_slot_count) =
            capture_preparation::root_slot_counts(&regions, Some(previous), delta);
        let (_, content_accumulator) = axes::storage_root_for(&regions);
        // This image has no canonical envelope or branch descriptor and cannot
        // become an observation, snapshot or publication. The candidate read
        // transport exposes physical projections only; installed companion
        // images from the basis remain inaccessible through that transport.
        Ok(Arc::new(Self {
            id,
            regions,
            entity_slot_count,
            relation_slot_count,
            content_accumulator,
            committed: None,
            content,
            ..previous.as_ref().clone()
        }))
    }
}
