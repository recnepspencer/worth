//! Installed generic mechanics consume Native Source loans, never caller scope lists.
use super::super::{
    adapter::{InstalledIndexProvider, TypedIndexProvider},
    source::occurrences::PersistentSourceRows,
    WorthQueryManagedIndexProvider, WorthQueryManagedIndexSourceChange as Change,
    WorthQueryManagedIndexSourceSlot as Slot,
};
use crate::domain_computation::{
    primary_graph::source_facts::SourceFactsAllocation,
    provider_session::WorthQueryGraphWorkSessionIdentity,
};
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
    ManagedIndexSourceProviderFailure, ManagedIndexSourceScope, PreparedManagedIndexSourceRoot,
};

pub(in crate::domain_computation::primary_graph::application_query::managed_index) type Payload =
    Arc<dyn Any + Send + Sync>;

pub(in crate::domain_computation::primary_graph::application_query::managed_index) trait SourceFactory:
    Send + Sync
{
    fn definition(
        &self,
    ) -> &worth_query_declaration::facade::application_query::ErasedApplicationManagedIndexDefinition;
    fn bind(self: Arc<Self>, program: Program, cold: Cold) -> Arc<dyn SourceService>;
}
pub(in crate::domain_computation::primary_graph) trait SourceService:
    Send + Sync
{
    fn program(&self) -> &Program;
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
struct TypedSourceService<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>, P> {
    provider: Arc<TypedIndexProvider<S, I, P>>,
    program: Program,
    cold: Cold,
}
impl<S, I, P> SourceFactory for TypedIndexProvider<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    fn definition(
        &self,
    ) -> &worth_query_declaration::facade::application_query::ErasedApplicationManagedIndexDefinition
    {
        self.installed.definition()
    }
    fn bind(self: Arc<Self>, program: Program, cold: Cold) -> Arc<dyn SourceService> {
        Arc::new(TypedSourceService {
            provider: self,
            program,
            cold,
        })
    }
}
impl<S, I, P> SourceService for TypedSourceService<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    fn program(&self) -> &Program {
        &self.program
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
        // Unavailable is an explicit Native physical posture, not an empty
        // membership result. All other before-view failures retain their cause.
        let before_exists = match scope.committed_scope_exists() {
            Ok(exists) => exists,
            Err(Denial::RootUnavailable) => false,
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
    fn cold(
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
            .map_err(|cause| retain_failure(execution, cause))?;
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
        let changed = scope.changed_occurrences()?;
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
            .map_err(|cause| retain_failure(execution, cause))?;
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

#[derive(Debug)]
struct PreparationFailure<E: std::error::Error + Send + Sync + 'static> {
    cause: E,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl<E: std::error::Error + Send + Sync + 'static> std::fmt::Display for PreparationFailure<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cause)
    }
}
impl<E: std::error::Error + Send + Sync + 'static> std::error::Error for PreparationFailure<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
fn source_storage(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Denial {
    Denial::Source(worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause))
}
pub(super) fn retain_failure<E: std::error::Error + Send + Sync + 'static>(
    execution: &ExecutionRequest<'_, '_>,
    cause: E,
) -> Denial {
    if let Err(stop) = execution.consult() {
        return source_storage(stop.into());
    }
    let Some(bytes) = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<PreparationFailure<E>>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
    else {
        return source_storage(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow);
    };
    let memory = match execution.reserve_retained_memory(bytes) {
        Ok(memory) => memory,
        Err(stop) => return source_storage(stop.into()),
    };
    Denial::Provider(ManagedIndexSourceProviderFailure::from_owned(Arc::new(
        PreparationFailure {
            cause,
            _memory: memory,
        },
    )))
}
