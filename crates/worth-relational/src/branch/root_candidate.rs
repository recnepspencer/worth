//! Unpublished physical images share the ordinary root-region capture owner.
use super::*;

impl RelationalBranchRoot {
    pub(crate) fn capture_applied_candidate(
        issuer: &RelationalBranchRootIdentityIssuer,
        previous: &Arc<Self>,
        delta: &RelationalPublishedPartitionDelta,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        issuer.validate_capture_capacity(delta.len())?;
        let id = issuer.issue_root_id()?;
        let (regions, _) = capture::build_incremental_regions(
            issuer,
            id,
            delta,
            previous,
            symbols,
            &previous.schema_authority,
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
            ..previous.as_ref().clone()
        }))
    }
}
