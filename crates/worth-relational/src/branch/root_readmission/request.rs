//! Reconstruct root DATA under the original request without service publication.
use super::*;
use crate::history::data::RequiredOutputPublicationPositions;
use worth_execution::ExecutionRequest;

impl RelationalBranchRoot {
    /// Only the actual mutation owner's empty journal may select this path.
    /// The original descriptor still must match the prior immutable storage.
    pub(crate) fn readmit_unchanged_for_request(
        issuer: &RelationalBranchRootIdentityIssuer,
        previous: &Arc<Self>,
        envelope: Arc<CanonicalCommitEnvelope>,
        descriptor: RelationalBranchRootDescriptor,
        schema: Arc<super::super::RelationalBranchRootSchemaAuthority>,
        positions: RequiredOutputPublicationPositions,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let memory = super::super::reserve_arc_memory::<Self>(Some(execution))?;
        let prepared = ReadmittedRegionSet {
            root_id: issuer.issue_root_id()?,
            regions: previous.regions.clone(),
            entity_slot_count: previous.entity_slot_count,
            relation_slot_count: previous.relation_slot_count,
            publication_cost: super::super::RelationalBranchRootPublicationCost {
                reused_regions: previous.regions.len() as u64,
                reused_schema_authorities: 1,
                new_authoritative_bytes: size_of::<Self>() as u64,
                ..Default::default()
            },
        };
        bind_original(
            prepared,
            envelope,
            descriptor,
            schema,
            positions,
            execution,
            memory,
            previous.retained_content(),
        )
    }
}

fn bind_original(
    prepared: ReadmittedRegionSet,
    envelope: Arc<CanonicalCommitEnvelope>,
    descriptor: RelationalBranchRootDescriptor,
    schema: Arc<super::super::RelationalBranchRootSchemaAuthority>,
    positions: RequiredOutputPublicationPositions,
    execution: &ExecutionRequest<'_, '_>,
    memory: Option<worth_execution::ExecutionMemoryReservation>,
    prior_content: &crate::content::ContentDirectory,
) -> Result<Arc<RelationalBranchRoot>, RelationalBranchRootCaptureDenial> {
    validate_schema_root(&envelope, &descriptor)?;
    let (storage_root, content_accumulator) = axes::storage_root_for(&prepared.regions);
    require_descriptor_content(&descriptor, storage_root)?;
    require_schema_authority(&schema, &envelope, &descriptor)?;
    let schema_root = *descriptor.schema_root();
    let correctness_index = RelationalRootCorrectnessIndex::AuthoritativeFallback;
    let visibility = RelationalBranchVisibilityCommitment::for_root_for_request(
        &envelope,
        storage_root,
        schema_root,
        correctness_index,
        execution,
    )?;
    crate::execution::check_request(execution)
        .map_err(RelationalBranchRootCaptureDenial::Request)?;
    let storage_version = envelope.commit.version_id.0;
    let incoming = envelope.content.records().clone();
    let mut root = RelationalBranchRoot {
        content: Default::default(),
        id: prepared.root_id,
        managed_source_image: Default::default(),
        regions: prepared.regions,
        entity_slot_count: prepared.entity_slot_count,
        relation_slot_count: prepared.relation_slot_count,
        content_accumulator,
        output_publication_positions: positions,
        schema_authority: schema,
        execution_contracts: Default::default(),
        committed: Some(RelationalCommittedBranchRoot {
            descriptor,
            evidence: super::super::RelationalRootCommitEvidence::Original(envelope),
            axes: RelationalBranchRootAxes {
                storage_version,
                storage_root,
                schema_root,
                correctness_index,
                visibility,
            },
            publication_cost: prepared.publication_cost,
        }),
        _memory: memory,
    };
    root.content = prior_content
        .reconstruct_original(
            execution,
            root.schema_authority.registry(),
            &incoming,
            &root,
            crate::identity::data::VersionId(storage_version),
        )
        .map_err(RelationalBranchRootCaptureDenial::Content)?;
    Ok(Arc::new(root))
}
