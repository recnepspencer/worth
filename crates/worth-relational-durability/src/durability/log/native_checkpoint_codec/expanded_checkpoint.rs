//! Explicit materialization of a member directory for reconstructive consumers.
use std::collections::BTreeMap;
use std::sync::Arc;

use super::manifest_capture::corrupt;
use super::member_frame;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_to_image_with_contracts;
use crate::durability::data::*;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
};
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;
use worth_relational_storage::internal_port::storage::residency::cell::{
    RelationalColdRegion, RelationalRegionResidency,
};

const MEMBER_LIMIT: u64 = 1 << 30;

pub(super) fn validate_manifest(
    manifest: &RelationalCheckpointManifest,
) -> Result<super::manifest_validation::ManifestDirectoryFacts, RelationalRecoveryStop> {
    worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
        super::manifest_validation::validate(lane, manifest)
    })
    .map_err(Into::into)
}

#[allow(dead_code)]
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
            worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency::readmit_selected(
                entry,
                directory.as_ref(),
                execution,
                MEMBER_LIMIT,
            )?,
        );
    }
    let mut symbol_table = worth_relational_contracts::internal_port::symbols::data::symbol_table_snapshot::SymbolTableSnapshot::default();
    for page in &manifest.symbols {
        let bytes = read(
            &*source,
            page.member,
            RelationalStorageSubject::Symbols {
                first: worth_relational_contracts::internal_port::symbols::data::symbol::Symbol(
                    page.first as u32,
                ),
                last: worth_relational_contracts::internal_port::symbols::data::symbol::Symbol(
                    page.last as u32,
                ),
            },
        )?;
        let payload = member_frame::decode(&bytes, page.member, MemberKind::Symbols)
            .map_err(|cause| corrupt(format!("symbol member frame: {cause:?}")))?;
        let snapshot =
            worth_relational_contracts::internal_port::symbols::data::symbol_table_snapshot::SymbolTableSnapshot::readmit_current(execution, payload)?;
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
    let mut symbols = worth_relational_contracts::internal_port::symbols::data::string_interner::StringInterner::default();
    worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
        symbols.restore_snapshot(lane, symbol_table.clone())
    });
    let admitted_symbols =
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable::default();
    admitted_symbols.replace(symbols);
    let symbols = admitted_symbols;
    let mut schemas = BTreeMap::new();
    for image in &manifest.schemas {
        let (registry, contracts) = image.readmit_registry().map_err(corrupt)?;
        let expected = registry
            .snapshot_for_compilation(worth_relational_contracts::internal_port::schema::compilation_control::SchemaCompilationControl::Request(*execution))
            .map_err(RelationalRecoveryStop::Request)?;
        let schema =
            worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority::readmit_exact(
                schemas.len() as u64 + 1,
                registry,
                contracts,
                &expected,
                image.descriptor_semantics_version(),
                worth_relational_contracts::internal_port::schema::compilation_control::SchemaCompilationControl::Request(*execution),
            )
            .map_err(RelationalRecoveryStop::Request)?
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
        let partition_image_digest =
            worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
                |lane| branch_root_partition_image_digest(lane, &partition_images),
            )
            .map_err(|error| corrupt(error.to_string()))?;
        let format_version = DurableBranchRootImage::CURRENT_FORMAT_VERSION;
        let content_records = crate::durability::checkpoints::content::read_records(
            execution,
            &root.content,
            source.as_ref(),
        )?;
        branch_roots.push(DurableBranchRootImage {
            content_records: content_records.clone(),
            format_version,
            commit_id: root.commit_id,
            partition_images,
            partition_image_digest,
            schema_carrier_digest: root.schema_carrier_digest,
            root_image_digest:
                worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
                    |lane| {
                        branch_root_image_digest(
                            lane,
                            format_version,
                            root.commit_id,
                            partition_image_digest,
                            root.schema_carrier_digest,
                            &content_records,
                        )
                    },
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
    let mut lineage_nodes =
        Vec::<worth_relational_contracts::internal_port::lineage::data::graph::LineageNode>::new();
    for page in &manifest.lineage {
        let bytes = read(
            &*source,
            page.member,
            RelationalStorageSubject::Lineage {
                first:
                    worth_relational_contracts::internal_port::identity::data::primitives::LineageId(
                        page.first,
                    ),
                last:
                    worth_relational_contracts::internal_port::identity::data::primitives::LineageId(
                        page.last,
                    ),
            },
        )?;
        let payload = member_frame::decode(&bytes, page.member, MemberKind::Lineage)
            .map_err(|cause| corrupt(format!("lineage member frame: {cause:?}")))?;
        let nodes: Vec<
            worth_relational_contracts::internal_port::lineage::data::graph::LineageNode,
        > = rmp_serde::from_slice(payload).map_err(|error| corrupt(error.to_string()))?;
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
        RelationalStorageSubject::Index(
            worth_relational_contracts::internal_port::indexes::data::definition::DerivedIndexId(0),
        ),
    )?;
    let payload = member_frame::decode(&bytes, manifest.derived_indexes, MemberKind::DerivedIndex)
        .map_err(|cause| corrupt(format!("index member frame: {cause:?}")))?;
    let (derived_index_artifacts, derived_index_checkpoint, derived_index_checkpoint_format) =
        rmp_serde::from_slice(payload).map_err(|error| corrupt(error.to_string()))?;
    let manifest = manifest.ordinary_owned_draft();
    let (runtime_name, branch_cells, branch_lifecycle, retired_branch_names) =
        manifest.branch_data.ordinary_owned_parts();
    let relation_lineages: Vec<_> = envelopes
        .iter()
        .flat_map(|envelope| {
            envelope
                .envelope()
                .lineage_relation_lineages()
                .iter()
                .copied()
        })
        .collect();
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
        record_identity: super::identity_pages::load_identity(&manifest.record_identity.roots, &source)?,
        record_generation_high_water: Vec::new(),
        reusable_record_slots: Vec::new(),
        record_slot_frontiers: Vec::new(),
        envelopes,
        partition_images,
        aspect_contracts: manifest.aspect_contracts,
        lineage: worth_relational_contracts::internal_port::lineage::data::artifacts::checkpoint_artifact::LineageCheckpointArtifact::new(
            manifest.lineage_basis.ordinary_owned_copy(),
            lineage_nodes,
        )
        .with_relation_lineages(relation_lineages),
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
    schemas: &BTreeMap<
        [u8; 32],
        Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
    >,
    symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    source: &Arc<dyn RelationalDurableColdSource>,
    history_members: &Arc<RelationalHistoryMemberDirectory>,
) -> Result<
    (
        Vec<PartitionCheckpointImage>,
        worth_relational_storage::internal_port::storage::residency::checkpoint_kind_images::CheckpointRegionKindImages,
    ),
    RelationalRecoveryStop,
>{
    let schema = schemas
        .get(&schema_digest)
        .ok_or_else(|| corrupt("missing region schema carrier"))?;
    let contracts =
        CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())?;
    let authority =
        worth_relational_storage::internal_port::storage::residency::kind_authority::RelationalRegionKindAuthority::for_request(execution)?;
    let cells = regions
        .iter()
        .map(|leaf| {
            let cell = RelationalRegionResidency::cold(RelationalColdRegion {
                readmission: &crate::storage::residency::region_load::CHECKPOINT_REGION_READMISSION,
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
                directory: leaf.directory.clone(),
                live_counts: leaf.live_counts,
                entity_slots: leaf.entity_slots,
                relation_slots: leaf.relation_slots,
                unavailable_records: leaf.unavailable_records,
            })?;
            Ok((leaf.partition_id, cell))
        })
        .collect::<Result<Vec<_>, worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailable>>()?;
    worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
        authority.bind_for_request(lane, cells.iter().cloned(), execution)
    })?;
    let admitted = cells
        .into_iter()
        .map(|(_, cell)| {
            let lease = cell.demand_full(execution, MEMBER_LIMIT, Some(&authority))?;
            drop(cell);
            Ok(lease.into_owned_partition())
        })
        .collect::<Result<Vec<_>, RelationalRecoveryStop>>()?;
    let kinds =
        worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
            worth_relational_storage::internal_port::storage::residency::checkpoint_kind_images::CheckpointRegionKindImages::capture(
                lane, execution, &admitted,
            )
        })?;
    let images = admitted
        .into_iter()
        .map(|partition| {
            worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
                |lane| {
                    partition_to_image_with_contracts(
                        lane,
                        partition,
                        schema.aspect_plans(),
                        &contracts,
                    )
                },
            )
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
