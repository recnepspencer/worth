use super::*;

impl IndexAuthority<'_> {
    pub(super) fn prepare_refresh(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &DerivedIndexBuildRequest,
        before: Option<&VisibilityProjectionView>,
        after: &VisibilityProjectionView,
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
            let compatible_before = before.filter(|view| {
                view.selected_schema_authority()
                    .map(|schema| schema.schema_version())
                    == Some(schema_version)
            });
            let (definition, current, prior) = self
                .runtime
                .indexes
                .maintenance_inputs_admitted(
                    index_id,
                    &request.branch_id,
                    request.source_commit_id,
                    after.version_id(),
                    compatible_before.map(|view| (view.version_id(), schema_version)),
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
                ))?;
            if work.field_indexes_only()
                && !matches!(
                    &definition.kind,
                    DerivedIndexKind::EntityField { .. } | DerivedIndexKind::RelationField { .. }
                )
            {
                return Err(DerivedIndexMaintenanceDenialKind::GenerationKindMismatch(
                    index_id,
                ));
            }
            if let Some(generation) = current
                && generation.applicability.schema_version == schema_version
            {
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
                work.note_reused_generations(1);
                continue;
            }
            if let Some(seed) = self.fork_seed_admitted(
                index_id,
                request,
                authoring,
                after.version_id(),
                schema_version,
                work,
            )? {
                work.grow_vec(&mut prepared)?;
                work.prepare(1, 0)?;
                prepared.push((index_id, seed.entries.clone()));
                work.note_seeded_generations(1);
                continue;
            }
            let prior = prior.filter(|generation| {
                compatible_before
                    .and_then(|view| view.selected_root().and_then(|root| root.commit_id()))
                    == Some(generation.source_commit_id)
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
                if work.prohibits_population_reconstruction() {
                    return Err(DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired);
                }
                if cold_changes.is_none() {
                    cold_changes = Some(changes::ChangedRecords::cold(after, work)?);
                }
                match &definition.kind {
                    DerivedIndexKind::EntityField { .. } => {
                        super::persistent_allocation::prepare_empty_map::<
                            crate::storage::data::AuthoritativeFieldComparisonKey,
                            crate::identity::data::EntityId,
                        >(work)?
                    }
                    DerivedIndexKind::RelationField { .. } => {
                        super::persistent_allocation::prepare_empty_map::<
                            crate::storage::data::AuthoritativeFieldComparisonKey,
                            crate::identity::data::RelationId,
                        >(work)?
                    }
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
                            execution, changes, old, after, work,
                        )?);
                    }
                    update_entries(
                        execution,
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
            execution,
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
