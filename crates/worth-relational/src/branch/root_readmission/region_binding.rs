//! Construct a private root and certify its complete content closure before selection.
use super::*;

struct ReadmittedRegionSet {
    root_id: u64,
    regions: Arc<crate::branch::RelationalPersistentRegionSet>,
    entity_slot_count: usize,
    relation_slot_count: usize,
    publication_cost: crate::branch::root::RelationalBranchRootPublicationCost,
}

impl RelationalBranchRoot {
    pub(crate) fn readmit(
        issuer: &RelationalBranchRootIdentityIssuer,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        content_records: &crate::content::RelationalContentAttachments,
        partitions: BTreeMap<PartitionId, PartitionState>,
        envelope: Arc<CanonicalCommitEnvelope>,
        descriptor: RelationalBranchRootDescriptor,
        schema_authority: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        validate_schema_root(&envelope, &descriptor)?;
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            required_text::validate(lane, &partitions, symbols)
        })?;
        for partition in partitions.values() {
            crate::history::data::current_route_presence::admission::validate(
                execution,
                partition,
                schema_authority.registry(),
            )
            .map_err(|cause| match cause {
                crate::storage::residency::RelationalResidencyDemandFailure::StorageUnavailable(
                    cause,
                ) => RelationalBranchRootCaptureDenial::StorageUnavailable(cause),
                crate::storage::residency::RelationalResidencyDemandFailure::Request(cause) => {
                    RelationalBranchRootCaptureDenial::Execution(cause)
                }
                crate::storage::residency::RelationalResidencyDemandFailure::Content(cause) => {
                    RelationalBranchRootCaptureDenial::Content(cause)
                }
            })?;
        }
        let root_id = issuer.issue_root_id()?;
        let entity_slot_count = partitions
            .values()
            .map(|partition| partition.entity_arena.slot_count())
            .sum();
        let relation_slot_count = partitions
            .values()
            .map(|partition| partition.relation_arena.slot_count())
            .sum();
        let (regions, publication_cost) =
            crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                |lane| {
                    capture::build_initial_regions_owned(
                        lane, execution, issuer, root_id, partitions, symbols,
                    )
                },
            )?;
        Self::bind_readmitted_regions(
            ReadmittedRegionSet {
                root_id,
                regions,
                entity_slot_count,
                relation_slot_count,
                publication_cost,
            },
            envelope,
            descriptor,
            schema_authority,
            execution,
            content_records,
        )
    }

    pub(crate) fn readmit_region_directory(
        issuer: &RelationalBranchRootIdentityIssuer,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        content_records: &crate::content::RelationalContentAttachments,
        directory: Vec<crate::branch::RelationalRootRegionCheckpoint>,
        envelope: Arc<CanonicalCommitEnvelope>,
        descriptor: RelationalBranchRootDescriptor,
        schema_authority: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let root_id = issuer.issue_root_id()?;
        let mut regions = BTreeMap::new();
        let mut entity_slot_count = 0_usize;
        let mut relation_slot_count = 0_usize;
        for region in directory {
            entity_slot_count = entity_slot_count
                .checked_add(region.entity_slots)
                .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
            relation_slot_count = relation_slot_count
                .checked_add(region.relation_slots)
                .ok_or(RelationalBranchRootCaptureDenial::SlotCountOverflow)?;
            let id = region.partition_id;
            let region = crate::branch::root_region::RelationalRootRegion::from_checkpoint(
                root_id,
                issuer.issue_region_id()?,
                region,
            );
            if regions.insert(id, Arc::new(region)).is_some() {
                return Err(RelationalBranchRootCaptureDenial::DuplicateRegion(id));
            }
        }
        let regions =
            crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                |lane| {
                    crate::branch::RelationalPersistentRegionSet::initial(
                        lane, root_id, regions, issuer,
                    )
                },
            )?;
        Self::bind_readmitted_regions(
            ReadmittedRegionSet {
                root_id,
                regions,
                entity_slot_count,
                relation_slot_count,
                publication_cost: Default::default(),
            },
            envelope,
            descriptor,
            schema_authority,
            execution,
            content_records,
        )
    }

    fn bind_readmitted_regions(
        prepared: ReadmittedRegionSet,
        envelope: Arc<CanonicalCommitEnvelope>,
        descriptor: RelationalBranchRootDescriptor,
        schema_authority: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        content_records: &crate::content::RelationalContentAttachments,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let ReadmittedRegionSet {
            root_id,
            regions,
            entity_slot_count,
            relation_slot_count,
            mut publication_cost,
        } = prepared;
        validate_schema_root(&envelope, &descriptor)?;
        let (storage_root, content_accumulator) = axes::storage_root_for(&regions);
        require_descriptor_content(&descriptor, storage_root)?;
        let schema_root = *descriptor.schema_root();
        if !schema_authority.matches(&envelope.schema_authority)
            || schema_authority.descriptor_semantics_version()
                != envelope.descriptor_semantics_version
        {
            return Err(RelationalBranchRootCaptureDenial::SchemaRootMismatch {
                descriptor_root: schema_root,
                canonical_root: schema_authority.registry().authority_digest_bytes(),
            });
        }
        publication_cost.new_schema_authorities = 1;
        publication_cost.new_authoritative_bytes = publication_cost
            .new_authoritative_bytes
            .saturating_add(schema_authority.authoritative_allocation_bytes());
        let correctness_index = RelationalRootCorrectnessIndex::AuthoritativeFallback;
        let axes = RelationalBranchRootAxes {
            storage_version: envelope.commit.version_id.0,
            storage_root,
            schema_root,
            correctness_index,
            visibility: RelationalBranchVisibilityCommitment::for_root(
                &envelope,
                storage_root,
                schema_root,
                correctness_index,
            ),
        };
        let version = envelope.commit.version_id;
        let mut root = Self {
            // Reconstructed storage has no live companion association.
            content: crate::content::ContentDirectory::default(),
            managed_source_image: std::sync::OnceLock::new(),
            id: root_id,
            regions,
            entity_slot_count,
            relation_slot_count,
            content_accumulator,
            output_publication_positions:
                crate::history::data::RequiredOutputPublicationPositions::unresolved_at(
                    envelope.commit.commit_id,
                ),
            schema_authority,
            committed: Some(RelationalCommittedBranchRoot {
                commit_change: None,
                descriptor,
                canonical_envelope: envelope,
                axes,
                publication_cost,
            }),
        };
        root.content =
            crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                |lane| {
                    crate::content::ContentDirectory::reconstruct(
                        lane,
                        execution,
                        root.schema_authority.registry(),
                        content_records,
                        root.schema_authority.retained_aspect_contracts(),
                        &root,
                        version,
                    )
                },
            )
            .map_err(RelationalBranchRootCaptureDenial::Content)?;
        Ok(Arc::new(root))
    }
}

impl RelationalBranchRoot {
    /// Explicit verification of resident bytes at recovery/inspection boundaries.
    /// The capability remains inside this fixed operation, on the caller's request.
    pub(in crate::branch::root) fn verify_resident_storage_content(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<([u8; 32], [u8; 32]), RelationalBranchRootCaptureDenial> {
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            axes::storage_root_from_authoritative_regions(lane, execution, &self.regions, symbols)
        })
    }
}
