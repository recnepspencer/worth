//! One registered association's cache status and original-funded root custody.
use super::{adapter::InstalledIndexProvider, source_image::SourceImage};
use crate::domain_computation::{
    execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    primary_graph::{
        application_query::one_shot::outcome::material::MaterialCompletion,
        source_facts::SourceFactsAllocation, WorthQuerySelectedProductOperation,
    },
    provider_session::{WorthQueryManagedGraphWorkSession, WorthQueryManagedIndexSessionDenial},
};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::{identity::EntityId, transactions::RelationalExecutionDenialCause};
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductBranchIncarnation};

mod publication;
mod transitions;

/// Retained cache status; it does not assert current authorization or Source availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexLifecycle {
    Created,
    Ready,
    Evicted,
    Rebuilding,
    Failed,
    Disposed,
}

/// Why the retained association could not perform a requested lifecycle step.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_query) enum WorthQueryManagedIndexLifecycleDenial
{
    Execution(RelationalExecutionDenialCause),
    ForeignAssociation,
    RegistrationCapacityExceeded,
    RegistrationIdentityExhausted,
    Disposed,
    BuildInProgress,
    EpochExhausted,
    StaleImage,
    NotReady,
    Session(WorthQueryManagedIndexSessionDenial),
}
impl From<RelationalExecutionDenialCause> for WorthQueryManagedIndexLifecycleDenial {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        Self::Execution(cause)
    }
}

pub(in crate::domain_computation::primary_graph::application_query) struct ReadyIndex<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    pub(super) source: Arc<SourceImage<S, I>>,
    pub(super) nodes: Arc<dyn Any + Send + Sync>,
    // The header follows every independent root share, including eviction.
    _allocation: Arc<SourceFactsAllocation>,
}
struct Retained<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> {
    epoch: u64,
    current: CompositeCommitIdentity,
    status: WorthQueryManagedIndexLifecycle,
    root: Option<Arc<ReadyIndex<S, I>>>,
}

pub(in crate::domain_computation::primary_graph::application_query) struct ManagedIndexState<
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    pub(in crate::domain_computation::primary_graph::application_query) association:
        WorthQueryInstalledManagedIndexParent,
    pub(super) adapter: Arc<dyn InstalledIndexProvider<S, I>>,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    principal: EntityId,
    scope: EntityId,
    incarnation: ProductBranchIncarnation,
    retained: Mutex<Retained<S, I>>,
    pub(in crate::domain_computation::primary_graph::application_query) allocation:
        Arc<SourceFactsAllocation>,
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ManagedIndexState<S, I> {
    pub(super) fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        association: WorthQueryInstalledManagedIndexParent,
        adapter: Arc<dyn InstalledIndexProvider<S, I>>,
        parent: &WorthQueryManagedGraphWorkSession,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
    ) -> Result<Arc<Self>, WorthQueryManagedIndexLifecycleDenial> {
        use WorthQueryManagedIndexLifecycleDenial as Denial;
        let allocation = SourceFactsAllocation::open(execution)?;
        let storage = MaterialCompletion::new(execution, &allocation);
        storage.work(1)?;
        let context = parent
            .retain_query_source_context()
            .map_err(|_| Denial::ForeignAssociation)?;
        if context.runtime() != selected.application().runtime.authority_identity()
            || !adapter
                .installed()
                .is_installed_in(selected.application().installed_schema())
            || !adapter.installed().admits_association(&association)
            || parent.binding() != association.binding_identity()
            || parent.obligation() != association.access_contract().parent_obligation()
        {
            return Err(Denial::ForeignAssociation);
        }
        if !parent.admits_managed_product(
            selected.product().read_lease_ref(),
            &mut |units| storage.work(units),
            || RelationalExecutionDenialCause::WorkCounterOverflow,
        )? {
            return Err(Denial::ForeignAssociation);
        }
        // The unique World incarnation includes its genuine owner and a never
        // reused ordinal; no branch-name heap copy participates in the key.
        storage.arc::<Self>()?;
        Ok(Arc::new(Self {
            association,
            adapter,
            runtime: context.runtime(),
            principal: context.principal(),
            scope: context.scope(),
            incarnation: selected.product().observation().lifecycle_incarnation(),
            retained: Mutex::new(Retained {
                epoch: 0,
                current: selected.product().selected_commit().clone(),
                status: WorthQueryManagedIndexLifecycle::Created,
                root: None,
            }),
            allocation,
        }))
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn same_association(
        &self,
        other: &Self,
    ) -> bool {
        self.runtime == other.runtime
            && self.principal == other.principal
            && self.scope == other.scope
            && self.incarnation == other.incarnation
            && self.association.same_association(&other.association)
            && Arc::ptr_eq(&self.adapter, &other.adapter)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn lifecycle(
        &self,
    ) -> WorthQueryManagedIndexLifecycle {
        self.retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .status
    }
}

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> ReadyIndex<S, I> {
    pub(super) fn retain(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: Arc<SourceImage<S, I>>,
        nodes: Arc<dyn Any + Send + Sync>,
    ) -> Result<Arc<Self>, WorthQueryManagedIndexLifecycleDenial> {
        let allocation = SourceFactsAllocation::open(execution)?;
        MaterialCompletion::new(execution, &allocation).arc::<Self>()?;
        Ok(Arc::new(Self {
            source,
            nodes,
            _allocation: allocation,
        }))
    }
}
