//! Native capture selects cells and leaf facts, never reconstructive images.
use std::collections::BTreeMap;

use super::checkpoint_capture::CapturedCheckpointBasis;
use crate::durability::data::*;
use crate::durability::log::native_checkpoint_codec::CapturedManifest;

impl CapturedCheckpointBasis {
    pub(super) fn build_native_capture(
        mut self,
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
        let mut lineage_basis = prior
            .as_ref()
            .map(|prior| prior.manifest.lineage_basis.clone());
        if let Some(prior) = &prior {
            let symbol_frontier = prior
                .manifest
                .symbols
                .iter()
                .map(|page| page.last)
                .max()
                .unwrap_or(0);
            if carry_metadata {
                self.symbol_table
                    .entries
                    .retain(|(symbol, _)| symbol.0 as u64 > symbol_frontier);
            }
            let lineage_frontier = prior
                .manifest
                .lineage
                .iter()
                .map(|page| page.last)
                .max()
                .unwrap_or(0);
            if carry_metadata {
                self.lineage_nodes
                    .retain(|node| node.lineage_id.0 > lineage_frontier);
            }
            let basis = lineage_basis
                .as_mut()
                .expect("selected directory supplies its lineage basis");
            for commit in self
                .envelopes
                .iter()
                .filter(|commit| commit.position() > prior.manifest.covered_position)
            {
                basis.published_lineage_commit_count += usize::from(commit.has_lineage_authority());
                basis.canonical_published_event_ids.extend(
                    commit
                        .lineage_digest_basis()
                        .canonical_event_ids()
                        .iter()
                        .copied(),
                );
                basis.published_lineage_event_count +=
                    commit.lineage_digest_basis().lineage_event_count();
                basis.published_lineage_decision_count +=
                    commit.lineage_digest_basis().lineage_decision_count();
            }
        }
        let carried_index_image = self.carried_index_image;
        let roots = std::mem::take(&mut self.branch_roots);
        let mirror = std::mem::take(&mut self.partitions);
        let mirror_schema = self.mirror_schema_authority.clone();
        // With storage sources removed, this constructs metadata only. The
        // memory checkpoint door alone materializes its selected substrate.
        let metadata = self.build_checkpoint_image()?;
        let mut captured = CapturedManifest::capture_metadata(
            &metadata,
            policy,
            prior.as_ref().map(|prior| &prior.manifest),
            placement,
        )?;
        captured.cold_source = placement
            .publication_source()
            .or_else(|| prior.as_ref().map(|prior| prior.source.clone()))
            .or(source);
        if placement.is_embedded() {
            captured.carried_envelopes = prior
                .as_ref()
                .map_or_else(Vec::new, |prior| prior.manifest.commits.clone());
        }
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
        let mut schemas = BTreeMap::new();
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
                let capture = region
                    .cell
                    .capture_checkpoint(schema, schema_digest, &symbols, current_members.as_ref())
                    .map_err(storage_failure)?;
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
                .map_err(storage_failure)?;
                leaves.push(capture.leaf);
                captured.publications.push(capture.publication);
                for member in capture.members {
                    insert_member(&mut captured, member);
                }
            }
            captured.manifest.roots.push(RelationalCheckpointRoot {
                commit_id: root.commit_id(),
                schema_carrier_digest: schema_digest,
                regions: leaves,
            });
        }
        if let Some(schema) = mirror_schema {
            let image = DurableBranchRootSchemaImage::capture(&schema)
                .map_err(|error| corrupt(error.to_string()))?;
            let digest = image.carrier_digest();
            schemas.entry(digest).or_insert(image);
            for (_, region) in mirror.cells() {
                policy.check_live()?;
                let capture = region
                    .cell()
                    .capture_checkpoint(&schema, digest, &symbols, current_members.as_ref())
                    .map_err(storage_failure)?;
                merge_history_dependencies(
                    &mut history_dependencies,
                    capture.history_dependencies,
                    capture.leaf.partition_id,
                )
                .map_err(storage_failure)?;
                captured.manifest.mirror.push(capture.leaf);
                captured.publications.push(capture.publication);
                for member in capture.members {
                    insert_member(&mut captured, member);
                }
            }
        } else if mirror.len() != 0 {
            return Err(corrupt("nonempty mirror has no selected schema authority").into());
        }
        if let Some(prior) = &prior {
            if carry_metadata {
                let mut symbols = prior.manifest.symbols.clone();
                symbols.append(&mut captured.manifest.symbols);
                captured.manifest.symbols = symbols;
                let mut buckets = prior
                    .manifest
                    .symbol_buckets
                    .iter()
                    .map(|bucket| (bucket.bucket, bucket.pages.clone()))
                    .collect::<BTreeMap<_, _>>();
                for bucket in std::mem::take(&mut captured.manifest.symbol_buckets) {
                    buckets
                        .entry(bucket.bucket)
                        .or_default()
                        .extend(bucket.pages);
                }
                captured.manifest.symbol_buckets = buckets
                    .into_iter()
                    .map(|(bucket, pages)| RelationalCheckpointSymbolBucket { bucket, pages })
                    .collect();
                for bucket in &prior.manifest.symbol_buckets {
                    for locator in &bucket.pages {
                        insert_member(
                            &mut captured,
                            RelationalCheckpointMember::Carried { locator: *locator },
                        );
                    }
                }
                let mut lineage = prior.manifest.lineage.clone();
                lineage.append(&mut captured.manifest.lineage);
                captured.manifest.lineage = lineage;
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
            captured.manifest.lineage_basis = lineage_basis.expect("selected lineage basis");
            // The full commit index, rather than resident head envelopes,
            // determines coverage even when its authoring branch was retired.
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
        }
        captured.manifest.schemas = schemas.into_values().collect();
        captured.manifest.history_dependencies = history_dependencies
            .into_iter()
            .map(|(member, bases)| RelationalCheckpointHistoryDependency { member, bases })
            .collect();
        captured.history_members = captured
            .manifest
            .readmit_history_members()
            .map_err(|cause| corrupt(format!("captured history dependencies: {cause:?}")))?;
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

fn storage_failure(failure: crate::storage::data::RelationalStorageUnavailable) -> DurabilityError {
    let RelationalRecoveryStop::Durability(error) = RelationalRecoveryStop::from(failure) else {
        unreachable!("storage unavailability is a durable-store refusal")
    };
    error
}

fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
