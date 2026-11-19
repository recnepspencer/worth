//! Only a Native-declared predecessor in the selected branch image can seed birth.
use super::*;

impl<S, I, P> TypedSourceService<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    pub(super) fn prepare_ordered(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: ManagedIndexSourceScope<'_>,
        predecessor: Option<&Payload>,
    ) -> Result<PreparedManagedIndexSourceRoot, Denial> {
        let parent = scope.ordered_predecessor()?;
        let (proposed, changes) = match (parent, predecessor) {
            (Some(parent), Some(payload)) => {
                let root = payload
                    .downcast_ref::<ManagedSourceRoot<S, I>>()
                    .ok_or(Denial::ForeignSource)?;
                if !root.installed.same_installation(&self.provider.installed)
                    || root.rows.program() != scope.program()
                    || root.rows.scope() != parent
                {
                    return Err(Denial::ForeignSource);
                }
                match scope.prepare_ordered_step(payload) {
                    Ok(step) => {
                        let changed = step
                            .into_iter()
                            .map(|(entity, after)| {
                                (
                                    root.rows
                                        .entry(entity)
                                        .map(|entry| entry.occurrence.clone()),
                                    after,
                                )
                            })
                            .collect::<Vec<_>>();
                        self.update_changed(execution, &scope, payload, &changed, true)?
                    }
                    Err(Denial::PredecessorUnavailable { .. })
                        if self.cold == Cold::AdmitRebuild =>
                    {
                        self.cold_born(execution, &scope)?
                    }
                    Err(cause) => return Err(cause),
                }
            }
            (Some(predecessor), None) if self.cold == Cold::Refuse => {
                return Err(Denial::PredecessorUnavailable { predecessor })
            }
            // A parent without retained material costs one successor build
            // and one acquisition of the parent's rows, never a recursive
            // preparation of the parent; the delta stays relative to it.
            (Some(_), None) => self.cold_born(execution, &scope)?,
            // The chain root builds its own membership: every member entered.
            (None, _) => {
                let built = self.cold(execution, &scope, &scope.proposed())?;
                let changes = super::entered_all::<S, I>(&scope, &built)?;
                (built, changes)
            }
        };
        execution
            .consult()
            .map_err(|cause| source_storage(cause.into()))?;
        Ok(scope.seal(None, Some(proposed), changes))
    }
    /// A successor built without its predecessor's retained material. Its
    /// delta is still relative to the predecessor's logical membership, read
    /// from Native's proposed view, so it equals the warm step's whatever is
    /// resident.
    fn cold_born(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
    ) -> Result<
        (
            Payload,
            worth_relational::facade::runtime::ManagedIndexSourceChanges,
        ),
        Denial,
    > {
        let built = self.cold(execution, scope, &scope.proposed())?;
        let changes = self.built_changes(execution, scope, &built)?;
        Ok((built, changes))
    }
    /// The delta of a scope built new, cold or seeded: relative to its
    /// ordered predecessor's logical membership when it has one, else every
    /// member entered.
    pub(in crate::domain_computation::primary_graph::application_query::managed_index::source) fn built_changes(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        built: &Payload,
    ) -> Result<worth_relational::facade::runtime::ManagedIndexSourceChanges, Denial> {
        let Some(parent) = scope.ordered_predecessor_rows()? else {
            return super::entered_all::<S, I>(scope, built);
        };
        let root = built
            .downcast_ref::<ManagedSourceRoot<S, I>>()
            .ok_or(Denial::ForeignSource)?;
        born_relative(
            execution,
            scope,
            &root.rows,
            &parent,
            self.provider.installed.source_read().query().read_graph(),
            self.maximum_source_rows(),
        )
    }
}

/// The concrete diff, compiled once: the predecessor's rows materialized from
/// the logical view, then each built row compared by facts.
fn born_relative(
    execution: &ExecutionRequest<'_, '_>,
    scope: &ManagedIndexSourceScope<'_>,
    built: &PersistentSourceRows,
    parent: &worth_relational::facade::runtime::ManagedIndexSourceRows<'_>,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphReadContract,
    maximum: usize,
) -> Result<worth_relational::facade::runtime::ManagedIndexSourceChanges, Denial> {
    let owner = WorthQueryGraphWorkSessionIdentity::for_native_source(scope)?;
    let baseline = PersistentSourceRows::acquire(execution, parent, owner, graph, maximum)
        .map_err(super::super::occurrences::MaterialDenial::into_native)?;
    built
        .born_changes(execution, scope, &baseline)
        .map_err(super::super::occurrences::MaterialDenial::into_native)
}
