//! Bind actual immutable region owners to selected-image evidence.
use super::*;
use crate::durability::checkpoint_admission::CheckpointAdmittedCommit;
use worth_execution::ExecutionRequest;

impl RelationalBranchRoot {
    pub(crate) fn readmit_checkpoint_region_directory(
        issuer: &RelationalBranchRootIdentityIssuer,
        directory: worth_execution::ExecutionArray<
            super::super::super::root_checkpoint::RelationalRootRegionCheckpoint,
        >,
        commit: CheckpointAdmittedCommit,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let memory = super::super::reserve_arc_memory::<Self>(Some(request))?;
        let root_id = issuer.issue_root_id()?;
        let mut regions =
            crate::branch::root_regions::PreparedRelationalRegionSet::new(root_id, request)?;
        let (mut entity_slots, mut relation_slots) = (0usize, 0usize);
        for region in directory.iter().cloned() {
            crate::execution::check_request(request)
                .map_err(RelationalBranchRootCaptureDenial::Request)?;
            entity_slots = entity_slots
                .checked_add(region.entity_slots)
                .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
            relation_slots = relation_slots
                .checked_add(region.relation_slots)
                .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
            let partition = region.partition_id;
            let region = super::super::RelationalRootRegion::from_checkpoint_for_request(
                root_id,
                issuer.issue_region_id()?,
                region,
                request,
            )?;
            regions.insert(partition, region, issuer, request)?;
        }
        let regions = regions.finish();
        let (storage_root, content_accumulator) = axes::storage_root_for(&regions);
        let descriptor = commit.index().root.clone();
        require_descriptor_content(&descriptor, storage_root)?;
        let schema = commit.schema().clone();
        if descriptor.schema_root() != &schema.authority_digest() {
            return Err(RelationalBranchRootCaptureDenial::SchemaRootMismatch {
                descriptor_root: *descriptor.schema_root(),
                canonical_root: schema.authority_digest(),
            });
        }
        let positions = commit
            .positions()
            .cloned()
            .ok_or(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch)?;
        let axes = RelationalBranchRootAxes {
            storage_version: commit.index().commit.version_id.0,
            storage_root,
            schema_root: *descriptor.schema_root(),
            correctness_index: RelationalRootCorrectnessIndex::AuthoritativeFallback,
            visibility: commit.visibility(),
        };
        Ok(Arc::new(Self {
            id: root_id,
            managed_source_image: Default::default(),
            regions,
            entity_slot_count: entity_slots,
            relation_slot_count: relation_slots,
            content_accumulator,
            output_publication_positions: positions,
            schema_authority: schema,
            execution_contracts: Default::default(),
            committed: Some(RelationalCommittedBranchRoot {
                descriptor,
                evidence: super::super::RelationalRootCommitEvidence::Checkpoint(commit),
                axes,
                publication_cost: Default::default(),
            }),
            _memory: memory,
        }))
    }
}
