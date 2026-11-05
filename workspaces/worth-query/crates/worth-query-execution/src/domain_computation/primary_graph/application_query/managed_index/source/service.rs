//! Installed generic mechanics consume Native Source loans, never caller scope lists.
use super::failure::{source_preparation_denial, source_storage};
mod correspondence;
mod factory;
use std::{any::Any, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationManagedIndexColdSource as Cold,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndex;
use worth_relational::facade::runtime::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram as Program,
    ManagedIndexSourceScope, PreparedManagedIndexSourceRoot,
};
use {
    super::super::adapter::InstalledIndexProvider, super::super::adapter::TypedIndexProvider,
    super::super::source::occurrences::PersistentSourceRows,
    super::super::WorthQueryManagedIndexProvider,
    super::super::WorthQueryManagedIndexSourceChange as Change,
    crate::domain_computation::primary_graph::vocabulary::application_query::source_rows::WorthQueryManagedIndexSourceSlot as Slot,
};
use {
    crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
    crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
};

pub(in crate::domain_computation::primary_graph::application_query::managed_index) type Payload =
    Arc<dyn Any + Send + Sync>;

pub(in crate::domain_computation::primary_graph::application_query::managed_index) trait SourceFactory:
    Send + Sync
{
    fn definition(
        &self,
    ) -> &worth_query_declaration::facade::application_query::ErasedApplicationManagedIndexDefinition;
    fn bind(
        self: Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
        program: Program,
        cold: Cold,
        authority: super::initialization::SourceInitializationAuthority,
    ) -> Result<
        Arc<dyn SourceService>,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    >;
}
pub(in crate::domain_computation::primary_graph) trait SourceService:
    Send + Sync
{
    fn predicate_axis(&self) -> Option<&Arc<crate::domain_computation::primary_graph::application_query::derived_view::SourcePredicateAxis>>;
    fn binding_type(&self) -> std::any::TypeId;
    fn admits_initialization(
        &self,
        proof: &worth_proof::AdmittedManagedIndexSourceInitialization,
    ) -> bool;
    fn prepare_initial(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        proof: &worth_proof::AdmittedManagedIndexSourceInitialization,
        scope: ManagedIndexSourceScope<'_>,
        retained: Option<&Payload>,
    ) -> Result<PreparedManagedIndexSourceRoot, Denial>;
    fn prepare(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: ManagedIndexSourceScope<'_>,
        retained: Option<&Payload>,
    ) -> Result<PreparedManagedIndexSourceRoot, Denial>;
}

pub(in crate::domain_computation::primary_graph::application_query::managed_index) struct ManagedSourceRoot<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) installed:
        WorthQueryInstalledManagedIndex<S, I>,
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) rows:
        PersistentSourceRows,
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) nodes:
        Payload,
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) adapter:
        Arc<dyn InstalledIndexProvider<S, I>>,
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) bindings:
        worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    _allocation: Arc<SourceFactsAllocation>,
}
pub(super) struct TypedSourceService<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>, P: WorthQueryManagedIndexProvider<S,I>>
{
    pub(super) provider: Arc<TypedIndexProvider<S, I, P>>,
    pub(super) program: Program,
    cold: Cold,
    predicate: Option<Arc<crate::domain_computation::primary_graph::application_query::derived_view::SourcePredicateAxis>>,
    pub(super) initialization: super::initialization::SourceInitializationAuthority,
}
impl<S, I, P> SourceService for TypedSourceService<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    fn predicate_axis(&self) -> Option<&Arc<crate::domain_computation::primary_graph::application_query::derived_view::SourcePredicateAxis>>{
        self.predicate.as_ref()
    }

    fn admits_initialization(
        &self,
        proof: &worth_proof::AdmittedManagedIndexSourceInitialization,
    ) -> bool {
        self.initialization.admits(proof)
    }
    fn binding_type(&self) -> std::any::TypeId {
        crate::equality_type_id::EqualityTypeId::of::<(S, I)>().into_external()
    }
    fn prepare_initial(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        proof: &worth_proof::AdmittedManagedIndexSourceInitialization,
        scope: ManagedIndexSourceScope<'_>,
        retained: Option<&Payload>,
    ) -> Result<PreparedManagedIndexSourceRoot, Denial> {
        self.initial_root(execution, proof, scope, retained)
    }
    fn prepare(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: ManagedIndexSourceScope<'_>,
        retained: Option<&Payload>,
    ) -> Result<PreparedManagedIndexSourceRoot, Denial> {
        if scope.program() != &self.program {
            return Err(Denial::ForeignSource);
        }
        let proposed_exists = scope.scope_exists()?;
        if !proposed_exists
            || scope.physical_availability()
                == worth_relational::facade::runtime::ManagedIndexSourceAvailability::Unavailable
        {
            self.invalidate_predicates(execution, &scope)?;
        }
        if proposed_exists && let Some(payload) = self.corresponding_root(execution, &scope)? {
            return Ok(scope.seal(None, Some(payload)));
        }
        // Unavailable is an explicit Native physical posture, not an empty
        // membership result. All other before-view failures retain their cause.
        let before_exists = match scope.committed_scope_exists() {
            Ok(exists) => exists,
            Err(Denial::RootUnavailable) => {
                self.invalidate_predicates(execution, &scope)?;
                false
            }
            Err(cause) => return Err(cause),
        };
        let before = if before_exists {
            match retained {
                Some(retained) => {
                    let root = retained
                        .downcast_ref::<ManagedSourceRoot<S, I>>()
                        .ok_or(Denial::ForeignSource)?;
                    if !root.installed.same_installation(&self.provider.installed) {
                        return Err(Denial::ForeignSource);
                    }
                    Some(Arc::clone(retained))
                }
                None if self.cold == Cold::AdmitRebuild => {
                    Some(self.cold(execution, &scope, &scope.committed()?)?)
                }
                None => return Err(Denial::RootUnavailable),
            }
        } else {
            None
        };
        if !proposed_exists {
            return Ok(scope.seal(before, None));
        }
        let proposed = if let Some(before) = &before {
            Some(self.update(execution, &scope, before)?)
        } else if self.cold == Cold::AdmitRebuild {
            Some(self.cold(execution, &scope, &scope.proposed())?)
        } else {
            return Err(Denial::RootUnavailable);
        };
        execution.consult().map_err(|cause| {
            Denial::Source(
                worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause.into()),
            )
        })?;
        Ok(scope.seal(before, proposed))
    }
}

impl<S, I, P> TypedSourceService<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    fn invalidate_predicates(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
    ) -> Result<(), Denial> {
        if let Some(axis) = &self.predicate {
            super::super::super::derived_view::mark_prepared_predicates_unavailable(
                execution,
                scope.preparation_metadata(),
                axis,
                scope.scope(),
                &scope.read_budget(),
            )?;
        }
        Ok(())
    }
    pub(super) fn cold(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        loan: &worth_relational::facade::runtime::ManagedIndexSourceRows<'_>,
    ) -> Result<Payload, Denial> {
        let owner = WorthQueryGraphWorkSessionIdentity::for_native_source(scope)?;
        let graph = self.provider.installed.source_read().query().read_graph();
        let rows = PersistentSourceRows::acquire(
            execution,
            loan,
            owner,
            graph,
            self.maximum_source_rows(),
        )
        .map_err(super::super::source::occurrences::MaterialDenial::into_native)?;
        let budget = scope.read_budget();
        let nodes = self
            .provider
            .build(
                execution,
                owner,
                rows.view(&self.provider.bindings, Some(&budget)),
            )
            .map_err(|cause| source_preparation_denial(execution, cause))?;
        retain_root(
            execution,
            self.provider.clone(),
            self.provider.bindings.clone(),
            rows,
            nodes,
        )
    }
    fn update(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
        retained: &Payload,
    ) -> Result<Payload, Denial> {
        let before = retained
            .downcast_ref::<ManagedSourceRoot<S, I>>()
            .ok_or(Denial::ForeignSource)?;
        let changed = scope.prepare_changed_occurrences()?;
        if changed.is_empty() {
            return Ok(Arc::clone(retained));
        }
        let graph = self.provider.installed.source_read().query().read_graph();
        let rows = before
            .rows
            .update(execution, scope, &changed, graph)
            .map_err(super::super::source::occurrences::MaterialDenial::into_native)?;
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
            super::super::super::derived_view::route_prepared_predicates(
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
    fn maximum_source_rows(&self) -> usize {
        let source = self.provider.installed.source_read();
        source.limits().maximum_results().get().min(
            std::num::NonZeroUsize::new(source.query().read_graph().maximum_result_count())
                .map_or(usize::MAX, std::num::NonZeroUsize::get),
        )
    }
}

pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn retain_root<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
>(
    execution: &ExecutionRequest<'_, '_>,
    adapter: Arc<dyn InstalledIndexProvider<S, I>>,
    bindings: worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    rows: PersistentSourceRows,
    nodes: Payload,
) -> Result<Payload, Denial> {
    // The backing itself, not an old request, follows the immutable share.
    let allocation = SourceFactsAllocation::open(execution).map_err(source_storage)?;
    super::super::super::one_shot::outcome::material::MaterialCompletion::new(
        execution,
        &allocation,
    )
    .arc::<ManagedSourceRoot<S, I>>()
    .map_err(source_storage)?;
    Ok(Arc::new(ManagedSourceRoot {
        installed: adapter.installed().clone(),
        adapter,
        bindings,
        rows,
        nodes,
        _allocation: allocation,
    }))
}
