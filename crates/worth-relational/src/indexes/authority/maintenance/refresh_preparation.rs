use super::*;

impl super::owner::IndexMaintenanceOwner<'_> {
    pub(super) fn prepare_refresh(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &DerivedIndexBuildRequest,
        before: Option<&root_view::IndexRootView>,
        after: &root_view::IndexRootView,
        work: &mut MaintenanceWork,
    ) -> Result<PreparedRefresh, DerivedIndexMaintenanceDenialKind> {
        self.prepare_refresh_with_inputs(execution, request, before, after, work, None)
    }
    pub(super) fn prepare_refresh_with_inputs(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &DerivedIndexBuildRequest,
        before: Option<&root_view::IndexRootView>,
        after: &root_view::IndexRootView,
        work: &mut MaintenanceWork,
        inputs: Option<super::refresh_inputs::RootMaintenanceInputs>,
    ) -> Result<PreparedRefresh, DerivedIndexMaintenanceDenialKind> {
        let root = after.root();
        let schema_version = root.schema_authority().schema_version();
        let authoring = root
            .canonical_envelope()
            .map(|envelope| &envelope.branch_context);
        if work.routes_installed_indexes()
            && self
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
        let mut patch_routing = None;
        let super::refresh_inputs::RootMaintenanceInputs {
            mut basis,
            inherited,
            mut patch_changes,
            requested,
        } = match inputs {
            Some(inputs) => inputs,
            None => self.refresh_inputs(execution, request, before, after, work)?,
        };
        for index_id in requested {
            let compatible_before = before.filter(|view| {
                Some(view.root().schema_authority().schema_version()) == Some(schema_version)
            });
            let (definition, current, prior) = self
                .indexes
                .maintenance_inputs_admitted(
                    index_id,
                    &request.branch_id,
                    request.source_commit_id,
                    after.version_id(),
                    compatible_before.and_then(|view| {
                        view.root()
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
            if let (Some(prior), Some(basis)) = (&prior, &basis)
                && basis.generation(index_id).is_some()
            {
                work.grow_vec(&mut reused)?;
                work.prepare(
                    1,
                    (prior.source_branch_id.0.len() + prior.applicability.branch_id.0.len()) as u64,
                )?;
                reused.push(prior.as_ref().clone());
                work.note_reused_generations(1);
                continue;
            }
            let empty_before = inherited
                .as_ref()
                .is_some_and(|basis| basis.generation_is_empty(index_id));
            if before.is_none() && empty_before {
                if let Some(basis) = &mut basis {
                    basis.remove_admitted(index_id, &mut |units, bytes| {
                        work.prepare(units, bytes)
                    })?;
                }
                work.grow_vec(&mut prepared)?;
                prepared.push((index_id, empty_entries(&definition.kind, work)?));
                continue;
            }
            let previous = prior.clone();
            let (entries, changes, old) = if let Some(prior) = prior {
                if patch_changes.is_none() {
                    patch_changes = Some(changes::ChangedRecords::patch(
                        root.commit_change()
                            .ok_or(DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired)?,
                        work,
                    )?);
                }
                (
                    prior.entries.clone(),
                    patch_changes.as_ref().unwrap(/* Proof: The preceding branch tests this Option as Some and does not mutate it before this borrow. */ ),
                    before,
                )
            } else if empty_before && before.is_some() {
                (
                    empty_entries(&definition.kind, work)?,
                    patch_changes
                        .as_ref()
                        .ok_or(DerivedIndexMaintenanceDenialKind::BeforeRootMismatch)?,
                    before,
                )
            } else {
                return Err(DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired);
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
                    let routing = &mut patch_routing;
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
                .indexes
                .definition_read_binding()
                .snapshot()
                .routing(after.root().schema_authority().authority_digest())
                .ok_or(DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired)?;
            field::refresh(
                execution,
                &mut pending_fields,
                &routing,
                patch_changes.as_ref(),
                None,
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
