use std::collections::BTreeMap;
use std::sync::Arc;

use super::member_frame::{self, MemberKind};
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_from_image;
use crate::durability::data::*;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};

pub(super) struct CapturedManifest {
    pub(super) manifest: RelationalCheckpointManifest,
    pub(super) members: BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
    pub(super) envelopes: Vec<Vec<u8>>,
}

impl CapturedManifest {
    pub(super) fn capture(
        checkpoint: &DurableCheckpoint,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, RelationalNativeCheckpointCaptureDenial> {
        let mut members = BTreeMap::new();
        let mut symbols = crate::symbols::data::StringInterner::default();
        symbols.restore_snapshot(checkpoint.symbol_table.clone());
        let mut roots = Vec::new();
        let mut schemas = BTreeMap::new();
        for image in &checkpoint.branch_root_schema_images {
            let (registry, contracts) = image.readmit_registry().map_err(corrupt)?;
            let expected = registry.authority_snapshot();
            let schema = crate::branch::RelationalBranchRootSchemaAuthority::readmit_exact(
                schemas.len() as u64 + 1,
                registry,
                contracts,
                &expected,
                image.descriptor_semantics_version(),
            )
            .ok_or_else(|| corrupt("schema carrier readmission failed"))?;
            schemas.insert(image.carrier_digest(), schema);
        }
        for root in &checkpoint.branch_roots {
            let schema = schemas
                .get(&root.schema_carrier_digest)
                .ok_or_else(|| corrupt("missing root schema carrier"))?;
            let partitions = restore_images(&root.partition_images, schema)?;
            let regions = capture_partitions(
                &partitions,
                schema,
                root.schema_carrier_digest,
                &symbols,
                &mut members,
            )?;
            roots.push(RelationalCheckpointRoot {
                commit_id: root.commit_id,
                schema_carrier_digest: root.schema_carrier_digest,
                regions,
            });
        }
        let mirror = if checkpoint.partition_images.is_empty() {
            Vec::new()
        } else {
            let digest = checkpoint
                .mirror_schema_digest
                .ok_or_else(|| corrupt("nonempty mirror has no schema carrier"))?;
            let schema = schemas
                .get(&digest)
                .ok_or_else(|| corrupt("missing mirror schema carrier"))?;
            capture_partitions(
                &restore_images(&checkpoint.partition_images, schema)?,
                schema,
                digest,
                &symbols,
                &mut members,
            )?
        };
        let mut commits = Vec::new();
        let mut envelopes = Vec::new();
        for positioned in &checkpoint.envelopes {
            policy.check_live()?;
            let payload_digest =
                RelationalCanonicalCommitArtifact::positioned_payload_digest(positioned)
                    .map_err(|error| corrupt(format!("canonical checkpoint digest: {error}")))?;
            let envelope = positioned.envelope();
            let root = checkpoint
                .commit_descriptors
                .get(&envelope.commit.commit_id)
                .ok_or_else(|| corrupt("checkpoint commit lacks its catalog descriptor"))?;
            commits.push(RelationalCheckpointCommit {
                commit: envelope.commit.clone(),
                position: positioned.position(),
                root: root.clone(),
                payload_digest,
                schema_version: envelope.schema_version,
            });
            envelopes.push(
                rmp_serde::to_vec_named(&super::embedded_commit::EmbeddedCommitRef::new(
                    positioned,
                ))
                .map_err(|error| corrupt(format!("embedded checkpoint envelope: {error}")))?,
            );
        }
        let mut entries = checkpoint.symbol_table.entries.clone();
        entries.sort_by_key(|(symbol, _)| *symbol);
        let mut symbol_pages = Vec::new();
        for page in entries.chunks(256) {
            let value = crate::symbols::data::SymbolTableSnapshot {
                entries: page.to_vec(),
            };
            let member = add(
                &mut members,
                member_frame::encode(MemberKind::Symbols, &value)
                    .map_err(|cause| corrupt(format!("symbol member: {cause:?}")))?,
            );
            symbol_pages.push(RelationalCheckpointPage {
                first: page[0].0 .0 as u64,
                last: page.last().unwrap().0 .0 as u64,
                member,
            });
        }
        let mut lineage_pages = Vec::new();
        for page in checkpoint.lineage.nodes().chunks(256) {
            let member = add(
                &mut members,
                member_frame::encode(MemberKind::Lineage, &page)
                    .map_err(|cause| corrupt(format!("lineage member: {cause:?}")))?,
            );
            lineage_pages.push(RelationalCheckpointPage {
                first: page[0].lineage_id.0,
                last: page.last().unwrap().lineage_id.0,
                member,
            });
        }
        let derived_indexes = add(
            &mut members,
            member_frame::encode(
                MemberKind::DerivedIndex,
                &(
                    &checkpoint.derived_index_artifacts,
                    &checkpoint.derived_index_checkpoint,
                    checkpoint.derived_index_checkpoint_format,
                ),
            )
            .map_err(|cause| corrupt(format!("derived index member: {cause:?}")))?,
        );
        Ok(Self {
            manifest: RelationalCheckpointManifest {
                coverage: checkpoint.coverage.clone(),
                covered_position: commits.last().map_or(
                    crate::publication::patch::data::PatchStreamPosition(0),
                    |entry| entry.position,
                ),
                runtime_name: checkpoint.runtime_name.clone(),
                branch_cells: checkpoint.branch_cells.clone(),
                branch_lifecycle: checkpoint.branch_lifecycle.clone(),
                retired_branch_names: checkpoint.retired_branch_names.clone(),
                schemas: checkpoint.branch_root_schema_images.clone(),
                record_identity: checkpoint.record_identity.clone(),
                aspect_contracts: checkpoint.aspect_contracts.clone(),
                index_definitions: checkpoint.index_definitions.clone(),
                roots,
                mirror_schema: checkpoint.mirror_schema_digest,
                mirror,
                commits,
                symbols: symbol_pages,
                lineage: lineage_pages,
                lineage_basis: checkpoint.lineage.digest_basis().clone(),
                derived_indexes,
            },
            members,
            envelopes,
        })
    }
}

fn restore_images(
    images: &[PartitionCheckpointImage],
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<BTreeMap<crate::identity::data::PartitionId, PartitionState>, DurabilityError> {
    let contracts =
        CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())?;
    let mut partitions = BTreeMap::new();
    for image in images {
        let partition = partition_from_image(image.clone(), schema.aspect_plans(), &contracts)?;
        if partitions
            .insert(partition.partition_id, partition)
            .is_some()
        {
            return Err(corrupt("duplicate partition image"));
        }
    }
    crate::storage::partition::rebuild_adjacency_kind_buckets(&mut partitions).map_err(corrupt)?;
    Ok(partitions)
}

fn capture_partitions(
    partitions: &BTreeMap<crate::identity::data::PartitionId, PartitionState>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    schema_digest: [u8; 32],
    symbols: &crate::symbols::data::StringInterner,
    members: &mut BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
) -> Result<Vec<RelationalCheckpointRegion>, DurabilityError> {
    partitions
        .values()
        .map(|partition| {
            let (commitment, _) = PartitionContentCommitment::capture(partition, symbols, None)
                .map_err(|error| corrupt(format!("region commitment: {error:?}")))?;
            let captured = crate::storage::residency::capture_region_members(
                partition,
                &commitment,
                schema,
                schema_digest,
            )
            .map_err(|cause| corrupt(format!("region member: {cause:?}")))?;
            Ok(RelationalCheckpointRegion {
                partition_id: partition.partition_id,
                content_digest: commitment.digest(partition.partition_id),
                entity_slots: partition.entity_arena.slot_count(),
                relation_slots: partition.relation_arena.slot_count(),
                unavailable_records: partition.entity_arena.lifecycle_counts().unavailable
                    + partition.relation_arena.lifecycle_counts().unavailable,
                head: add(members, captured.head),
                history: add(members, captured.history),
            })
        })
        .collect()
}

fn add(
    members: &mut BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
    (locator, bytes): (RelationalDurableMemberLocator, Vec<u8>),
) -> RelationalDurableMemberLocator {
    members.entry(locator).or_insert_with(|| bytes.into());
    locator
}

pub(super) fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
