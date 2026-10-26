use std::sync::Arc;

use crate::history::data::CanonicalCommitEnvelope;
use crate::storage::overlay::PartitionAccess;
use crate::storage::RelationalPublishedPartitionDelta;

use super::{
    axes, capture, PreparedRelationalBranchRootCapture, RelationalBranchRoot,
    RelationalBranchRootAxes, RelationalBranchRootCaptureDenial,
    RelationalBranchRootIdentityIssuer, RelationalBranchRootPublicationCost,
    RelationalBranchRootSchemaAuthority, RelationalBranchVisibilityCommitment,
    RelationalCommittedBranchRoot, RelationalPersistentRegionSet, RelationalRootCorrectnessIndex,
};

struct PreparedRootRegions {
    regions: Arc<RelationalPersistentRegionSet>,
    entity_slot_count: usize,
    relation_slot_count: usize,
    storage_root: [u8; 32],
    content_accumulator: [u8; 32],
    publication_cost: RelationalBranchRootPublicationCost,
}

struct PreparedRootSchema {
    authority: Arc<RelationalBranchRootSchemaAuthority>,
    schema_root: [u8; 32],
    allocation: PreparedSchemaAllocation,
}

enum PreparedSchemaAllocation {
    Reused,
    New { authoritative_bytes: u64 },
}

pub(super) fn prepare(
    issuer: &RelationalBranchRootIdentityIssuer,
    partitions: &impl PartitionAccess,
    published_delta: &RelationalPublishedPartitionDelta,
    previous: Option<&Arc<RelationalBranchRoot>>,
    envelope: Arc<CanonicalCommitEnvelope>,
    registry: &crate::schema::data::RelationalSchemaRegistry,
    symbols: &crate::runtime::RuntimeSymbolTable,
) -> Result<PreparedRelationalBranchRootCapture, RelationalBranchRootCaptureDenial> {
    let root_id = issuer.issue_root_id()?;
    let prepared_schema = prepare_schema(issuer, previous, &envelope, registry)?;
    let prepared_regions = prepare_regions(
        issuer,
        root_id,
        partitions,
        published_delta,
        previous,
        symbols,
        &prepared_schema.authority,
    )?;
    let publication_cost = prepared_schema.apply_cost(prepared_regions.publication_cost);
    let correctness_index = RelationalRootCorrectnessIndex::AuthoritativeFallback;
    let axes = RelationalBranchRootAxes {
        storage_version: envelope.commit.version_id.0,
        storage_root: prepared_regions.storage_root,
        schema_root: prepared_schema.schema_root,
        correctness_index,
        visibility: RelationalBranchVisibilityCommitment::for_root(
            &envelope,
            prepared_regions.storage_root,
            prepared_schema.schema_root,
            correctness_index,
        ),
    };
    let root = Arc::new(RelationalBranchRoot {
        id: root_id,
        regions: prepared_regions.regions,
        entity_slot_count: prepared_regions.entity_slot_count,
        relation_slot_count: prepared_regions.relation_slot_count,
        content_accumulator: prepared_regions.content_accumulator,
        output_publication_positions: if envelope.is_unmerged_output_publication_step() {
            previous
                .map(|root| root.output_publication_positions.clone())
                .unwrap_or_default()
        } else {
            // The recovery class refuses merged physical output ownership.
            // Carry no old selector across an unsupported canonical step.
            crate::history::data::RequiredOutputPublicationPositions::unresolved_at(
                envelope.commit.commit_id,
            )
        },
        schema_authority: prepared_schema.authority,
        committed: Some(RelationalCommittedBranchRoot {
            descriptor: super::RelationalBranchRootDescriptor::new(
                axes.storage_root,
                axes.schema_root,
            ),
            canonical_envelope: envelope,
            axes,
            publication_cost,
        }),
    });
    Ok(PreparedRelationalBranchRootCapture { root })
}

#[allow(clippy::too_many_arguments)]
fn prepare_regions(
    issuer: &RelationalBranchRootIdentityIssuer,
    root_id: u64,
    partitions: &impl PartitionAccess,
    published_delta: &RelationalPublishedPartitionDelta,
    previous: Option<&Arc<RelationalBranchRoot>>,
    symbols: &crate::runtime::RuntimeSymbolTable,
    schema: &Arc<RelationalBranchRootSchemaAuthority>,
) -> Result<PreparedRootRegions, RelationalBranchRootCaptureDenial> {
    let (regions, publication_cost) = match previous {
        Some(previous) if previous.committed.is_some() => capture::build_incremental_regions(
            issuer,
            root_id,
            published_delta,
            previous,
            symbols,
            schema,
        )?,
        _ => {
            let projected = published_delta.projected_partitions_from_access(partitions);
            capture::build_initial_regions(
                issuer,
                root_id,
                &projected,
                &symbols.interner_snapshot(),
            )?
        }
    };
    let (entity_slot_count, relation_slot_count) =
        root_slot_counts(&regions, previous, published_delta);
    let (storage_root, content_accumulator) = axes::storage_root_for(&regions);
    Ok(PreparedRootRegions {
        regions,
        entity_slot_count,
        relation_slot_count,
        storage_root,
        content_accumulator,
        publication_cost,
    })
}

fn root_slot_counts(
    regions: &RelationalPersistentRegionSet,
    previous: Option<&Arc<RelationalBranchRoot>>,
    published_delta: &RelationalPublishedPartitionDelta,
) -> (usize, usize) {
    let Some(previous) = previous.filter(|root| root.committed.is_some()) else {
        return regions
            .values()
            .fold((0, 0), |(entities, relations), region| {
                (
                    entities.saturating_add(region.entity_slots),
                    relations.saturating_add(region.relation_slots),
                )
            });
    };
    published_delta.partition_ids().fold(
        (previous.entity_slot_count, previous.relation_slot_count),
        |(entities, relations), partition_id| {
            let old = previous.regions.get(partition_id);
            let new = regions.get(partition_id);
            (
                entities
                    .saturating_sub(old.map_or(0, |region| region.entity_slots))
                    .saturating_add(new.map_or(0, |region| region.entity_slots)),
                relations
                    .saturating_sub(old.map_or(0, |region| region.relation_slots))
                    .saturating_add(new.map_or(0, |region| region.relation_slots)),
            )
        },
    )
}

fn prepare_schema(
    issuer: &RelationalBranchRootIdentityIssuer,
    previous: Option<&Arc<RelationalBranchRoot>>,
    envelope: &CanonicalCommitEnvelope,
    registry: &crate::schema::data::RelationalSchemaRegistry,
) -> Result<PreparedRootSchema, RelationalBranchRootCaptureDenial> {
    let schema_root =
        crate::schema::data::schema_authority_snapshot_digest_bytes(&envelope.schema_authority);
    if let Some(previous) =
        previous.filter(|root| root.schema_authority.matches(&envelope.schema_authority))
    {
        return Ok(PreparedRootSchema {
            authority: Arc::clone(&previous.schema_authority),
            schema_root,
            allocation: PreparedSchemaAllocation::Reused,
        });
    }
    let allocation_id = issuer.issue_schema_authority_id()?;
    let authority = RelationalBranchRootSchemaAuthority::capture(
        allocation_id,
        registry,
        &envelope.schema_authority,
        envelope.descriptor_semantics_version,
        previous.map(|root| root.schema_authority.as_ref()),
    )
    .ok_or(RelationalBranchRootCaptureDenial::SchemaRootMismatch {
        descriptor_root: schema_root,
        canonical_root: registry.authority_digest_bytes(),
    })?;
    Ok(PreparedRootSchema {
        allocation: PreparedSchemaAllocation::New {
            authoritative_bytes: authority.authoritative_allocation_bytes(),
        },
        authority,
        schema_root,
    })
}

impl PreparedRootSchema {
    fn apply_cost(
        &self,
        mut cost: RelationalBranchRootPublicationCost,
    ) -> RelationalBranchRootPublicationCost {
        match self.allocation {
            PreparedSchemaAllocation::Reused => cost.reused_schema_authorities = 1,
            PreparedSchemaAllocation::New {
                authoritative_bytes,
            } => {
                cost.new_schema_authorities = 1;
                cost.new_authoritative_bytes = cost
                    .new_authoritative_bytes
                    .saturating_add(authoritative_bytes);
            }
        }
        cost
    }
}
