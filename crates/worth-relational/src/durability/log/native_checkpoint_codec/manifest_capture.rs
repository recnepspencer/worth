mod seal;
use std::collections::BTreeMap;
use std::sync::Arc;

use super::member_frame::{self, MemberKind};
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_from_image;
use crate::durability::data::*;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};

mod expanded_regions;
use expanded_regions::{capture_partitions, restore_images};

pub(crate) struct CapturedManifest<M = RelationalCheckpointManifestDraft> {
    pub(crate) component_holds: Vec<crate::branch::AdmittedRelationalBranchBasis>,
    pub(crate) manifest: M,
    pub(crate) members: BTreeMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    pub(crate) canonical_publication:
        Option<crate::durability::checkpoint_history::publication::CanonicalCheckpointPublication>,
    pub(crate) canonical_payloads:
        Option<worth_execution::ExecutionArray<RelationalCheckpointMember>>,
    pub(crate) envelope_cells: worth_execution::ExecutionOrderedMap<
        crate::publication::patch::data::PatchStreamPosition,
        Arc<crate::history::RelationalCommitEnvelopeResidency>,
    >,
    pub(crate) publications: Vec<crate::storage::residency::RelationalCheckpointRegionPublication>,
    pub(crate) cold_source: Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
    pub(crate) guarded_members: BTreeMap<
        RelationalDurableMemberLocator,
        (
            crate::storage::data::RelationalStorageSubject,
            Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        ),
    >,
    pub(crate) history_members: Arc<RelationalHistoryMemberDirectory>,
}

impl CapturedManifest {
    pub(crate) fn capture_metadata(
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        checkpoint: &DurableCheckpoint,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        prior: Option<&RelationalCheckpointManifest>,
        journal_index: Vec<crate::history::RelationalCommitIndexEntry>,
        control: &mut super::CheckpointEncodingControl<'_, '_, '_, '_, '_>,
        original_digest: impl Fn(
            crate::publication::patch::data::PatchStreamPosition,
        ) -> Result<[u8; 32], RelationalNativeCheckpointCaptureDenial>,
    ) -> Result<Self, RelationalNativeCheckpointCaptureDenial> {
        let mut members = BTreeMap::new();
        let mut commits = BTreeMap::new();
        for entry in prior
            .into_iter()
            .flat_map(|prior| prior.commits.iter().cloned())
            .chain(journal_index)
        {
            if let Some(existing) = commits.insert(entry.position, entry.clone())
                && existing != entry
            {
                return Err(
                    corrupt("checkpoint commit index has competing placement facts").into(),
                );
            }
        }
        for positioned in &checkpoint.envelopes {
            control.checkpoint(0)?;
            let envelope = positioned.envelope();
            let root = checkpoint
                .commit_descriptors
                .get(&envelope.commit.commit_id)
                .ok_or_else(|| corrupt("checkpoint commit lacks its catalog descriptor"))?;
            if let Some(existing) = commits.get(&positioned.position()) {
                if existing.commit != envelope.commit
                    || existing.association_discovery != envelope.association_discovery()
                    || &existing.root != root
                    || existing.schema_version != envelope.schema_version
                {
                    return Err(
                        corrupt("resident envelope disagrees with its admitted index").into(),
                    );
                }
            } else {
                let payload_digest = original_digest(positioned.position())?;
                commits.insert(
                    positioned.position(),
                    crate::history::RelationalCommitIndexEntry {
                        association_discovery: envelope.association_discovery(),
                        commit: envelope.commit.clone(),
                        position: positioned.position(),
                        root: root.clone(),
                        payload_digest,
                        schema_version: envelope.schema_version,
                    },
                );
            }
        }
        let commits = commits.into_values().collect::<Vec<_>>();
        let dictionary = super::dictionary_capture::capture(
            lane,
            &checkpoint.symbol_table,
            policy,
            &mut members,
        )?;
        let symbol_pages = dictionary.by_id;
        let symbol_buckets = dictionary.by_text;
        let mut lineage_pages = Vec::new();
        for page in checkpoint.lineage.nodes().chunk_by(|a, b| {
            RelationalCheckpointPage::range_of(a.lineage_id.0)
                == RelationalCheckpointPage::range_of(b.lineage_id.0)
        }) {
            let member = add(
                &mut members,
                member_frame::encode(MemberKind::Lineage, &page)
                    .map_err(|cause| corrupt(format!("lineage member: {cause:?}")))?,
            );
            lineage_pages.push(
                RelationalCheckpointPage::from_ids(
                    page.iter().map(|node| node.lineage_id.0),
                    member,
                )
                .ok_or_else(|| corrupt("invalid captured lineage page membership"))?,
            );
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
            component_holds: Vec::new(),
            envelope_cells: Default::default(),
            manifest: RelationalCheckpointManifestDraft {
                coverage: checkpoint.coverage.clone(),
                covered_position: commits.last().map_or(
                    crate::publication::patch::data::PatchStreamPosition(0),
                    |entry| entry.position,
                ),
                branch_data: crate::branch::CheckpointBranchMetadata::from_ordinary_parts(
                    checkpoint.runtime_name.clone(),
                    checkpoint.branch_cells.clone(),
                    checkpoint.branch_lifecycle.clone(),
                    checkpoint.retired_branch_names.clone(),
                ),
                schemas: CheckpointSchemaImages::from_ordinary(
                    checkpoint.branch_root_schema_images.clone(),
                ),
                record_identity: checkpoint.record_identity.clone(),
                aspect_contracts: checkpoint.aspect_contracts.clone(),
                index_definitions: CheckpointIndexDefinitions::from_ordinary(
                    checkpoint.index_definitions.clone(),
                ),
                index_generation_frontier: checkpoint.index_generation_frontier,
                index_definition_frontier: checkpoint.index_definition_frontier,
                roots: CheckpointRoots::default(),
                retained_empty_components: checkpoint.retained_empty_components.clone(),
                mirror_schema: checkpoint.mirror_schema_digest,
                mirror: Vec::new(),
                history_dependencies: Vec::new(),
                commits: CheckpointCommitIndex::from_ordinary(commits),
                canonical_members: Default::default(),
                symbols: CheckpointPages::from_ordinary(symbol_pages),
                symbol_identity_frontier: checkpoint.symbol_identity_frontier,
                symbol_buckets: CheckpointSymbolBuckets::from_ordinary(symbol_buckets),
                lineage: CheckpointPages::from_ordinary(lineage_pages),
                lineage_events: Default::default(),
                lineage_basis: CheckpointLineageBasis::from_ordinary(
                    checkpoint.lineage.digest_basis().clone(),
                ),
                derived_indexes,
                allocations: Default::default(),
            },
            members,
            canonical_payloads: None,
            canonical_publication: None,
            publications: Vec::new(),
            cold_source: None,
            guarded_members: BTreeMap::new(),
            history_members: Arc::default(),
        })
    }
    #[cfg(test)]
    pub(super) fn capture(
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: &DurableCheckpoint,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, RelationalNativeCheckpointCaptureDenial> {
        Self::capture_expanded(
            lane,
            execution,
            checkpoint,
            policy,
            &mut super::checkpoint_encoding::CheckpointEncodingControl::Policy(policy),
        )
    }
    pub(crate) fn capture_expanded(
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: &DurableCheckpoint,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        control: &mut super::checkpoint_encoding::CheckpointEncodingControl<'_, '_, '_, '_, '_>,
    ) -> Result<Self, RelationalNativeCheckpointCaptureDenial> {
        let mut captured = Self::capture_metadata(
            lane,
            checkpoint,
            policy,
            None,
            Vec::new(),
            control,
            |position| {
                let positioned = checkpoint
                    .envelopes
                    .iter()
                    .find(|commit| commit.position() == position)
                    .ok_or_else(|| corrupt("fixture canonical position unavailable"))?;
                RelationalCanonicalCommitArtifact::positioned_payload_digest(positioned)
                    .map_err(|error| corrupt(format!("fixture canonical digest: {error}")).into())
            },
        )?;
        let rows = checkpoint
            .envelopes
            .iter()
            .flat_map(|commit| {
                commit
                    .lineage_events()
                    .iter()
                    .map(|event| RelationalCheckpointLineageEventRow {
                        event: Arc::new(event.clone()),
                        publication_commit_id: commit.commit.commit_id,
                    })
            })
            .collect();
        captured.append_lineage_events(rows, policy)?;
        let mut members = std::mem::take(&mut captured.members);
        let mut symbols = crate::symbols::data::StringInterner::default();
        symbols.restore_snapshot(lane, checkpoint.symbol_table.clone());
        let mut roots = Vec::new();
        let mut schemas = BTreeMap::new();
        for image in &checkpoint.branch_root_schema_images {
            let (registry, contracts) = image.readmit_registry().map_err(corrupt)?;
            let expected = registry
                .snapshot_for_compilation(crate::schema::SchemaCompilationControl::Request(
                    *execution,
                ))
                .map_err(RelationalNativeCheckpointCaptureDenial::Request)?;
            let schema = crate::branch::RelationalBranchRootSchemaAuthority::readmit_exact(
                schemas.len() as u64 + 1,
                registry,
                contracts,
                &expected,
                image.descriptor_semantics_version(),
                crate::schema::SchemaCompilationControl::Request(*execution),
            )
            .map_err(RelationalNativeCheckpointCaptureDenial::Request)?
            .ok_or_else(|| corrupt("schema carrier readmission failed"))?;
            schemas.insert(image.carrier_digest(), schema);
        }
        for root in &checkpoint.branch_roots {
            let schema = schemas
                .get(&root.schema_carrier_digest)
                .ok_or_else(|| corrupt("missing root schema carrier"))?;
            let partitions = restore_images(
                lane,
                execution,
                &root.partition_images,
                schema,
                checkpoint
                    .root_kind_images
                    .get(&root.commit_id)
                    .ok_or_else(|| corrupt("missing root kind images"))?,
            )?;
            let regions = capture_partitions(
                lane,
                execution,
                &partitions,
                schema,
                root.schema_carrier_digest,
                &symbols,
                &mut members,
            )?;
            roots.push(RelationalCheckpointRoot::from_ordinary_parts(
                root.commit_id,
                checkpoint
                    .recorded_components
                    .get(&root.commit_id)
                    .cloned()
                    .ok_or_else(|| corrupt("root lacks component observations"))?,
                root.schema_carrier_digest,
                regions,
                root.content_records
                    .records()
                    .map(|record| {
                        let member = add(
                            &mut members,
                            (
                                member_frame::locator(record.bytes()),
                                record.bytes().to_vec(),
                            ),
                        );
                        RelationalCheckpointContent {
                            identity: record.identity(),
                            member,
                        }
                    })
                    .collect(),
            ));
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
                lane,
                execution,
                &restore_images(
                    lane,
                    execution,
                    &checkpoint.partition_images,
                    schema,
                    &checkpoint.mirror_kind_images,
                )?,
                schema,
                digest,
                &symbols,
                &mut members,
            )?
        };
        captured.manifest.roots = CheckpointRoots::from_ordinary(roots);
        captured.manifest.mirror = mirror;
        captured.members = members;
        Ok(captured)
    }
}

pub(super) fn add(
    members: &mut BTreeMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    (locator, bytes): (RelationalDurableMemberLocator, Vec<u8>),
) -> RelationalDurableMemberLocator {
    members
        .entry(locator)
        .or_insert_with(|| RelationalCheckpointMember::New {
            locator,
            bytes: worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
                bytes.into_boxed_slice(),
            )),
        });
    locator
}

pub(super) fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
