use super::*;

impl IndexAuthority<'_> {
    pub(super) fn prepare_refresh(
        &self,
        request: &DerivedIndexBuildRequest,
        before: Option<&VisibilityProjectionView<'_>>,
        after: &VisibilityProjectionView<'_>,
        work: &mut MaintenanceWork,
    ) -> Result<PreparedRefresh, DerivedIndexMaintenanceDenialKind> {
        let root = after.selected_root().expect("validated exact root");
        let schema_version = root.schema_authority().schema_version();
        let authoring = root
            .canonical_envelope()
            .map(|envelope| &envelope.branch_context);
        let mut prepared = Vec::new();
        let mut pending_fields = Vec::new();
        let mut reused = Vec::new();
        let mut cold_changes = None;
        let mut patch_changes = None;
        let mut cold_routing = None;
        let mut patch_routing = None;
        let mut requested = std::collections::BTreeSet::new();
        for index_id in &request.index_ids {
            work.ordered::<DerivedIndexId, ()>(requested.len(), 1, 0)?;
            requested.insert(*index_id);
        }
        for index_id in requested {
            let (definition, admitted_generation) = if work.has_preparation() {
                self.runtime
                    .indexes
                    .field_maintenance_inputs_admitted(
                        index_id,
                        &request.branch_id,
                        request.source_commit_id,
                        after.version_id(),
                        |units, bytes| work.prepare(units, bytes),
                    )
                    .map_err(|stop| match stop {
                        SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                        SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                            DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
                        }
                    })?
                    .ok_or(DerivedIndexMaintenanceDenialKind::IndexUnavailable(
                        index_id,
                    ))?
            } else {
                (
                    self.runtime.indexes.definition(index_id).ok_or(
                        DerivedIndexMaintenanceDenialKind::IndexUnavailable(index_id),
                    )?,
                    None,
                )
            };
            if work.has_preparation()
                && !matches!(
                    &definition.kind,
                    DerivedIndexKind::EntityField { .. } | DerivedIndexKind::RelationField { .. }
                )
            {
                return Err(DerivedIndexMaintenanceDenialKind::GenerationKindMismatch(
                    index_id,
                ));
            }
            // A global index is current from any branch's generation at this
            // exact commit and version; a scoped one only from its own.
            let current = if work.has_preparation() {
                admitted_generation
            } else {
                self.runtime.indexes.published_generation_for_commit(
                    index_id,
                    definition.branch_scoped.then_some(&request.branch_id),
                    request.source_commit_id,
                    after.version_id(),
                )
            };
            if let Some(generation) = current {
                if generation.applicability.schema_version == schema_version {
                    work.grow_vec(&mut reused)?;
                    let initialized = generation
                        .source_branch_id
                        .0
                        .len()
                        .checked_add(generation.applicability.branch_id.0.len())
                        .ok_or(DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded)?;
                    let backing = generation
                        .source_branch_id
                        .0
                        .capacity()
                        .checked_add(generation.applicability.branch_id.0.capacity())
                        .ok_or(DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded)?;
                    work.prepare(initialized as u64 + 1, backing as u64)?;
                    reused.push(generation.as_ref().clone());
                    work.counts.reused_generations += 1;
                    continue;
                }
            }
            if let Some(seed) = (!work.has_preparation())
                .then(|| {
                    self.fork_seed(
                        index_id,
                        request,
                        authoring,
                        after.version_id(),
                        schema_version,
                    )
                })
                .flatten()
            {
                work.grow_vec(&mut prepared)?;
                work.prepare(1, 0)?;
                prepared.push((index_id, seed.entries.clone()));
                work.counts.seeded_generations += 1;
                continue;
            }
            let prior = before
                .filter(|view| {
                    view.selected_schema_authority().map(|s| s.schema_version())
                        == Some(schema_version)
                })
                .and_then(|view| {
                    self.runtime
                        .indexes
                        .exact_generation(
                            index_id,
                            Some(&request.branch_id),
                            view.version_id(),
                            schema_version,
                        )
                        .filter(|generation| {
                            view.selected_root().and_then(|root| root.commit_id())
                                == Some(generation.source_commit_id)
                        })
                });
            let (entries, changes, old) = if let Some(prior) = prior {
                if patch_changes.is_none() {
                    patch_changes = Some(changes::ChangedRecords::patch(root, work)?);
                }
                (
                    prior.entries.clone(),
                    patch_changes.as_ref().unwrap(),
                    before,
                )
            } else {
                if cold_changes.is_none() {
                    cold_changes = Some(changes::ChangedRecords::cold(after, work)?);
                }
                match &definition.kind {
                    DerivedIndexKind::EntityField { .. } => entry_edits::prepare_map::<
                        crate::storage::data::AuthoritativeFieldComparisonKey,
                        crate::identity::data::EntityId,
                    >(0, 1, work)?,
                    DerivedIndexKind::RelationField { .. } => entry_edits::prepare_map::<
                        crate::storage::data::AuthoritativeFieldComparisonKey,
                        crate::identity::data::RelationId,
                    >(0, 1, work)?,
                    _ => {}
                }
                (
                    empty_entries(&definition.kind),
                    cold_changes.as_ref().unwrap(),
                    None,
                )
            };
            match (&definition.kind, entries) {
                (
                    DerivedIndexKind::EntityField { field_locator },
                    DerivedIndexEntries::EntityField(entries),
                ) => {
                    work.grow_vec(&mut pending_fields)?;
                    work.locator(field_locator)?;
                    pending_fields.push(field::PendingField::Entity {
                        index_id,
                        locator: field_locator.clone(),
                        entries,
                        edits: Default::default(),
                        patch: old.is_some(),
                    });
                }
                (
                    DerivedIndexKind::RelationField { field_locator },
                    DerivedIndexEntries::RelationField(entries),
                ) => {
                    work.grow_vec(&mut pending_fields)?;
                    work.locator(field_locator)?;
                    pending_fields.push(field::PendingField::Relation {
                        index_id,
                        locator: field_locator.clone(),
                        entries,
                        edits: Default::default(),
                        patch: old.is_some(),
                    });
                }
                (_, mut entries) => {
                    let routing = if old.is_some() {
                        &mut patch_routing
                    } else {
                        &mut cold_routing
                    };
                    if routing.is_none() {
                        *routing = Some(change_routing::ChangeRouting::classify(
                            changes, old, after, work,
                        )?);
                    }
                    update_entries(
                        &definition,
                        &mut entries,
                        routing.as_ref().unwrap(),
                        old,
                        after,
                        work,
                    )?;
                    work.grow_vec(&mut prepared)?;
                    prepared.push((index_id, entries));
                }
            }
        }
        field::refresh(
            &mut pending_fields,
            patch_changes.as_ref(),
            cold_changes.as_ref(),
            before,
            after,
            work,
        )?;
        for field in pending_fields {
            work.grow_vec(&mut prepared)?;
            work.prepare(1, 0)?;
            prepared.push(field.into_prepared());
        }
        work.prepare(
            request.branch_id.0.len() as u64,
            request.branch_id.0.capacity() as u64,
        )?;
        let publication = IndexGenerationPublicationBasis::new(
            request,
            request.branch_id.clone(),
            after.version_id(),
            schema_version,
        );
        Ok(PreparedRefresh {
            publication,
            entries: prepared,
            reused,
        })
    }
}
