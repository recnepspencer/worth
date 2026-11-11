//! Apply one admitted same-scope or predecessor step through the existing node edit.
use super::*;
impl<S, I, P> TypedSourceService<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    pub(super) fn update(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        retained: &Payload,
    ) -> Result<Payload, Denial> {
        let changed = scope.prepare_changed_occurrences()?;
        self.update_changed(execution, scope, retained, &changed, false)
    }
    pub(super) fn update_changed(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        retained: &Payload,
        changed: &[(
            Option<worth_relational::facade::runtime::ManagedIndexSourceOccurrence>,
            Option<worth_relational::facade::runtime::PreparedManagedIndexSourceOccurrence>,
        )],
        inherited: bool,
    ) -> Result<Payload, Denial> {
        let before = retained
            .downcast_ref::<ManagedSourceRoot<S, I>>()
            .ok_or(Denial::ForeignSource)?;
        if changed.is_empty() && !inherited {
            return Ok(Arc::clone(retained));
        }
        let graph = self.provider.installed.source_read().query().read_graph();
        let baseline = if inherited {
            Some(before.rows.handoff(scope)?)
        } else {
            None
        };
        let baseline = baseline.as_ref().unwrap_or(&before.rows);
        let rows = baseline
            .update(execution, scope, changed, graph, inherited)
            .map_err(super::super::occurrences::MaterialDenial::into_native)?;
        if rows.len() > self.maximum_source_rows() {
            return Err(
                worth_relational::facade::runtime::DerivedRelationReadDenial::Definition(
                    worth_relational::facade::schema::DerivedRelationDenial::WorkBudgetExceeded,
                )
                .into(),
            );
        }
        let allocation = SourceFactsAllocation::open(execution).map_err(source_storage)?;
        allocation
            .claim_slice::<Change>(execution, changed.len())
            .map_err(source_storage)?;
        let mut changes = Vec::new();
        changes.try_reserve_exact(changed.len()).map_err(|_| source_storage(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed,
        ))?;
        changes.extend(changed.iter().map(|(old, new)| Change {
            before: old.as_ref().map(|row| Slot {
                owner: before.rows.owner(),
                ordinal: 0,
                entity: Some(row.entity()),
            }),
            after: new.as_ref().map(|row| Slot {
                owner: before.rows.owner(),
                ordinal: 0,
                entity: Some(row.entity()),
            }),
        }));
        let bindings = &self.provider.bindings;
        let budget = scope.read_budget();
        let nodes = self
            .provider
            .update(
                execution,
                before.rows.view(bindings, Some(&budget)),
                rows.view(bindings, Some(&budget)),
                &changes,
                before.nodes.as_ref(),
            )
            .map_err(|cause| source_preparation_denial(execution, cause))?;
        if let Some(axis) = &self.predicate {
            super::super::super::super::derived_view::route_prepared_predicates(
                execution,
                scope.preparation_metadata(),
                axis,
                scope.scope(),
                before.rows.view(bindings, Some(&budget)),
                rows.view(bindings, Some(&budget)),
                &changes,
            )?;
        }
        retain_root(
            execution,
            self.provider.clone(),
            self.provider.bindings.clone(),
            rows,
            nodes,
        )
    }
}
