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
        let root = after.selected_root().expect(/* Proof: Maintenance admission requires an exact selected root before constructing this private pass; exact read bounds and metadata are available for that root. */ "validated exact root");
        let schema_version = root.schema_authority().schema_version();
        let authoring = root
            .canonical_envelope()
            .map(|envelope| &envelope.branch_context);
        if work.routes_installed_indexes()
            && self
                .runtime
                .indexes
                .with_definitions(|definitions| definitions.is_empty())
        {
            return Ok(PreparedRefresh {
                publication: IndexGenerationPublicationBasis::new(
                    request,
                    request.branch_id.clone(),
                    after.version_id(),
                    schema_version,
                ),
                entries: Vec::new(),
                reused: Vec::new(),
                basis: None,
            });
        }
        let mut prepared = Vec::new();
        let mut pending_fields = Vec::new();
        let mut reused = Vec::new();
        let mut cold_changes = None;
        let mut patch_changes = None;
        let mut cold_routing = None;
        let mut patch_routing = None;
        let mut basis = if let Some(before) = before {
            let previous = before
                .selected_root()
                .ok_or(DerivedIndexMaintenanceDenialKind::BeforeRootMismatch)?;
            if let Some(envelope) = previous.canonical_envelope() {
                self.runtime
                    .indexes
                    .prepare_basis_selection(
                        &request.branch_id,
                        request.source_commit_id,
                        after.version_id(),
                        schema_version,
                        &envelope.branch_context,
                        envelope.commit.commit_id,
                        before.version_id(),
                        work.publication_is_deferred(),
                        &mut |units, bytes| work.prepare(units, bytes),
                    )
                    .map(Some)
                    .or_else(|stop| {
                        if stop == DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired {
                            Ok(None)
                        } else {
                            Err(stop)
                        }
                    })?
            } else {
                None
            }
        } else if let Some(authoring) = authoring.filter(|branch| *branch != &request.branch_id) {
            self.runtime
                .indexes
                .prepare_basis_selection(
                    &request.branch_id,
                    request.source_commit_id,
                    after.version_id(),
                    schema_version,
                    authoring,
                    request.source_commit_id,
                    after.version_id(),
                    work.publication_is_deferred(),
                    &mut |units, bytes| work.prepare(units, bytes),
                )
                .map(Some)
                .or_else(|stop| {
                    if stop == DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired {
                        Ok(None)
                    } else {
                        Err(stop)
                    }
                })?
        } else {
            None
        };
        let affected = if let Some(before) = before {
            let changes = changes::ChangedRecords::patch(root, work)?;
            let affected =
                affected_indexes::select(self.runtime, execution, &changes, before, after, work)?;
            patch_changes = Some(changes);
            Some(affected)
        } else {
            None
        };
        let inherited = basis.clone();
        if let (Some(basis), Some(affected)) = (&mut basis, &affected) {
            for index in affected {
                basis.remove_admitted(*index, &mut |units, bytes| work.prepare(units, bytes))?;
            }
        }
        let requested = if work.routes_installed_indexes() {
            if basis.is_none() {
                return Err(DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired);
            }
            affected.ok_or(DerivedIndexMaintenanceDenialKind::BeforeRootMismatch)?
        } else {
            let mut requested = std::collections::BTreeSet::new();
            for index_id in &request.index_ids {
                work.ordered::<DerivedIndexId, ()>(requested.len(), 1, 0)?;
                requested.insert(*index_id);
            }
            requested
        };
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
                    compatible_before.and_then(|view| {
                        view.selected_root()?
                            .commit_id()
                            .map(|commit| (view.version_id(), schema_version, commit))
                    }),
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
                work.grow_vec(&mut reused)?;
                work.prepare(
                    1,
                    (seed.source_branch_id.0.len() + seed.applicability.branch_id.0.len()) as u64,
                )?;
                reused.push(seed.as_ref().clone());
                work.note_seeded_generations(1);
                continue;
            }
            if let (Some(prior), Some(basis)) = (&prior, &basis) {
                if basis.generation(index_id).is_some() {
                    work.grow_vec(&mut reused)?;
                    work.prepare(
                        1,
                        (prior.source_branch_id.0.len() + prior.applicability.branch_id.0.len())
                            as u64,
                    )?;
                    reused.push(prior.as_ref().clone());
                    work.note_reused_generations(1);
                    continue;
                }
            }
            let previous = prior.clone();
            let (entries, changes, old) = if let Some(prior) = prior {
                if patch_changes.is_none() {
                    patch_changes = Some(changes::ChangedRecords::patch(root, work)?);
                }
                (
                    prior.entries.clone(),
                    patch_changes.as_ref().unwrap(/* Proof: The preceding branch tests this Option as Some and does not mutate it before this borrow. */ ),
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
                    cold_changes.as_ref().unwrap(/* Proof: The preceding branch tests this Option as Some and does not mutate it before this borrow. */ ),
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
                        changed: false,
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
                        changed: false,
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
                    let changed = update_entries(
                        execution,
                        &definition,
                        &mut entries,
                        routing.as_ref().unwrap(/* Proof: The preceding branch tests this Option as Some and does not mutate it before this borrow. */ ),
                        old,
                        after,
                        work,
                    )?;
                    if !changed && let Some(previous) = previous {
                        generation_reuse::retain(previous, &mut basis, &mut reused, work)?;
                    } else {
                        work.grow_vec(&mut prepared)?;
                        prepared.push((index_id, entries));
                    }
                }
            }
        }
        if !pending_fields.is_empty() {
            let routing = self
                .runtime
                .indexes
                .definition_read_binding()
                .snapshot()
                .routing(schema_version)
                .ok_or(DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired)?;
            field::refresh(
                execution,
                &mut pending_fields,
                &routing,
                patch_changes.as_ref(),
                cold_changes.as_ref(),
                before,
                after,
                work,
            )?;
        }
        for field in pending_fields {
            let (index, entries, changed) = field.into_prepared();
            if !changed
                && let Some(previous) = inherited.as_ref().and_then(|basis| basis.generation(index))
            {
                generation_reuse::retain(previous, &mut basis, &mut reused, work)?;
            } else {
                work.grow_vec(&mut prepared)?;
                work.prepare(1, 0)?;
                prepared.push((index, entries));
            }
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
            basis,
        })
    }
}
