//! Explicit materialization of a member directory for reconstructive consumers.
use std::collections::BTreeMap;
use std::sync::Arc;

use super::manifest_capture::corrupt;
use super::member_frame::{self, MemberKind};
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_to_image_with_contracts;
use crate::durability::data::*;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
use crate::storage::residency::{RelationalColdRegion, RelationalRegionResidency};

const MEMBER_LIMIT: u64 = 1 << 30;

pub(super) fn expand(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    manifest: RelationalCheckpointManifest,
    source: Arc<dyn RelationalDurableColdSource>,
    history_members: Arc<RelationalHistoryMemberDirectory>,
) -> Result<DurableCheckpoint, RelationalRecoveryStop> {
    let directory = Arc::new(
        PreparedCheckpointDirectory {
            manifest: manifest.clone(),
            source: source.clone(),
            history_members: history_members.clone(),
        }
        .readmit(execution)?,
    );
    let mut envelopes = Vec::new();
    for entry in &manifest.commits {
        envelopes.push(
            crate::history::RelationalCommitEnvelopeResidency::readmit_selected(
                entry,
                &directory,
                execution,
                MEMBER_LIMIT,
            )?,
        );
    }
    let mut symbol_table = crate::symbols::data::SymbolTableSnapshot::default();
    for page in &manifest.symbols {
        let bytes = read(
            &*source,
            page.member,
            RelationalStorageSubject::Symbols {
                first: crate::symbols::data::Symbol(page.first as u32),
                last: crate::symbols::data::Symbol(page.last as u32),
            },
        )?;
        let payload = member_frame::decode(&bytes, page.member, MemberKind::Symbols)
            .map_err(|cause| corrupt(format!("symbol member frame: {cause:?}")))?;
        let snapshot =
            crate::symbols::data::SymbolTableSnapshot::readmit_current(execution, payload)?;
        if RelationalCheckpointPage::from_ids(
            snapshot.entries.iter().map(|(id, _)| u64::from(id.0)),
            page.member,
        )
        .as_ref()
            != Some(page)
        {
            return Err(corrupt("symbol page membership mismatch").into());
        }
        symbol_table.entries.extend(snapshot.entries);
    }
    symbol_table.entries.sort_by_key(|(id, _)| *id);
    let mut symbols = crate::symbols::data::StringInterner::default();
    symbols.restore_snapshot(symbol_table.clone());
    let admitted_symbols = crate::runtime::RuntimeSymbolTable::default();
    admitted_symbols.replace(symbols);
    let symbols = admitted_symbols;
    let mut schemas = BTreeMap::new();
    for image in &manifest.schemas {
        let (registry, contracts) = image.readmit_registry().map_err(corrupt)?;
        let expected = registry.authority_snapshot();
        let schema = crate::branch::RelationalBranchRootSchemaAuthority::readmit_exact(
            schemas.len() as u64 + 1,
            registry,
            contracts,
            &expected,
            image.descriptor_semantics_version(),
        )
        .ok_or_else(|| corrupt("checkpoint schema readmission failed"))?;
        if schemas.insert(image.carrier_digest(), schema).is_some() {
            return Err(corrupt("duplicate checkpoint schema carrier").into());
        }
    }
    let mut branch_roots = Vec::new();
    let mut root_kind_images = BTreeMap::new();
    for root in &manifest.roots {
        let (partition_images, kinds) = expand_regions(
            execution,
            &root.regions,
            root.schema_carrier_digest,
            &schemas,
            &symbols,
            &source,
            &history_members,
        )?;
        root_kind_images.insert(root.commit_id, kinds);
        let partition_image_digest = branch_root_partition_image_digest(&partition_images)
            .map_err(|error| corrupt(error.to_string()))?;
        let format_version = DurableBranchRootImage::CURRENT_FORMAT_VERSION;
        branch_roots.push(DurableBranchRootImage {
            format_version,
            commit_id: root.commit_id,
            partition_images,
            partition_image_digest,
            schema_carrier_digest: root.schema_carrier_digest,
            root_image_digest: branch_root_image_digest(
                format_version,
                root.commit_id,
                partition_image_digest,
                root.schema_carrier_digest,
            ),
        });
    }
    let (partition_images, mirror_kind_images) = if let Some(schema) = manifest.mirror_schema {
        expand_regions(
            execution,
            &manifest.mirror,
            schema,
            &schemas,
            &symbols,
            &source,
            &history_members,
        )?
    } else if manifest.mirror.is_empty() {
        (Vec::new(), Default::default())
    } else {
        return Err(corrupt("checkpoint mirror has no schema authority").into());
    };
    let mut lineage_nodes = Vec::<crate::lineage::data::LineageNode>::new();
    for page in &manifest.lineage {
        let bytes = read(
            &*source,
            page.member,
            RelationalStorageSubject::Lineage {
                first: crate::identity::data::LineageId(page.first),
                last: crate::identity::data::LineageId(page.last),
            },
        )?;
        let payload = member_frame::decode(&bytes, page.member, MemberKind::Lineage)
            .map_err(|cause| corrupt(format!("lineage member frame: {cause:?}")))?;
        let nodes: Vec<crate::lineage::data::LineageNode> =
            rmp_serde::from_slice(payload).map_err(|error| corrupt(error.to_string()))?;
        if RelationalCheckpointPage::from_ids(
            nodes.iter().map(|node| node.lineage_id().0),
            page.member,
        )
        .as_ref()
            != Some(page)
        {
            return Err(corrupt("lineage page membership mismatch").into());
        }
        lineage_nodes.extend(nodes);
    }
    lineage_nodes.sort_by_key(|node| node.lineage_id());
    let bytes = read(
        &*source,
        manifest.derived_indexes,
        RelationalStorageSubject::Index(crate::indexes::data::DerivedIndexId(0)),
    )?;
    let payload = member_frame::decode(&bytes, manifest.derived_indexes, MemberKind::DerivedIndex)
        .map_err(|cause| corrupt(format!("index member frame: {cause:?}")))?;
    let (derived_index_artifacts, derived_index_checkpoint, derived_index_checkpoint_format) =
        rmp_serde::from_slice(payload).map_err(|error| corrupt(error.to_string()))?;
    let manifest = manifest.ordinary_owned_draft();
    let (runtime_name, branch_cells, branch_lifecycle, retired_branch_names) =
        manifest.branch_data.ordinary_owned_parts();
    Ok(DurableCheckpoint {
        recorded_components: manifest
            .roots
            .iter()
            .map(|root| (root.commit_id, root.recorded_components.clone()))
            .collect(),
        retained_empty_components: manifest.retained_empty_components.clone(),
        directory: None,
        canonical_directory: Some(crate::durability::data::NativeCheckpointDirectoryEdge::new(
            directory,
        )),
        root_kind_images,
        mirror_kind_images,
        coverage: manifest.coverage,
        commit_descriptors: manifest
            .commits
            .iter()
            .map(|entry| (entry.commit.commit_id, entry.root.clone()))
            .collect(),
        mirror_schema_digest: manifest.mirror_schema,
        branch_cells,
        branch_lifecycle,
        retired_branch_names,
        branch_roots,
        branch_root_schema_images: manifest.schemas.ordinary_owned_copy(),
        record_identity: manifest.record_identity,
        record_generation_high_water: Vec::new(),
        reusable_record_slots: Vec::new(),
        record_slot_frontiers: Vec::new(),
        envelopes,
        partition_images,
        aspect_contracts: manifest.aspect_contracts,
        lineage: crate::lineage::data::LineageCheckpointArtifact::new(
            manifest.lineage_basis.ordinary_owned_copy(),
            lineage_nodes,
        ),
        index_generation_frontier: manifest.index_generation_frontier,
        index_definition_frontier: manifest.index_definition_frontier,
        index_definitions: manifest.index_definitions.ordinary_owned_copy(),
        derived_index_artifacts,
        derived_index_checkpoint,
        derived_index_checkpoint_format,
        symbol_table,
        symbol_identity_frontier: manifest.symbol_identity_frontier,
        runtime_name,
    })
}

fn expand_regions(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    regions: &[RelationalCheckpointRegion],
    schema_digest: [u8; 32],
    schemas: &BTreeMap<[u8; 32], Arc<crate::branch::RelationalBranchRootSchemaAuthority>>,
    symbols: &crate::runtime::RuntimeSymbolTable,
    source: &Arc<dyn RelationalDurableColdSource>,
    history_members: &Arc<RelationalHistoryMemberDirectory>,
) -> Result<
    (
        Vec<PartitionCheckpointImage>,
        crate::storage::residency::CheckpointRegionKindImages,
    ),
    RelationalRecoveryStop,
> {
    let schema = schemas
        .get(&schema_digest)
        .ok_or_else(|| corrupt("missing region schema carrier"))?;
    let contracts =
        CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())?;
    let authority =
        crate::storage::residency::RelationalRegionKindAuthority::for_request(execution)?;
    let cells = regions
        .iter()
        .map(|leaf| {
            let cell = RelationalRegionResidency::cold(RelationalColdRegion {
                kind_authority: leaf.kind_authority,
                source: Some(source.clone()),
                partition_id: leaf.partition_id,
                head: leaf.head,
                history: leaf.history,
                history_members: history_members.clone(),
                schema: schema.clone(),
                schema_carrier_digest: schema_digest,
                symbols: symbols.clone(),
                content_digest: leaf.content_digest,
                entity_slots: leaf.entity_slots,
                relation_slots: leaf.relation_slots,
                unavailable_records: leaf.unavailable_records,
            })?;
            Ok((leaf.partition_id, cell))
        })
        .collect::<Result<Vec<_>, crate::storage::data::RelationalStorageUnavailable>>()?;
    authority.bind_for_request(cells.iter().cloned(), execution)?;
    let admitted = cells
        .into_iter()
        .map(|(_, cell)| {
            let lease = cell.demand_full(execution, MEMBER_LIMIT, Some(&authority))?;
            drop(cell);
            Ok(lease.into_owned_partition())
        })
        .collect::<Result<Vec<_>, RelationalRecoveryStop>>()?;
    let kinds = crate::storage::residency::CheckpointRegionKindImages::capture(&admitted)
        .map_err(|cause| corrupt(format!("expanded kind projection: {cause:?}")))?;
    let images = admitted
        .into_iter()
        .map(|partition| {
            partition_to_image_with_contracts(partition, schema.aspect_plans(), &contracts)
                .map_err(Into::into)
        })
        .collect::<Result<Vec<_>, RelationalRecoveryStop>>()?;
    Ok((images, kinds))
}

fn read(
    source: &dyn RelationalDurableColdSource,
    locator: RelationalDurableMemberLocator,
    subject: RelationalStorageSubject,
) -> Result<Vec<u8>, RelationalRecoveryStop> {
    source
        .read_member(locator, MEMBER_LIMIT)
        .map_err(|cause| RelationalStorageUnavailable { subject, cause }.into())
}
