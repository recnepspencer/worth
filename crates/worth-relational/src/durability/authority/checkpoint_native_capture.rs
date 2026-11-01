//! Native capture selects cells, consolidating history only at its declared bound.
use std::collections::BTreeMap;

use super::checkpoint_capture::CapturedCheckpointBasis;
use crate::durability::data::*;
use crate::durability::log::native_checkpoint_codec::CapturedManifest;

impl CapturedCheckpointBasis {
    pub(super) fn build_native_capture(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        source: Option<std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
        placement: &RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<CapturedManifest, RelationalNativeCheckpointCaptureDenial> {
        let mut symbols = crate::symbols::data::StringInterner::default();
        symbols.restore_snapshot(self.symbol_table.clone());
        let prior = self.selected_directory.take();
        let current_members = placement.current_members();
        let carry_metadata = super::checkpoint_member_carry::may_carry_metadata(
            &self,
            prior.as_ref().map(|prior| &prior.manifest),
            current_members.as_ref(),
        )?;
        if carry_metadata && let Some(prior) = &prior {
            self.symbol_table.entries.retain(|(symbol, _)| {
                RelationalCheckpointPage::find(&prior.manifest.symbols, u64::from(symbol.0))
                    .is_none()
            });
            self.lineage_nodes.retain(|node| {
                RelationalCheckpointPage::find(&prior.manifest.lineage, node.lineage_id.0).is_none()
            });
        }
        let resident_event_rows = if placement.is_embedded() {
            self.lineage_events
                .iter()
                .map(|row| (row.event.event_id(), row.clone()))
                .collect()
        } else {
            BTreeMap::new()
        };
        let mut event_rows = std::mem::take(&mut self.lineage_events);
        if carry_metadata && let Some(prior) = &prior {
            let covered = prior
                .manifest
                .commits
                .iter()
                .map(|entry| entry.commit.commit_id)
                .collect::<std::collections::BTreeSet<_>>();
            event_rows.retain(|row| !covered.contains(&row.publication_commit_id));
        }
        let guarded_member_sources = std::mem::take(&mut self.guarded_member_sources);
        let _lineage_event_leases = std::mem::take(&mut self.lineage_event_leases);
        let carried_index_image = self.carried_index_image;
        let roots = std::mem::take(&mut self.branch_roots);
        let recorded_components = self.recorded_components.clone();
        let empty_component_schemas = self.empty_component_schemas.clone();
        let component_holds = std::mem::take(&mut self.component_holds);
        let mirror = std::mem::take(&mut self.partitions);
        let mirror_schema = self.mirror_schema_authority.clone();
        // With storage sources removed, this constructs metadata only. The
        // memory checkpoint door alone materializes its selected substrate.
        let journal_index = std::mem::take(&mut self.journal_index);
        let envelope_cells = std::mem::take(&mut self.envelope_cells);
        let mut captured_cells = worth_execution::ExecutionOrderedMap::default();
        for cell in envelope_cells {
            if let Some(position) = cell.captured_position() {
                captured_cells
                    .insert_cloned_native_for_request(&position, &cell, *execution)
                    .map_err(|cause| {
                        crate::durability::checkpoint_history::capture::capture_stop(
                            crate::durability::checkpoint_history::capture::map_stop(cause),
                        )
                    })?;
            }
        }
        let metadata = self.build_checkpoint_image()?;
        let mut captured = CapturedManifest::capture_metadata(
            &metadata,
            policy,
            prior.as_ref().map(|prior| &prior.manifest),
            journal_index,
            |position| {
                captured_cells
                    .get(&position)
                    .ok_or_else(|| corrupt("original Memory canonical cell unavailable"))?
                    .original_memory_payload_digest()
                    .map_err(Into::into)
            },
        )?;
        captured.component_holds = component_holds;
        captured.append_lineage_events(event_rows, policy)?;
        captured.envelope_cells = captured_cells;
        captured.cold_source = placement
            .publication_source()
            .or_else(|| prior.as_ref().map(|prior| prior.source.clone()))
            .or(source);
        if let Some(locator) = carried_index_image {
            if current_members
                .as_ref()
                .is_some_and(|members| !members.contains(&locator))
            {
                return Err(super::checkpoint_member_carry::unavailable(
                    crate::storage::data::RelationalStorageSubject::Index(
                        crate::indexes::data::DerivedIndexId(0),
                    ),
                )
                .into());
            }
            captured.members.remove(&captured.manifest.derived_indexes);
            captured.manifest.derived_indexes = locator;
            insert_member(
                &mut captured,
                RelationalCheckpointMember::Carried { locator },
            );
        }
        let mut schemas = empty_component_schemas
            .into_iter()
            .map(|image| (image.carrier_digest(), image))
            .collect::<BTreeMap<_, _>>();
        let mut history_dependencies = BTreeMap::new();
        for root in roots {
            policy.check_live()?;
            let schema = root.schema_authority();
            let image = DurableBranchRootSchemaImage::capture(schema)
                .map_err(|error| corrupt(error.to_string()))?;
            let schema_digest = image.carrier_digest();
            schemas.entry(schema_digest).or_insert(image);
            let mut leaves = Vec::new();
            for region in root.regions() {
                let capture = region.cell.capture_checkpoint(
                    execution,
                    schema,
                    schema_digest,
                    &symbols,
                    current_members.as_ref(),
                )?;
                if capture.leaf.partition_id != region.partition_id
                    || capture.leaf.content_digest != region.content_digest
                    || capture.leaf.entity_slots != region.entity_slots
                    || capture.leaf.relation_slots != region.relation_slots
                    || capture.leaf.unavailable_records != region.unavailable_records
                    || capture.leaf.kind_authority != region.kind_commitment
                {
                    return Err(
                        corrupt("captured region differs from its selected root leaf").into(),
                    );
                }
                merge_history_dependencies(
                    &mut history_dependencies,
                    capture.history_dependencies,
                    capture.leaf.partition_id,
                )
                .map_err(DurabilityError::from)?;
                leaves.push(capture.leaf);
                captured.publications.push(capture.publication);
                captured.guarded_members.extend(
                    region
                        .cell
                        .checkpoint_export_sources(&capture.members, captured.cold_source.clone())
                        .map_err(DurabilityError::from)?,
                );
                for member in capture.members {
                    insert_member(&mut captured, member);
                }
            }
            captured
                .manifest
                .roots
                .push_ordinary(RelationalCheckpointRoot::from_ordinary_parts(
                    root.commit_id(),
                    recorded_components
                        .get(&root.commit_id())
                        .cloned()
                        .ok_or_else(|| corrupt("root lacks exact component observations"))?,
                    schema_digest,
                    leaves,
                ));
        }
        if let Some(schema) = mirror_schema {
            let image = DurableBranchRootSchemaImage::capture(&schema)
                .map_err(|error| corrupt(error.to_string()))?;
            let digest = image.carrier_digest();
            schemas.entry(digest).or_insert(image);
            for (_, region) in mirror.cells() {
                policy.check_live()?;
                let capture = region.cell().capture_checkpoint(
                    execution,
                    &schema,
                    digest,
                    &symbols,
                    current_members.as_ref(),
                )?;
                merge_history_dependencies(
                    &mut history_dependencies,
                    capture.history_dependencies,
                    capture.leaf.partition_id,
                )
                .map_err(DurabilityError::from)?;
                captured.manifest.mirror.push(capture.leaf);
                captured.publications.push(capture.publication);
                captured.guarded_members.extend(
                    region
                        .cell()
                        .checkpoint_export_sources(&capture.members, captured.cold_source.clone())
                        .map_err(DurabilityError::from)?,
                );
                for member in capture.members {
                    insert_member(&mut captured, member);
                }
            }
        } else if mirror.len() != 0 {
            return Err(corrupt("nonempty mirror has no selected schema authority").into());
        }
        if let Some(prior) = &prior
            && carry_metadata
        {
            let mut symbols = prior.manifest.symbols.ordinary_owned_copy();
            symbols.extend(captured.manifest.symbols.ordinary_owned_copy());
            symbols.sort_by_key(|page| (page.range(), page.first));
            captured.manifest.symbols = CheckpointPages::from_ordinary(symbols);
            let mut buckets = prior
                .manifest
                .symbol_buckets
                .iter()
                .map(|bucket| (bucket.bucket, bucket.pages.clone()))
                .collect::<BTreeMap<_, _>>();
            for bucket in captured.manifest.symbol_buckets.ordinary_owned_copy() {
                buckets
                    .entry(bucket.bucket)
                    .or_default()
                    .extend(bucket.pages);
            }
            captured.manifest.symbol_buckets = CheckpointSymbolBuckets::from_ordinary(
                buckets
                    .into_iter()
                    .map(|(bucket, pages)| RelationalCheckpointSymbolBucket { bucket, pages })
                    .collect(),
            );
            for bucket in &prior.manifest.symbol_buckets {
                for locator in &bucket.pages {
                    insert_member(
                        &mut captured,
                        RelationalCheckpointMember::Carried { locator: *locator },
                    );
                }
            }
            let mut event_pages = prior.manifest.lineage_events.ordinary_owned_copy();
            event_pages.extend(captured.manifest.lineage_events.ordinary_owned_copy());
            captured.manifest.lineage_events =
                CheckpointLineageEventPages::from_ordinary(event_pages);
            for page in &prior.manifest.lineage_events {
                insert_member(
                    &mut captured,
                    RelationalCheckpointMember::Carried {
                        locator: page.member,
                    },
                );
            }
            let mut lineage = prior.manifest.lineage.ordinary_owned_copy();
            lineage.extend(captured.manifest.lineage.ordinary_owned_copy());
            lineage.sort_by_key(|page| (page.range(), page.first));
            captured.manifest.lineage = CheckpointPages::from_ordinary(lineage);
            for page in prior.manifest.symbols.iter().chain(&prior.manifest.lineage) {
                insert_member(
                    &mut captured,
                    RelationalCheckpointMember::Carried {
                        locator: page.member,
                    },
                );
            }
            if captured.manifest.derived_indexes == prior.manifest.derived_indexes {
                insert_member(
                    &mut captured,
                    RelationalCheckpointMember::Carried {
                        locator: prior.manifest.derived_indexes,
                    },
                );
            }
        }
        captured.guarded_members.extend(guarded_member_sources);
        if placement.is_embedded() {
            captured.materialize_resident_event_members(&resident_event_rows, policy)?;
        }
        captured.manifest.coverage.up_to_commit = captured
            .manifest
            .commits
            .last()
            .map(|entry| entry.commit.clone());
        captured.manifest.coverage.up_to_version = captured
            .manifest
            .commits
            .last()
            .map(|entry| entry.commit.version_id);
        captured.manifest.schemas =
            CheckpointSchemaImages::from_ordinary(schemas.into_values().collect());
        captured.manifest.history_dependencies = history_dependencies
            .into_iter()
            .map(|(member, bases)| RelationalCheckpointHistoryDependency { member, bases })
            .collect();
        captured.history_members = captured
            .manifest
            .readmit_history_members()
            .map_err(|cause| corrupt(format!("captured history dependencies: {cause:?}")))?;
        captured.validate_lineage_events()?;
        crate::durability::checkpoint_history::capture::capture(&mut captured, execution)?;
        Ok(captured)
    }
}

fn merge_history_dependencies(
    declarations: &mut BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        Vec<crate::durability::medium::RelationalDurableMemberLocator>,
    >,
    incoming: Vec<RelationalCheckpointHistoryDependency>,
    partition: crate::identity::data::PartitionId,
) -> Result<(), crate::storage::data::RelationalStorageUnavailable> {
    for declaration in incoming {
        if let Some(prior) = declarations.get(&declaration.member) {
            if prior != &declaration.bases {
                return Err(crate::storage::data::RelationalStorageUnavailable {
                    subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                    cause:
                        crate::storage::data::RelationalStorageUnavailableCause::IntegrityMismatch,
                });
            }
        } else {
            declarations.insert(declaration.member, declaration.bases);
        }
    }
    Ok(())
}

fn insert_member(captured: &mut CapturedManifest, member: RelationalCheckpointMember) {
    let key = member.locator();
    if matches!(member, RelationalCheckpointMember::New { .. }) {
        captured.members.insert(key, member);
    } else {
        captured.members.entry(key).or_insert(member);
    }
}

fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
